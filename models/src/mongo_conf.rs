//! Initialize and return a connection to the MongoDB database.

use mongodb::{IndexModel, bson, options};
use tracing::instrument;
use users::{Users, auth::Token};

use crate::{DataBases, settings};

#[must_use]
pub fn database_from_name(manager: &mongodb::Client, database_name: &str) -> mongodb::Database {
    manager.database(database_name)
}

/// # Errors
///
///    - Returns an error if the settings are erroneously unavailable
#[must_use = "The connection pool must be used to interact with the database"]
#[instrument(
    name = "Get Connection Pool for MongoDb",
    level = "info",
    target = "podcasters",
    skip(manager)
)]
pub async fn establish_connection(manager: &mongodb::Client) -> anyhow::Result<mongodb::Database> {
    tracing::info!("Get mongo connection pool");

    let settings = settings::get()?;
    // .expect("Application settings are unavailable");

    Ok(database_from_name(manager, &settings.mongo.db))
}

/// # Errors
///
///   - This function will error if the database query fails
pub async fn get_user(
    filter: mongodb::bson::Document,
    mongo_client: &mongodb::Client,
) -> Option<users::Users> {
    let db: mongodb::Collection<Users> = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Users>("users");

    tracing::info!("Checking the cache-missed info against the db: {filter}");

    match db.find_one(filter).await {
        Ok(user) => {
            let user: Option<Users> = user;
            tracing::info!("user data queried");
            user.as_ref()?;
            user
        }
        Err(err) => {
            tracing::error!("No query possible: {err}");
            None
        }
    }
}

/// # Errors
///
///   - Error if the database is unavailble;
/// # Notes
///
///  - Only refresh the session token if it doesn't exist;
pub async fn set_token(
    filter: mongodb::bson::Document,
    mongo_client: &mongodb::Client,
    session: Token,
) -> anyhow::Result<()> {
    let db: mongodb::Collection<users::Users> = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Users>("users");

    let update_doc = mongodb::bson::doc! {
        "$set": {
            "session": {
                "opaque": session.get(),
                "expiration": session.expiration()
            }
        }
    };

    let idx_ops: options::IndexOptions =
        options::IndexOptions::builder().unique(Some(true)).build();

    let idx: IndexModel = IndexModel::builder()
        .keys(bson::doc! {"session": 1})
        .options(idx_ops)
        .build();

    match db.create_index(idx).await {
        Ok(val) => {
            tracing::info!("ttl set for the session key: {val:#?}");
        }
        Err(err) => {
            tracing::error!("Index creation error: {err}");
            return Err(err.into());
        }
    }

    match db.update_one(filter, update_doc).await {
        Ok(result) => {
            // tracing::info!("Key updated: {result:#?}");
            if result.modified_count.eq(&1) {
                tracing::info!("update successful!");
            }
        }
        Err(err) => {
            tracing::error!("Unable to save the new session token: {err}");
            return Err(anyhow::Error::msg(format!("{err}")));
        }
    }

    Ok(())
}

/// # Errors
///
///  - Error if the database is unavailable;
pub async fn remove_token(
    filter: mongodb::bson::Document,
    mongo_client: &mongodb::Client,
) -> anyhow::Result<()> {
    let db: mongodb::Collection<users::Users> = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Users>("users");

    let del_token = mongodb::bson::doc! {
        "$set": {"session": {"opaque": "", "expiration": bson::DateTime::now()}}
    };

    match db.update_one(filter, del_token).await {
        Ok(val) => {
            tracing::info!("Update result: {val:#?}");
            Ok(())
        }
        Err(err) => {
            tracing::error!("Delete token error: {err}");
            Err(anyhow::Error::msg(format!("{err}")))
        }
    }
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
