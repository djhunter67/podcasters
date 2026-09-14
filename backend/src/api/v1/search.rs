use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::{HttpResponse, web};
use models::{DataBases, redis_conf::RedisKeys};
use mongodb::bson::{doc, oid::ObjectId};
use podcasting::episode::Podcast;
use redis::AsyncTypedCommands;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use shared::settings::Settings;
use tracing::instrument;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Searcher {
    #[serde(rename = "q")]
    podcast: String,
    limit: u16,
}

#[instrument(
    name = "Episode Getter",
    level = "info",
    target = "Podcasting",
    skip(mongo_client, redis_client, settings)
)]
#[actix_web::get("/search")]
pub async fn podcast_search(
    mongo_client: web::Data<mongodb::Client>,
    redis_client: web::Data<redis::aio::ConnectionManager>,
    settings: web::Data<Settings>,
    json: web::Query<Searcher>,
) -> HttpResponse {
    tracing::info!("querying for a podcast episode");

    let too_long_of_a_query: bool = json.podcast.len() > String::from("An excessively long podcast episode query requested name that will search the database to find.").len();
    if too_long_of_a_query {
        return HttpResponse::BadRequest().json(web::Json("Invalid search query"));
    }

    let result = search_podcasts(
        mongo_client,
        &mut redis_client.as_ref().clone(),
        settings,
        &json,
    )
    .await;

    tracing::warn!("The resulting podcast: {result:#?}");

    match result {
        Ok(res) => {
            tracing::info!("Search Result: {res:#?}");
            return HttpResponse::Accepted().json(web::Json(res));
        }
        Err(err) => {
            tracing::error!("{err}");
            return HttpResponse::NotFound().json(web::Json(err.to_string()));
        }
    };
}

pub async fn search_podcasts(
    mongo_client: web::Data<mongodb::Client>,
    redis_client: &mut redis::aio::ConnectionManager,
    settings: web::Data<Settings>,
    criteria: &Searcher,
) -> anyhow::Result<Podcast> {
    tracing::info!("The search name provided: {criteria:#?}");

    let cache_key = RedisKeys::new(&settings.redis.namespace).podcast(&settings.redis.namespace);

    // Search the cache layer (cache-hit)
    let _cache: Option<String> = redis_client.get(cache_key).await.map_err(|err| {
        tracing::error!("Failure to cache oid for Podcast: {err}");
        anyhow::anyhow!(err.to_string())
    })?;

    // Search the db (db-hit)
    let _conn = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Podcast>(&DataBases::PodCast.to_string());

    let _oid = ObjectId::from_str(&criteria.podcast)?;

    tracing::info!("made it past the db and cache layer instantiation");
    // Search the podcast index
    search_index(&criteria.podcast, criteria.limit).await?;

    Ok(Podcast::default())
}

const PODCAST_INDEX_SEARCH_URL: &str = "https://api.podcastindex.org/api/1.0/search/byterm";

async fn search_index(podcast: &str, limit: u16) -> anyhow::Result<()> {
    tracing::info!("Podcast to show: {podcast}");
    tracing::info!("How many to give: {limit}");
    let api_key = std::env::var("PODCAST_INDEX_API_KEY");
    tracing::warn!("API KEY: {api_key:#?}");

    let api_secret = std::env::var("PODCAST_INDEX_API_SECRET");
    tracing::warn!("The API secret: {api_secret:#?}");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .to_string();
    tracing::warn!("The current timestamp: {timestamp}");

    let auth_input = format!("{api_key:#?}{api_secret:#?}{timestamp:#?}");

    let mut hasher = Sha1::new();
    hasher.update(auth_input.as_bytes());

    let authorization = hex::encode(hasher.finalize());
    let _client = reqwest::Client::new();

    tracing::info!("Ready to GET: {authorization}");
    tracing::info!("Will search at: {PODCAST_INDEX_SEARCH_URL}");

    // let response = client
    //     .get(PODCAST_INDEX_SEARCH_URL)
    //     .query(&[("q", "rust"), ("max", "2")])
    //     .header(reqwest::header::USER_AGENT, "Podcasters/0.1")
    //     .header("X-Auth-Key", &api_key)
    //     .header("X-Auth-Date", &timestamp)
    //     .header("Authorization", &authorization)
    //     .send()
    //     .await?
    //     .error_for_status()?;

    Ok(())
}
