//! Initialize and return a connection to the MongoDB database.

use tracing::instrument;

use crate::settings;

#[must_use]
pub fn database_from_name(manager: &mongodb::Client, database_name: &str) -> mongodb::Database {
    manager.database(database_name)
}

#[must_use = "The connection pool must be used to interact with the database"]
#[instrument(
    name = "Get Connection Pool for MongoDb",
    level = "info",
    target = "podcasters",
    skip(manager)
)]
pub async fn establish_connection(manager: &mongodb::Client) -> anyhow::Result<mongodb::Database> {
    tracing::info!("Get mongo connection pool");

    let settings = settings::get().expect("Application settings are unavailable");

    Ok(database_from_name(manager, &settings.mongo.db))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[tokio::test]
    #[case("development")]
    #[case("test")]
    #[case("podcasters")]
    async fn database_from_name_selects_database(#[case] database_name: &str) {
        let client = mongodb::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();

        let database = database_from_name(&client, database_name);

        assert_eq!(database.name(), database_name);
    }

    #[rstest]
    #[case("development")]
    #[case("test")]
    #[case("podcasters")]
    #[case("podcasters_ci")]
    #[tokio::test]
    async fn database_from_name_selects_expected_database(#[case] database_name: &str) {
        let client = mongodb::Client::with_uri_str("mongodb://127.0.0.1:27017")
            .await
            .unwrap();

        let database = database_from_name(&client, database_name);

        assert_eq!(database.name(), database_name);
    }
}
