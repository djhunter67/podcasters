use crate::api;
use actix_web::{self, App, HttpServer, http::KeepAlive, middleware, web};
use models;
use shared::settings;
use std::net;
use tracing::{instrument, warn};

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
            .service(web::scope("/v1").service(api::health))
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
