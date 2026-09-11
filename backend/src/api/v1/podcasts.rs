use std::str::FromStr;

use actix_web::{HttpResponse, web};
use models::DataBases;
use mongodb::{
    IndexModel,
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
};
use podcasting::episode::Podcast;
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Deserialize, Serialize)]
struct PreviewQuery {
    uri: String,
    id: Option<mongodb::bson::oid::ObjectId>,
}

#[actix_web::post("/preview")]
#[instrument(name = "Preview Endpoint", level = "debug", target = "Podcasting")]
pub async fn preview(uri: web::Json<PreviewQuery>) -> HttpResponse {
    tracing::info!("Getting the preview for: {}", uri.uri);
    match Podcast::new(&uri.uri).await {
        Ok(data) => HttpResponse::Ok().json(web::Json(data)),
        Err(err) => {
            tracing::error!("Unable to generate the preview: {err:#?}");
            HttpResponse::InternalServerError().json(web::Json(err.to_string().trim_matches('\"')))
        }
    }
}

#[allow(clippy::similar_names)]
#[actix_web::post("/podcast")]
#[instrument(
    name = "Save a podcast to the DB",
    level = "info",
    target = "Podcasting",
    skip(uri, mongo_client, redis_client)
)]
pub async fn set_podcast(
    mut uri: web::Json<PreviewQuery>,
    mongo_client: web::Data<mongodb::Client>,
    redis_client: web::Data<redis::aio::ConnectionManager>,
) -> HttpResponse {
    tracing::info!("Save a podcast at uri: {}", uri.uri);

    let cache_key: String = format!("podcast:{}", uri.uri);

    // Check the cache layer for the uri
    uri = cache_check(&mut redis_client.as_ref().clone(), &cache_key, uri).await;

    if let Some(_id) = uri.id {
        tracing::warn!("CACHE HIT!");
        // No GUARANTEE THAT THIS KEY EXISTS IN THE DB
        return HttpResponse::Ok().json(web::Json(uri));
    }
    tracing::warn!("Cache layer MISS");

    uri = db_check(mongo_client.as_ref().clone(), uri).await;

    if let Some(_id) = uri.id {
        tracing::warn!("DB CACHE HIT!");
        let _ = redis_client
            .as_ref()
            .clone()
            .set_ex(
                &cache_key,
                uri.id.expect("Fail to retrieve OID").to_string(),
                86_400,
            )
            .await
            .map_err(|err| {
                tracing::error!("Failure to cache oid for Podcast: {err}");
                HttpResponse::InternalServerError().json(web::Json(err.to_string()))
            });

        return HttpResponse::Ok().json(web::Json(uri));
    }

    tracing::warn!("DB layer MISS");

    // Never before seen URI
    let xml = reqwest::get(&uri.uri)
        .await
        .expect("Fail to GET")
        .text()
        .await
        .expect("Fail to get XML from the web");

    let mut pod = match Podcast::new(&xml).await {
        Ok(data) => data,
        Err(err) => {
            tracing::error!("Unable to generate the preview: {err:#?}");
            return HttpResponse::NotFound().json(web::Json("Invalid URI"));
        }
    };

    pod.set_uri(&uri.0.uri);

    // Save the podcast to the database
    let conn = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Podcast>(&DataBases::PodCast.to_string());

    // Ensure unique Documents based on the uri
    let index = IndexModel::builder()
        .keys(doc! {"uri": 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let _created_index = conn.create_index(index).await;

    match conn.insert_one(&pod).await {
        // A new URI passed in
        Ok(id) => {
            tracing::info!("Saved the podcast to the DB");
            let oid = id.inserted_id.as_object_id();
            tracing::warn!("The OID saved");
            let _ = pod.set_id(oid);
            uri.id = oid;
            let _ = redis_client
                .as_ref()
                .clone()
                .set_ex(
                    &cache_key,
                    oid.expect("Fail to retrieve OID").to_string(),
                    86_400,
                )
                .await
                .map_err(|err| {
                    tracing::error!("Failure to cache oid for Podcast: {err}");
                    HttpResponse::InternalServerError().json(web::Json(err.to_string()))
                });
            tracing::info!("Cache layer updated");
        }
        Err(err) => {
            tracing::error!("Unable to insert the podcast in the databse: {err:#?}");
            return HttpResponse::AlreadyReported().json(web::Json(uri));
        }
    };

    HttpResponse::Ok().json(web::Json(uri))
}

#[derive(Debug, Deserialize)]
struct Getter {
    #[serde(rename = "id")]
    podcast_id: String,
    #[serde(default)]
    limit: u16,
}

#[instrument(
    name = "Podcast getter",
    level = "info",
    target = "Podcasting",
    skip(mongo_client)
)]
#[actix_web::get("/podcast")]
pub async fn get_podcast(
    mongo_client: web::Data<mongodb::Client>,
    json: web::Query<Getter>,
) -> HttpResponse {
    tracing::info!("querying for a podcast");
    let conn = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Podcast>(&DataBases::PodCast.to_string());

    let oid = match ObjectId::from_str(&json.podcast_id) {
        Ok(val) => val,
        Err(err) => {
            tracing::error!("Unable to convert ID to ObjectID: {err:#?}");
            return HttpResponse::BadRequest().json(web::Json("Invalid ID passed"));
        }
    };

    match conn
        .find_one(doc! {
            "_id": oid
        })
        .await
    {
        Ok(find_one) => {
            tracing::info!("query performed");
            if let Some(pod_cast) = find_one {
                if !json.limit.eq(&0) {
                    tracing::info!("Limiting the number of episodes: {}", json.limit);
                    // pod_cast
                }
                tracing::warn!("Information found");
                return HttpResponse::Ok().json(web::Json(pod_cast));
            } else {
                tracing::warn!("Podcast not found");
                return HttpResponse::NotFound().json(web::Json("Not found"));
            }
        }
        Err(err) => {
            tracing::error!("Failed to query the DB: {err:#?}");
            return HttpResponse::InternalServerError().json(err.to_string());
        }
    }
}

#[instrument(
    name = "Episode Getter",
    level = "info",
    target = "Get an Episode",
    skip(mongo_client)
)]
#[actix_web::get("/episode/{id}")]
pub async fn get_episode(
    mongo_client: web::Data<mongodb::Client>,
    json: web::Path<Getter>,
) -> HttpResponse {
    tracing::info!("querying for a podcast");
    let conn = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Podcast>(&DataBases::PodCast.to_string());

    let oid = match ObjectId::from_str(&json.podcast_id) {
        Ok(val) => val,
        Err(err) => {
            tracing::error!("Unable to convert ID to ObjectID: {err:#?}");
            return HttpResponse::BadRequest().json(web::Json("Invalid ID passed"));
        }
    };

    match conn
        .find_one(doc! {
            "_id": oid
        })
        .await
    {
        Ok(find_one) => {
            tracing::info!("query performed");
            if let Some(pod_cast) = find_one {
                if !json.limit.eq(&0) {
                    tracing::info!("will limiting the number of episodes: {}", json.limit);
                }
                tracing::warn!("Information found");
                return HttpResponse::Ok().json(web::Json(pod_cast));
            } else {
                tracing::warn!("Podcast not found");
                return HttpResponse::NotFound().json(web::Json("Not found"));
            }
        }
        Err(err) => {
            tracing::error!("Failed to query the DB: {err:#?}");
            return HttpResponse::InternalServerError().json(err.to_string());
        }
    }
}

#[instrument(
    name = "Cache check",
    level = "debug",
    target = "Save a podcast",
    skip(redis_client, cache_key, uri)
)]
async fn cache_check(
    redis_client: &mut redis::aio::ConnectionManager,
    cache_key: &str,
    mut uri: web::Json<PreviewQuery>,
) -> web::Json<PreviewQuery> {
    tracing::debug!("cache check");
    let cache: Option<String> = redis_client
        .get(cache_key)
        .await
        .map_err(|err| {
            tracing::error!("Failure to cache oid for Podcast: {err}");
            err.to_string()
        })
        .expect("Failure to get cached value");

    if let Some(found) = cache {
        uri.id = ObjectId::from_str(&found).ok();
    }

    uri
}

#[instrument(
    name = "DB check",
    level = "debug",
    target = "Save a podcast",
    skip(mongo_client, uri)
)]
async fn db_check(
    mongo_client: mongodb::Client,
    mut uri: web::Json<PreviewQuery>,
) -> web::Json<PreviewQuery> {
    tracing::debug!("db cache check");
    let conn = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Podcast>(&DataBases::PodCast.to_string());

    // Check if the uri has been previously submitted
    if let Ok(cache_podcast) = conn
        .find_one(doc! {
        "uri": &uri.uri
        })
        .await
        && let Some(pod) = cache_podcast
    {
        uri.id = pod.get_id();
    }
    uri
}
