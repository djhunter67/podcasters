use async_trait::async_trait;
use feed::{PodcastFeed, PodcastSearchResult};
use serde::Deserialize;
use serde_xml_rs::from_str;
use tracing::instrument;

mod episode;
mod feed;
mod podcast;

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "rss")]
struct Rss {
    channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    title: String,
    #[serde(default)]
    item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    title: String,
    enclosure: Option<Enclosure>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Enclosure {
    #[serde(rename = "@url")]
    url: String,
    #[serde(rename = "@type")]
    r#type: String,
}

/// # Errors
///
/// This function will error if the XML is malformed
#[instrument(name = "fetch the feed from URL", level = "debug")]
pub async fn fetch_feed(url: &str) -> anyhow::Result<String> {
    let xml = reqwest::get(url).await?.text().await?;
    // let channel_start = xml
    //     .find("<channel")
    //     .map_or(0, |i| xml[i..].find('>').map_or(0, |j| i + j + 1));

    // let items_start = xml.find("<item>").unwrap_or(xml.len());

    // let channel_block = &xml[channel_start..items_start];

    // tracing::warn!("CHANNEL BLOCK: {channel_block:#?}");

    let feed: PodcastFeed = from_str(&xml)?;

    tracing::warn!("Title: {}", feed.get_title());
    for entry in feed.channel.item.into_iter().take(1) {
        tracing::warn!("title: {}", entry.title);
        tracing::info!("descript: {:#?}", entry.description);
        let encl = entry.enclosure.expect("No enclosure");
        tracing::info!("type: {}", encl.r#type);
        tracing::info!("URL: {}", encl.url);
    }

    Ok(String::from(url))
}

#[async_trait]
pub trait SearchProvider {
    async fn search_podcasts(&self, query: &str) -> anyhow::Result<Vec<PodcastSearchResult>>;
}
