use std::net;

use actix_web::{self, App, HttpServer, http::KeepAlive, middleware, web};
use shared::settings;
use tracing::{instrument, warn};

use crate::{
    endpoints::{self, discover, index, podcasts},
    images,
};

pub const PARSE_COUNT: u8 = 9;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[instrument(
    name = "Running the server",
    target = "demo_web_app",
    level = "info",
    skip(listener, settings)
)]
async fn run(
    listener: std::net::TcpListener,
    settings: settings::Settings,
) -> Result<actix_web::dev::Server, std::io::Error> {
    let (redis_pool, mongo_pool) = match models::init_db().await {
        Ok((red, mong)) => (red, mong),
        Err(err) => {
            tracing::error!(err);
            panic!("Unable to init the app due to the lack of a DB connection");
        }
    };

    // Connect to the MongoDB database
    let db_redis = web::Data::new(redis_pool);
    let db_mongo = web::Data::new(mongo_pool);
    tracing::info!("Processed DB & Cache connection pool for distribution");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION")))) // Security consideration
            .app_data(db_redis.clone())
            .app_data(db_mongo.clone())
            .service(
                web::scope("/static")
                    .service(images::favicon)
                    .service(images::icon_192)
                    .service(images::icon_512)
                    .service(images::icon_large)
                    .service(images::link_preview)
                    .service(images::manifest)
                    .service(images::logomain)
                    .service(images::usmc_patrolling)
                    .service(images::stylesheet)
                    .service(images::source_map)
                    .service(images::htmx)
                    .service(images::response_targets)
                    .service(images::sse)
                    .service(images::action_script)
                    .service(images::prof_headshot)
                    .service(images::spinner)
                    .service(images::github)
                    .service(images::linkedin)
                    .service(images::settings_icon)
                    .service(images::random_images),
            )
            .service(index::index)
            .service(endpoints::health)
            .service(podcasts::podcasts)
            .service(discover::discover)
        // .service(endpoints::bs_logic::schedule)
        // .service(endpoints::bs_logic::testimonials)
        // .service(endpoints::bs_logic::finances)
        // .service(endpoints::bs_logic::contact)
        // .service(
        //     web::scope("/v1")
        //         .service(index::create_post)
        //         .service(login::login_template)
        //         .service(login::login_user)
        //         .service(logout::logout)
        //         .service(register::register_template)
        //         .service(register::register_user)
        //         .service(settings::settings_template)
        //         .service(settings::settings_change)
        //         .service(validate_email::validate_email)
        //         .service(user_input::submit_text)
        //         .service(user_input::edit_submission)
        //         .service(user_input::editor_submission)
        //         .service(user_input::update_text)
        //         .service(user_input::delete_submission)
        //         .service(user_input::post_image),
        // )
        // .route("/sse", web::get().to(index::sse))
    })
    .keep_alive(KeepAlive::Os) // Keep the connection alive; OS handled
    .disable_signals() // Disable the signals to allow the OS to handle the signals
    .workers(2)
    .shutdown_timeout(3)
    .listen(listener)?
    .run();

    if settings.debug {
        warn!("Debug mode");
    } else {
        warn!("Production mode");
    }

    Ok(server)
}

pub struct Application {
    port: u16,
    server: actix_web::dev::Server,
}

impl Application {
    /// # Result
    ///  - `Ok(Application)` if the application was successfully built
    /// # Errors
    ///  - `std::io::Error` if the application could not be built
    /// # Panics
    ///  - If the application could not be built
    #[instrument(
        name = "Build Application",
        level = "info",
        target = "demo_web_app",
        skip(settings)
    )]
    pub async fn build(settings: &mut settings::Settings) -> Result<Self, std::io::Error> {
        tracing::info!("Buidling the main application");

        let app_address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );

        tracing::info!("Binding the TCP port: {app_address}");
        let listener: net::TcpListener = net::TcpListener::bind(&app_address)?;
        let port = listener.local_addr()?.port();
        let server = run(listener, settings.clone()).await?;

        Ok(Self { port, server })
    }

    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// # Result
    ///  - `Ok(())` if the application was successfully started
    /// # Errors
    ///  - `std::io::Error` if the application could not be started
    /// # Panics
    ///  - If the application could not be started
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        tracing::info!("Running until stopped");
        self.server.await
    }
}
