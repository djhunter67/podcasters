#![allow(clippy::unwrap_used)]

use models::redis_conf::{UserSession, authenticated_user_id};
use redis::{AsyncCommands, aio::ConnectionManager};

fn redis_uri() -> String {
    std::env::var("TEST_REDIS_URI").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string())
}

async fn redis_connection() -> ConnectionManager {
    let client = redis::Client::open(redis_uri()).expect("Redis URI should be valid");

    ConnectionManager::new(client)
        .await
        .expect("Redis test instance should accept a connection")
}

#[tokio::test]
async fn redis_accepts_connections() {
    let mut connection = redis_connection().await;

    let response: String = redis::cmd("PING")
        .query_async(&mut connection)
        .await
        .expect("Redis should respond to PING");

    assert_eq!(response, "PONG");
}

#[tokio::test]
async fn redis_can_store_and_retrieve_value() {
    let mut connection = redis_connection().await;

    let key = format!("integration:test:{}", mongodb::bson::oid::ObjectId::new());

    let _: () = connection
        .set(&key, "podcasters")
        .await
        .expect("Redis SET should succeed");

    let value: String = connection
        .get(&key)
        .await
        .expect("Redis GET should succeed");

    assert_eq!(value, "podcasters");

    let _: () = connection
        .del(&key)
        .await
        .expect("Redis cleanup should succeed");
}

#[tokio::test]
async fn redis_missing_key_returns_none() {
    let mut connection = redis_connection().await;

    let key = format!(
        "integration:missing:{}",
        mongodb::bson::oid::ObjectId::new()
    );

    let value: Option<String> = connection.get(key).await.expect("Redis GET should succeed");

    assert_eq!(value, None);
}

#[tokio::test]
async fn authenticated_user_id_returns_cached_object_id() {
    let mut redis = redis_connection().await;

    let mongo = mongodb::Client::with_uri_str(
        std::env::var("TEST_MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://127.0.0.1:27017".to_string()),
    )
    .await
    .unwrap();

    let session_id = format!("integration-{}", mongodb::bson::oid::ObjectId::new());

    let email = format!("{}@podcasters.test", mongodb::bson::oid::ObjectId::new());

    let object_id = mongodb::bson::oid::ObjectId::new();

    let session_key = format!("session:{session_id}");

    let auth_key = format!("user:auth:{email}");

    let cached_session = serde_json::to_string(&UserSession {
        oid: object_id.to_hex(),
    })
    .unwrap();

    let _: () = redis.set(&session_key, &email).await.unwrap();

    let _: () = redis.set(&auth_key, cached_session).await.unwrap();

    let result = authenticated_user_id(&session_id, &mongo, &mut redis)
        .await
        .expect("Cached authentication should succeed");

    assert_eq!(result, object_id);

    let _: () = redis.del(&session_key).await.unwrap();

    let _: () = redis.del(&auth_key).await.unwrap();
}

use models::redis_conf::AuthenticationError;

#[tokio::test]
async fn invalid_cached_object_id_returns_invalid_session() {
    let mut redis = redis_connection().await;

    let mongo = mongodb::Client::with_uri_str(
        std::env::var("TEST_MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://127.0.0.1:27017".to_string()),
    )
    .await
    .unwrap();

    let session_id = format!(
        "integration-invalid-{}",
        mongodb::bson::oid::ObjectId::new()
    );

    let email = "invalid@podcasters.test";

    let session_key = format!("session:{session_id}");

    let auth_key = format!("user:auth:{email}");

    let _: () = redis.set(&session_key, email).await.unwrap();

    let _: () = redis
        .set(&auth_key, r#"{"oid":"not-an-object-id"}"#)
        .await
        .unwrap();

    let result = authenticated_user_id(&session_id, &mongo, &mut redis).await;

    assert_eq!(result, Err(AuthenticationError::InvalidSession));

    let _: () = redis.del(&session_key).await.unwrap();

    let _: () = redis.del(&auth_key).await.unwrap();
}

#[tokio::test]
async fn missing_redis_session_returns_missing_session() {
    let mut redis = redis_connection().await;

    let mongo = mongodb::Client::with_uri_str(
        std::env::var("TEST_MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://127.0.0.1:27017".to_string()),
    )
    .await
    .unwrap();

    let session_id = format!("does-not-exist-{}", mongodb::bson::oid::ObjectId::new());

    let result = authenticated_user_id(&session_id, &mongo, &mut redis).await;

    assert_eq!(result, Err(AuthenticationError::MissingSession));
}
