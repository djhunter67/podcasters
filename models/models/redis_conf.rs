//! Initialize and return a connection to the ``Redis`` database.

use actix_web::{HttpRequest, web::Data};
use mongodb::bson::oid::ObjectId;

use redis::{AsyncCommands, aio};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::models::mongo;

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
    skip(redis_client, req, mongo_client)
)]
pub async fn authenticated_user_id(
    req: &HttpRequest,
    mongo_client: &Data<mongodb::Client>,
    redis_client: &Data<aio::ConnectionManager>,
) -> Result<ObjectId, AuthenticationError> {
    let session_cookie = req
        .cookie("session_id")
        .ok_or(AuthenticationError::MissingSession)?;

    let user_session = format!("session:{}", session_cookie.value());

    tracing::debug!("Establishing the Redis connection");
    // let mut redis_conn =
    // establish_connection(redis_client).map_err(|_| AuthenticationError::Redis)?;

    tracing::debug!("Getting the user from the session");
    let user_email: Option<String> = redis_client
        .as_ref()
        .clone()
        .get(&user_session)
        .await
        .map_err(|_| AuthenticationError::Redis)?;

    tracing::debug!("Checking that the email to check against is valid: {user_email:#?}");

    let cache_key = format!("user:auth:{}", user_email.clone().unwrap_or_default());
    let user_id: Option<String> = redis_client
        .as_ref()
        .clone()
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
mod tests {
    use r2d2::Pool;
    use redis::{Cmd, Commands, ConnectionLike, Value};
    use rstest::{fixture, rstest};
    use std::{num::NonZero, thread::spawn, time::Duration};

    use crate::settings::{self};

    #[fixture]
    fn pool() -> Pool<redis::Client> {
        let manager = redis::Client::open(settings::get().unwrap().redis.uri)
            .expect("Failed to create Redis client");
        Pool::builder()
            .max_size(15)
            .connection_timeout(Duration::from_secs(5))
            .build(manager)
            .expect("Failed to create Redis connection pool")
    }

    #[rstest]
    fn test_can_write_to_redis(pool: Pool<redis::Client>) {
        let mut conn = pool.get().unwrap();

        // Test query
        conn.set::<&str, &str, String>("test", "test").unwrap();
        let result: String = conn.get("test").unwrap();
        assert_eq!(result, "test");

        // Clean up
        conn.del::<&str, i32>("test").unwrap();
    }

    #[rstest]
    fn test_can_write_to_redis_concurrently(pool: Pool<redis::Client>) {
        let handles = (0..10)
            .map(|_| {
                let pool = pool.clone();
                spawn(move || {
                    let mut conn = pool.get().unwrap();
                    conn.set::<&str, &str, String>("test_1", "test_1").unwrap();

                    let result: String = conn.get("test_1").unwrap();
                    assert_eq!(result, "test_1");
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[rstest]
    fn test_single_writer_redis(pool: Pool<redis::Client>) {
        // Insert 5 items using a for loop
        let mut conn = pool.get().unwrap();
        conn.set::<&str, &str, String>("test_2", "test_2").unwrap();

        // Verify the count matches
        let result: String = conn.get("test_2").unwrap();
        assert_eq!(result, "test_2");

        // Clean up
        conn.del::<&str, i32>("test_2").unwrap();
    }

    #[rstest]
    fn test_write_and_read_redis(pool: Pool<redis::Client>) {
        // Create table if not exists
        let mut conn = pool.get().unwrap();

        for i in 0..5 {
            conn.set::<&str, &str, String>(&format!("test_{}", i + 3), &format!("test_{}", i + 3))
                .unwrap();
        }

        for i in 0..5 {
            let result: String = conn.get(format!("test_{}", i + 3)).unwrap();
            assert_eq!(result, format!("test_{}", i + 3));
        }

        // Clean up
        for i in 0..5 {
            assert!(conn.del::<&str, i32>(&format!("test_{}", i + 3)).unwrap() > 0);
        }
    }

    #[rstest]
    fn test_basic_connection_and_ping_redis(pool: Pool<redis::Client>) {
        let mut conn = pool.get().unwrap();
        // Basic ping command to verify connection
        let result = conn.req_command(Cmd::new().arg("PING")).unwrap();
        assert_eq!(result, Value::SimpleString(String::from("PONG")));
    }

    #[rstest]
    fn test_string_operations_redis(pool: Pool<redis::Client>) {
        // Establish a connection
        let mut conn = pool.get().unwrap();

        // Set a key-value pair
        let test_key = "test_key";
        let test_value = "Test Value";

        conn.set::<&str, &str, String>(test_key, test_value)
            .unwrap();

        // Retrieve and assert the value
        let retrieved: String = conn.get::<&str, String>(test_key).unwrap();
        assert_eq!(retrieved, test_value);

        // Clean up
        conn.del::<&str, i32>(test_key).unwrap();
    }

    #[rstest]
    fn test_list_operations_redis(pool: Pool<redis::Client>) {
        // Establish a connection
        let mut conn = pool.get().unwrap();

        // Test list operations
        let test_list = "test_list";

        // Push elements into the list
        conn.rpush::<&str, &str, i32>(test_list, "element1")
            .unwrap();
        conn.rpush::<&str, &str, i32>(test_list, "element2")
            .unwrap();
        conn.rpush::<&str, &str, i32>(test_list, "element3")
            .unwrap();

        // Retrieve all elements and assert count and values
        let elements: Vec<String> = conn.lrange(test_list, 0, -1).unwrap();

        // Remove the list
        for _ in 0..elements.len() {
            conn.rpop::<&str, Vec<String>>(test_list, Some(NonZero::new(1).unwrap()))
                .unwrap();
        }
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements,
            vec![
                "element1".to_string(),
                "element2".to_string(),
                "element3".to_string()
            ]
        );

        // Clean up
        conn.del::<&str, i32>(test_list).unwrap();
    }

    #[rstest]
    fn test_key_expiration_redis(pool: Pool<redis::Client>) {
        // Establish a connection
        let mut conn = pool.get().unwrap();

        // Test key expiration
        let test_key = "test_expired_key";

        // Set key with expiration in 1 second
        conn.set_ex::<&str, &str, String>(test_key, "Value", 1)
            .unwrap();

        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Try to get the key after expiration
        let retrieved: Option<String> = conn.get(test_key).unwrap();

        assert!(
            retrieved.is_none(),
            "Key should have expired and been removed"
        );
    }
}
