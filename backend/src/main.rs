use personal_journal::{settings, startup::Application, telemetry};
use std::io;
use tracing::{error, info, warn};

#[actix_web::main]
async fn main() -> io::Result<()> {
    // This is a macro that allows for multiple loggers to be used at once

    dotenvy::dotenv().ok();

    let mut settings = match settings::get() {
        Ok(settings) => settings,
        Err(err) => {
            println!("Failed to load settings: {err}");
            panic!("Failed to load settings");
        }
    };

    let subscriber = telemetry::get_subcriber(settings.clone().debug);
    telemetry::init_subscriber(subscriber);

    info!("Building the application");
    let application = match Application::build(&mut settings).await {
        Ok(app) => app,
        Err(err) => {
            error!("Failed to build application: {err}");
            panic!("Failed to build application");
        }
    };

    info!("Listening on port: {}", application.port());
    application.run_until_stopped().await?;
    warn!("Shutting down");

    Ok(())
}
