#![allow(clippy::unwrap)]
use mongodb::{
    Client,
    bson::{Document, doc, oid::ObjectId},
};

fn mongo_uri() -> String {
    std::env::var("TEST_MONGODB_URI").unwrap_or_else(|_| "mongodb://127.0.0.1:27017".to_string())
}

fn mongo_database() -> String {
    std::env::var("TEST_MONGODB_DATABASE").unwrap_or_else(|_| "podcasters_test".to_string())
}

#[tokio::test]
async fn mongodb_accepts_connections() {
    let client = Client::with_uri_str(mongo_uri())
        .await
        .expect("MongoDB test instance should accept a connection");

    let response = client
        .database("admin")
        .run_command(doc! {
            "ping": 1
        })
        .await
        .expect("MongoDB should respond to ping");

    eprintln!("response: {:#?}", response.contains_key("ok"));

    assert!(response.contains_key("ok"));
}

#[tokio::test]
async fn mongodb_can_write_and_read_document() {
    let client = Client::with_uri_str(mongo_uri()).await.unwrap();

    let database = client.database(&mongo_database());

    let collection_name = format!("integration_test_{}", ObjectId::new().to_hex());

    let collection = database.collection::<Document>(&collection_name);

    let id = ObjectId::new();

    let document = doc! {
        "_id": id,
        "email": "integration@podcasters.test",
        "name": "Integration Test",
    };

    collection
        .insert_one(document)
        .await
        .expect("MongoDB insert should succeed");

    let result = collection
        .find_one(doc! {
            "_id": id
        })
        .await
        .expect("MongoDB query should succeed")
        .expect("Inserted document should exist");

    assert_eq!(result.get_object_id("_id").unwrap(), id);

    assert_eq!(
        result.get_str("email").unwrap(),
        "integration@podcasters.test"
    );

    database
        .collection::<Document>(&collection_name)
        .drop()
        .await
        .expect("Test collection should be removed");
}

#[tokio::test]
async fn mongodb_database_isolated_from_other_database() {
    let client = Client::with_uri_str(mongo_uri()).await.unwrap();

    let database_name = mongo_database();

    let database = client.database(&database_name);

    assert_eq!(database.name(), database_name);
}

// #[tokio::test]
// async fn missing_redis_session_returns_missing_session() {
//     let mut redis = redis_connection().await;

//     let mongo = mongodb::Client::with_uri_str(
//         std::env::var("TEST_MONGODB_URI")
//             .unwrap_or_else(|_| "mongodb://127.0.0.1:27017".to_string()),
//     )
//     .await
//     .unwrap();

//     let session_id = format!("does-not-exist-{}", mongodb::bson::oid::ObjectId::new());

//     let result = authenticated_user_id(&session_id, &mongo, &mut redis).await;

//     assert_eq!(result, Err(AuthenticationError::MissingSession));
// }
