//! Initialize and return a connection to the ``MongoDb`` database.

use mongodb::{
    IndexModel,
    bson::{DateTime as BsonDateTime, doc, oid::ObjectId},
    options::IndexOptions,
};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

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
    info!("Get mongo connection pool");
    let settings = settings::get().expect("Application settings are unavailable");
    Ok(manager.database(&settings.mongo.db))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalDraft {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: ObjectId,
    pub state: DraftState,
    pub title: String,
    pub body: String,
    pub author: String,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
    pub revision: i64,
}

impl JournalDraft {
    pub fn to_name() -> String {
        String::from("journal_drafts")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftState {
    Active,
    Published,
}

/// # Errors
///
/// - `mongodb::error::Error` if the index could not be created
pub async fn create_journal_draft_indexes(
    collection: &mongodb::Collection<JournalDraft>,
) -> mongodb::error::Result<()> {
    let options = IndexOptions::builder()
        .name("one_active_draft_per_user".to_owned())
        .unique(true)
        .build();

    let index = IndexModel::builder()
        .keys(doc! {
        "user_id": 1,

        })
        .options(options)
        .build();

    collection.create_index(index).await?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {}
