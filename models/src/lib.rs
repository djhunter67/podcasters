#![allow(clippy::empty_enums)]
use std::time;

use mongodb::options::ClientOptions;
use redis::aio::{self, ConnectionManagerConfig};
use shared::settings;

pub mod billing;
pub mod bookmark;
pub mod database;
pub mod device;
pub mod episode;
pub mod indexes;
pub mod mongo_conf;
pub mod redis_conf;

struct _Users {}
struct _AuthSession {}
struct _Devices {}
struct _Podcasts {}
struct _Episodes {}
struct _Subscriptions {}
enum _PlaybackStates {}
struct _PlayList {}
struct _Bookmarks {}
struct _BillingCustomers {}
struct _Entitlements {}

/// # Errors
///
///   - Error if the cache layer or the database cannot be initialized
/// # Panics
///
///   - Panic if no connection are available for outside connections
pub async fn init_db()
-> Result<(aio::ConnectionManager, mongodb::Client), Box<dyn std::error::Error>> {
    let settings = match settings::get() {
        Ok(sets) => sets,
        Err(err) => {
            tracing::error!("Unable to acquire the settings to init the DB: {err:#?}");
            return Err(format!("Unable to acquire the settings: {err:#?}").into());
        }
    };

    let redis_client: redis::Client = match redis::Client::open(settings.redis.uri.clone()) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to connect to the cache layer: {err:#?}");
            panic!("Application cannot start: {err:#?}")
            // try to connect to a locally running instance of redis
        }
    };

    let redis_config = ConnectionManagerConfig::new()
        .set_connection_timeout(Some(time::Duration::from_secs(2))) // Time to establish TCP connection
        .set_response_timeout(Some(time::Duration::from_secs(1))) // Time to wait for command response
        .set_exponent_base(2.) // Exponential backoff base
        .set_number_of_retries(3); // Max retries before failing

    let redis_pool: redis::aio::ConnectionManager =
        match aio::ConnectionManager::new_with_config(redis_client, redis_config).await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::error!("Unable to connect to the cache layer: {err:#?}");
                panic!("Application cannot start: {err:#?}")
            }
        };

    let mongo_options: ClientOptions = match ClientOptions::parse(&settings.mongo.uri).await {
        Ok(mut conn) => {
            let mongo_settings = settings.mongo;

            conn.connect_timeout = Some(time::Duration::from_secs(
                mongo_settings.connection_timeout.into(),
            ));
            conn.server_selection_timeout = Some(time::Duration::from_secs(4));
            conn.app_name = Some(mongo_settings.db);
            conn
        }
        Err(err) => {
            tracing::error!("Unable to connect to the database: {err:#?}",);
            ClientOptions::parse("mongodb://localhost:27017")
                .await
                .expect("Unable to procure the database")
        }
    };

    let mongo_pool: mongodb::Client = match mongodb::Client::with_options(mongo_options) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to connect to the database: {err:#?}");
            panic!("Application cannot start: {err:#?}")
        }
    };

    Ok((redis_pool, mongo_pool))
}
