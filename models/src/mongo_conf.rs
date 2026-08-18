//! Initialize and return a connection to the ``MongoDb`` database.

use tracing::instrument;

use crate::settings;

#[must_use = "The connection pool must be used to interact with the database"]
#[instrument(
    name = "Get Connection Pool for MongoDb",
    level = "info",
    target = "sundayLifeServices web app",
    skip(manager)
)]
/// # Result
///  - `Ok(Database)` if the connection pool was successfully created
/// # Errors
///  - `mongodb::error::Error` if the connection pool could not be created
/// # Panics
///  - If the connection application settings are unavaible
pub async fn establish_connection(manager: &mongodb::Client) -> anyhow::Result<mongodb::Database> {
    tracing::info!("Get mongo connection pool");
    let settings = settings::get().expect("Application settings are unavailable");
    Ok(manager.database(&settings.mongo.db))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {}
