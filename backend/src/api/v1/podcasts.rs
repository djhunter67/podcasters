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
use shared::settings;
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
            HttpResponse::InternalServerError().json(web::Json(Podcast::error(
                err.to_string().trim_matches('\"'),
            )))
        }
    }
}

#[allow(clippy::similar_names)]
#[actix_web::post("/podcast")]
#[instrument(
    name = "Save a podcast to the DB",
    level = "debug",
    target = "Podcasting",
    // skip(uri, redis_client)
)]
pub async fn podcast(
    mut uri: web::Json<PreviewQuery>,
    mongo_client: web::Data<mongodb::Client>,
    // redis_client: web::Data<&mut redis::aio::ConnectionManager>,
) -> HttpResponse {
    tracing::info!("Save a podcast at uri: {}", uri.uri);

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
            return HttpResponse::InternalServerError().json(web::Json(Podcast::error(
                err.to_string().trim_matches('\"'),
            )));
        }
    };

    pod.set_uri(&uri.0.uri);

    let cache_key = format!("podcast:{}", pod.get_title());

    uri.0.uri = pod.get_title();

    // Save the podcast to the database
    let conn = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Podcast>(&DataBases::PodCast.to_string());

    // Ensure unique Documents based on the title
    let index = IndexModel::builder()
        .keys(doc! {"uri": 1})
        .options(IndexOptions::builder().unique(true).build())
        .build();
    let created_index = conn.create_index(index).await;

    tracing::info!("Tracing info: {created_index:#?}");

    let mut redis_client =
        redis::Client::open(settings::get().expect("Fail to get settings").redis.uri)
            .expect("Broken cache layer")
            .get_multiplexed_async_connection()
            .await
            .expect("Fail to connect to cache");

    match conn.insert_one(&pod).await {
        Ok(id) => {
            tracing::info!("Saved the podcast to the DB");
            let oid = id.inserted_id.as_object_id();
            tracing::warn!("The save OID: {oid:#?}");
            let _ = pod.set_id(oid);
            uri.id = oid;
            let _ = redis_client
                // .as_ref()
                // .clone()
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
            tracing::info!("Cache save: {oid:#?}");
        }
        Err(err) if is_dup(&err) => {
            // Alredy inserted
            tracing::info!("Value already found in DB");
            let cache: Option<String> = redis_client
                .get(&cache_key)
                .await
                .map_err(|err| {
                    tracing::error!("Failure to cache oid for Podcast: {err}");
                    err.to_string()
                })
                .expect("Failure to get cacched value");
            tracing::warn!("cache key: {cache:#?}");

            match cache {
                Some(oid) => {
                    tracing::warn!("Cache-Hit!");
                    tracing::debug!("Returning an OID {oid:#?}");
                    let _ = pod.set_id(ObjectId::from_str(&oid).ok());
                    uri.id = ObjectId::from_str(&oid).ok();
                    return HttpResponse::Ok().json(web::Json(uri));
                }
                None => {
                    tracing::warn!("Cache Miss!");
                    let existing: Option<Podcast> = match mongo_client
                        .database(&DataBases::PodCast.to_string())
                        .collection::<Podcast>(&DataBases::PodCast.to_string())
                        .find_one(doc! {
                            "title": pod.get_title(),
                        })
                        .await
                    {
                        Ok(res) => {
                            tracing::warn!("Found item in the DB");
                            res
                        }
                        Err(err) => {
                            tracing::error!("Cache error: {err:#?}");
                            return HttpResponse::InternalServerError()
                                .json(web::Json(err.to_string()));
                        }
                    };

                    let oid = match existing {
                        Some(data) => {
                            tracing::debug!("Existing Podcast: {}", data.get_title());
                            data.get_id().expect("Failed to get IOD")
                        }
                        None => {
                            tracing::error!("Failed to get existing Podcast");
                            return HttpResponse::InternalServerError()
                                .json(web::Json(err.to_string()));
                        }
                    };

                    let _: () = redis_client
                        .set_ex(cache_key, oid.to_string(), 86_400)
                        .await
                        .map_err(|err| err.to_string())
                        .expect("Failure to reset cache");

                    uri.id = Some(oid);

                    return HttpResponse::Ok().json(web::Json(uri));
                }
            }
        }
        Err(err) => {
            tracing::error!("Unable to insert the podcast in the databse: {err:#?}");
            return HttpResponse::AlreadyReported().json(web::Json(uri));
        }
    };

    HttpResponse::Ok().json(web::Json(uri))
}

fn is_dup(err: &mongodb::error::Error) -> bool {
    matches!(
        *err.kind,
        mongodb::error::ErrorKind::Write(mongodb::error::WriteFailure::WriteError(
            mongodb::error::WriteError { code: 11000, .. }
        ))
    )
}
