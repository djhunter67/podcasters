use crate::api::{self};
use actix_cors::Cors;
use actix_web::{
    self, App, HttpResponse, HttpServer, error,
    http::{KeepAlive, header},
    middleware, web,
};
use models;
use shared::settings;
use std::{net, time};
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
    let mongo_pool = match models::init_db().await {
        Ok(mong) => {
            tracing::info!("Database connection established");
            mong
        }
        Err(err) => {
            tracing::error!(err);
            panic!("Unable to init the app due to the lack of a DB connection");
        }
    };

    let redis_client: redis::Client = match redis::Client::open(settings.redis.uri.clone()) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to connect to the cache layer: {err:#?}");
            panic!("Application cannot start: {err:#?}")
            // try to connect to a locally running instance of redis
        }
    };

    let redis_config = redis::aio::ConnectionManagerConfig::new()
        .set_connection_timeout(Some(time::Duration::from_secs(2))) // Time to establish TCP connection
        .set_response_timeout(Some(time::Duration::from_secs(1))) // Time to wait for command response
        .set_exponent_base(2.) // Exponential backoff base
        .set_number_of_retries(3); // Max retries before failing

    let redis_pool: redis::aio::ConnectionManager =
        match redis::aio::ConnectionManager::new_with_config(redis_client, redis_config).await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::error!("Unable to connect to the cache layer: {err:#?}");
                panic!("Application cannot start: {err:#?}")
            }
        };

    // Connect to the MongoDB database
    let db_redis: web::Data<redis::aio::ConnectionManager> = web::Data::new(redis_pool);
    let db_mongo: web::Data<mongodb::Client> = web::Data::new(mongo_pool);
    tracing::info!("Processed DB & Cache connection pool for distribution");

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            // .allowed_origin("https://app.example.com")
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
            .max_age(3600);

        let json_config = web::JsonConfig::default()
            .limit(409)
            .error_handler(|err, _req| {
                // create custom error response
                error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION")))) // Security consideration
            .wrap(cors)
            .app_data(json_config)
            .app_data(db_redis.clone())
            .app_data(db_mongo.clone())
            .service(
                web::scope("/v1")
                    .service(api::v1::podcasts::preview)
                    .service(api::v1::podcasts::set_podcast)
                    .service(api::v1::podcasts::get_podcast)
                    .service(api::v1::podcasts::get_episode)
                    .service(api::health),
            )
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
