mod episode;
mod feed;
mod podcast;

pub async fn fetch_feed(url: &str) -> anyhow::Result<String> {
    Ok(String::from(url))
}

#[async_trait]
pub trait SearchProvider {
    async fn search_podcasts(&self, query: &str) -> anyhow::Result<Vec<PodcastSearchResult>>;
}
