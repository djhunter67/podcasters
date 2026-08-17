//! Initialize and return a connection to the ``Redis`` database.

use mongodb::bson::oid::ObjectId;

use redis::{AsyncCommands, aio};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::mongo;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub oid: String,
}

#[derive(Debug)]
pub enum AuthenticationError {
    MissingSession,
    InvalidSession,
    Redis,
}

/// # Errors
///
/// - `AuthenticationError::MissingSession` if the session cookie is missing
/// - `AuthenticationError::InvalidSession` if the session is invalid
/// - `AuthenticationError::Redis` if there is an error connecting to Redis
#[allow(clippy::future_not_send)]
#[instrument(
    name = "Serving main page",
    level = "debug",
    target = "actix_web::main",
    skip(redis_client, mongo_client)
)]
pub async fn authenticated_user_id(
    mongo_client: &mongodb::Client,
    redis_client: &mut aio::ConnectionManager,
) -> Result<ObjectId, AuthenticationError> {
    // let session_cookie = req
    // .cookie("session_id")
    // .ok_or(AuthenticationError::MissingSession)?;
    let session_cookie = "cookie_string";

    // let user_session = format!("session:{}", session_cookie.value());
    let user_session = format!("session:{}", session_cookie);

    tracing::debug!("Establishing the Redis connection");
    // let mut redis_conn =
    // establish_connection(redis_client).map_err(|_| AuthenticationError::Redis)?;

    tracing::debug!("Getting the user from the session");
    let user_email: Option<String> = redis_client
        // .as_ref()
        // .clone()
        .get(&user_session)
        .await
        .map_err(|_| AuthenticationError::Redis)?;

    tracing::debug!("Checking that the email to check against is valid: {user_email:#?}");

    let cache_key = format!("user:auth:{}", user_email.clone().unwrap_or_default());
    let user_id: Option<String> = redis_client
        // .as_ref()
        // .clone()
        .get(&cache_key)
        .await
        .map_err(|_| AuthenticationError::Redis)?;

    tracing::debug!("Checking that the serialized session is valid: {user_id:#?}");
    // I need an email for the user to be able to get the oid from Mongo
    let user_id = match user_id {
        None => {
            tracing::error!("Cache-Miss: {user_session}");
            let mongo_client = mongo::establish_connection(mongo_client)
                .await
                .map_err(|_| AuthenticationError::InvalidSession)?;

            let filter = mongodb::bson::doc! { "email": user_email.clone().unwrap_or_default() };
            let user_doc = mongo_client
                .collection::<mongodb::bson::Document>("development")
                .find_one(filter)
                .await
                .map_err(|_| AuthenticationError::InvalidSession)?;

            tracing::debug!("Checking that the db user data is valid: {user_doc:?}");

            if let Some(user_doc) = user_doc
                && let Ok(user_id) = user_doc.get_object_id("_id")
            {
                tracing::debug!("User ID found in MongoDB: {user_id}");
                return Ok(user_id);
            }

            tracing::error!(
                "No user data associated with the received session key: {user_session}",
            );
            Err(AuthenticationError::MissingSession)?
        }
        Some(user_bson_oid) => {
            tracing::debug!(
                "Cache-Hit: {}",
                user_bson_oid
                    .split(':')
                    .next_back()
                    .unwrap_or_default()
                    .trim_matches('}')
                    .trim_matches('"')
            );
            user_bson_oid
                .split(':')
                .next_back()
                .unwrap_or_default()
                .trim_matches('}')
                .trim_matches('"')
                .parse::<ObjectId>()
                .map_err(|err| {
                    tracing::error!("Failed to parse ObjectId from session: {err:#?}");
                    AuthenticationError::InvalidSession
                })?
        }
    };

    // .ok_or(AuthenticationError::InvalidSession)?;

    // let session: UserSession =
    // serde_json::from_str(&user_id).map_err(|_| AuthenticationError::InvalidSession)?;

    // tracing::debug!("Passing back the ObjectId from the session: {user_id:#?}");
    // ObjectId::parse_str(session).map_err(|_| AuthenticationError::InvalidSession)
    Ok(user_id)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {}
