#![allow(clippy::empty_enums)]
use std::{fmt, time};

use mongodb::options::ClientOptions;

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

pub enum DataBases {
    PodCast,
    User,
}

impl fmt::Display for DataBases {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PodCast => write!(f, "podcasts"),
            Self::User => write!(f, "users"),
        }
    }
}

/// # Errors
///
///   - Error if the cache layer or the database cannot be initialized
/// # Panics
///
///   - Panic if no connection are available for outside connections
pub async fn init_db() -> Result<mongodb::Client, Box<dyn std::error::Error>> {
    let settings = match settings::get() {
        Ok(sets) => sets,
        Err(err) => {
            tracing::error!("Unable to acquire the settings to init the DB: {err:#?}");
            return Err(format!("Unable to acquire the settings: {err:#?}").into());
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

    Ok(mongo_pool)
}
