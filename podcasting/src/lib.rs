use async_trait::async_trait;
use episode::Rss;
use feed::{PodcastFeed, PodcastSearchResult};
use serde_xml_rs::from_str;
use tracing::instrument;

mod episode;
mod feed;
mod podcast;

/// # Errors
///
/// This function will error if the XML is malformed
#[instrument(name = "fetch the feed from URL", level = "debug")]
pub async fn fetch_feed(url: &str) -> anyhow::Result<String> {
    // let mut xml = reqwest::get(url).await?.text().await?;
    // Get the first 25 characters of the feed to see if it is valid XML from the file 'feed.xml'
    let xml = include_str!("../feed.xml");

    tracing::warn!("FEED: {}", xml.split_at(650).0);

    tracing::info!(
        "Count of the word 'channel' in the feed: {}",
        xml.matches("<rss").count()
    );

    // let channel_start = xml
    //     .find("<channel")
    //     .map_or(0, |i| xml[i..].find('>').map_or(0, |j| i + j + 1));

    // let items_start = xml.find("<item>").unwrap_or(xml.len());

    // let channel_block = &xml[channel_start..items_start];

    // tracing::warn!("CHANNEL BLOCK: {channel_block:#?}");

    // let feed: Rss = from_str(&xml)?;

    // tracing::warn!("Title: {}", feed.get_title());
    // for entry in feed.get_episode().into_iter().take(1) {
    //     tracing::warn!("title: {}", entry.get_title());
    //     tracing::info!("descript: {:#?}", entry.get_description());
    //     // tracing::info!("type: {}", encl.r#type);
    //     // tracing::info!("URL: {}", encl.url);
    // }

    Ok(String::from(url))
}

#[async_trait]
pub trait SearchProvider {
    async fn search_podcasts(&self, query: &str) -> anyhow::Result<Vec<PodcastSearchResult>>;
}
