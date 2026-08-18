use backend::{startup::Application, telemetry};
use shared::settings;
use std::io;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // This is a macro that allows for multiple loggers to be used at once

    dotenvy::dotenv().ok();

    let mut settings = match settings::get() {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("Failed to load settings: {err}");
            panic!("Failed to load settings: {err:#?}");
        }
    };

    let subscriber = telemetry::get_subcriber(settings.clone().debug);
    telemetry::init_subscriber(subscriber);

    tracing::info!("Building the application");
    let application = match Application::build(&mut settings).await {
        Ok(app) => app,
        Err(err) => {
            tracing::error!("Failed to build application: {err}");
            panic!("Failed to build application: {err:#?}");
        }
    };

    tracing::info!("Listening on port: {}", application.port());
    application.run_until_stopped().await?;
    tracing::warn!("Shutting down");

    Ok(())
}
