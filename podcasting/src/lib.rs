use async_trait::async_trait;
use episode::Rss;
use feed::PodcastSearchResult;
use tracing::instrument;

mod episode;
mod feed;
mod podcast;

/// # Errors
///
/// This function will error if the XML is malformed
#[instrument(name = "fetch the feed from URL", level = "debug")]
pub async fn fetch_feed(url: &str) -> anyhow::Result<Rss> {
    let xml = reqwest::get(url).await?.text().await?;
    // Get the first 25 characters of the feed to see if it is valid XML from the file 'feed.xml'

    // tracing::warn!("FEED: {}", xml.split_at(25).0);

    tracing::info!(
        "Count of the number of 'item'(s) in the feed: {}",
        xml.matches("<item").count()
    );

    // let channel_start = xml
    //     .find("<channel")
    //     .map_or(0, |i| xml[i..].find('>').map_or(0, |j| i + j + 1));

    // let items_start = xml.find("<item>").unwrap_or(xml.len());

    // let channel_block = &xml[channel_start..items_start];

    // tracing::warn!("CHANNEL BLOCK: {channel_block:#?}");

    let feed: Rss = match serde_xml_rs::from_reader(&mut xml.as_bytes()) {
        Ok(val) => val,
        Err(err) => {
            tracing::error!("Unable to parse the feed: {err:#?}");
            let xml_start: &str = xml.find("itunes:image").map_or("None", |i| &xml[i..]);

            tracing::info!("XML: {}", xml.chars().take(150).collect::<String>());
            tracing::info!(
                "XML start: {}",
                xml_start.chars().take(50).collect::<String>()
            );

            return Err(anyhow::Error::msg(format!(
                "Unable to parse the XML: {err:#?}"
            )));
        }
    };

    let var_name = &"None".to_string();
    let err_ret = String::from("None");
    tracing::warn!(
        "Podcast: {}",
        feed.channel.title.as_ref().unwrap_or(var_name)
    );
    feed.channel.item.iter().take(1).for_each(|entry| {
        tracing::warn!("title: {}", entry.title.as_ref().unwrap_or(&err_ret));
        tracing::info!(
            "Published: {:#?}",
            entry.pubDate.as_ref().unwrap_or(&err_ret)
        );
    });

    Ok(feed)
}

#[async_trait]
pub trait SearchProvider {
    async fn search_podcasts(&self, query: &str) -> anyhow::Result<Vec<PodcastSearchResult>>;
}
