use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::{HttpResponse, web};
use mongodb::bson::doc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tracing::instrument;

#[derive(Debug)]
enum SearchError {
    NotFound,
    ResponseError(String),
    PodCastIndexError(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Not Found"),
            Self::ResponseError(val) => write!(f, "{val}"),
            Self::PodCastIndexError(val) => write!(f, "{val}"),
        }
    }
}

trait SearchProvider {
    async fn search_podcasts(
        &self,
        query: &str,
        limit: u16,
    ) -> Result<Vec<PodcastSearchResult>, SearchError>;
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct SearchQuery {
    #[serde(rename = "q")]
    podcast: String,
    limit: u16,
}

pub struct PodcastIndexProvider {
    client: reqwest::Client,
    api_key: String,
    api_secret: String,
}

impl PodcastIndexProvider {
    pub fn new(client: reqwest::Client, api_key: String, api_secret: String) -> Self {
        Self {
            client,
            api_key,
            api_secret,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
pub enum SearchProviderKind {
    PodcastIndex,
    LocalCatalog,
}

impl SearchProvider for PodcastIndexProvider {
    async fn search_podcasts(
        &self,
        query: &str,
        limit: u16,
    ) -> anyhow::Result<Vec<PodcastSearchResult>, SearchError> {
        tracing::info!("Podcast to show: {query}");
        tracing::info!("How many to give: {limit}");
        // let api_key = std::env::var("PODCAST_INDEX_API_KEY")?;
        tracing::warn!("API KEY: {:#?}", self.api_key);

        // let api_secret = std::env::var("PODCAST_INDEX_API_SECRET");
        tracing::warn!("The API secret: {:#?}", self.api_secret);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| SearchError::PodCastIndexError(err.to_string()))?
            .as_secs()
            .to_string();

        tracing::warn!("The current timestamp: {timestamp}");

        let auth_input = format!("{}{}{timestamp}", self.api_key, self.api_secret);

        let mut hasher = Sha1::new();
        hasher.update(auth_input.as_bytes());

        let authorization: String = hex::encode(hasher.finalize());

        // tracing::info!("Encoded Hash: {:#?}", hasher.finalize());
        tracing::info!("Auth key: {}", authorization);
        tracing::info!("Will search at: {PODCAST_INDEX_SEARCH_URL}");

        let response = match self
            .client
            .get(PODCAST_INDEX_SEARCH_URL)
            .query(&[("q", query), ("max", &format!("{limit}"))])
            .header(reqwest::header::USER_AGENT, "Podcasters/0.1")
            .header("X-Auth-Key", &self.api_key)
            .header("X-Auth-Date", &timestamp)
            .header("Authorization", &authorization)
            .send()
            .await
        {
            Ok(data) => {
                tracing::warn!("Received the data: {}", data.status());
                data
            }
            Err(err) => {
                tracing::error!("Unable to query: {err:#?}");
                return Err(SearchError::NotFound);
            }
        };
        // .error_for_status()?;

        tracing::warn!("Response: {:#?}", response.status());

        let body: String = response
            .text()
            .await
            .map_err(|err| SearchError::ResponseError(err.to_string()))?;

        // let json_result: serde_json::Value = serde_json::from_str(&body).expect("No values");
        // tracing::warn!("Podcast Index result: {json_result:#?}");

        // tracing::warn!("Body: {:#?}", body);
        let mut result: PodcastSearchResult = serde_json::from_str(&body)
            .map_err(|err| SearchError::ResponseError(err.to_string()))?;
        result.provider = Some(SearchProviderKind::PodcastIndex);

        Ok(vec![result])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastSearchResult {
    pub provider: Option<SearchProviderKind>,
    status: String,
    feeds: Vec<Feed>,
    count: u16,
    query: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    #[serde(rename = "id")]
    episode_id: usize,
    title: String,
    url: String,
    #[serde(rename = "originalUrl")]
    origonal_url: String,
    link: String,
    description: String,
    author: String,
    #[serde(rename = "ownerName")]
    owner_name: String,
    image: String,
    artwork: String,
    #[serde(rename = "lastUpdateTime")]
    last_update: usize,
    #[serde(rename = "lastCrawlTime")]
    last_crawl_time: usize,
    #[serde(rename = "lastParseTime")]
    last_parse_time: usize,
    #[serde(rename = "lastGoodHttpStatusTime")]
    last_good_http_time: usize,
    #[serde(rename = "lastHttpStatus")]
    last_http_status: u16,
    language: String,
    explicit: bool,
    #[serde(rename = "podcastGuid")]
    guid: String,
    #[serde(rename = "episodeCount")]
    episode_count: u16,
}
#[instrument(
    name = "Episode Getter",
    level = "info",
    target = "Podcasting",
    skip(json)
)]
#[actix_web::get("/search")]
pub async fn podcast_search(json: web::Query<SearchQuery>) -> HttpResponse {
    tracing::info!("querying for a podcast episode");

    let too_long_of_a_query: bool = json.podcast.len() > String::from("An excessively long podcast episode query requested name that will search the database to find.").len();
    if too_long_of_a_query {
        return HttpResponse::BadRequest().json(web::Json("Invalid search query"));
    }

    let client: reqwest::Client = Client::new();
    let api_key = match std::env::var("PODCAST_INDEX_API_KEY") {
        Ok(val) => val,
        Err(err) => {
            tracing::error!("Unable to find the PODCAST_INDEX API KEY: {err:#?}");
            return HttpResponse::InternalServerError().json(web::Json(err.to_string()));
        }
    };
    tracing::warn!("API KEY: {api_key:#?}");

    let api_secret = match std::env::var("PODCAST_INDEX_API_SECRET") {
        Ok(val) => val,
        Err(err) => {
            tracing::error!("Unable to find the PODCAST_INDEX API KEY: {err:#?}");
            return HttpResponse::InternalServerError().json(web::Json(err.to_string()));
        }
    };
    tracing::warn!("The API secret: {api_secret:#?}");

    let searcher = PodcastIndexProvider::new(client, api_key, api_secret);

    let result = searcher.search_podcasts(&json.podcast, json.limit).await;

    // tracing::warn!("The resulting podcast: {result:#?}");

    match result {
        Ok(res) => {
            tracing::info!("Search Result: {res:#?}");
            return HttpResponse::Accepted().json(web::Json(res));
        }
        Err(err) => {
            tracing::error!("{err:#?}");
            return HttpResponse::NotFound().json(web::Json(err.to_string()));
        }
    };

    // HttpResponse::Ok().json(web::Json(result))
}

const PODCAST_INDEX_SEARCH_URL: &str = "https://api.podcastindex.org/api/1.0/search/byterm";
