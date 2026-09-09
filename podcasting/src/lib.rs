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
pub async fn fetch_feed(xml: &str) -> anyhow::Result<Rss> {
    if xml.matches("<item").count().eq(&0) {
        return Err(anyhow::Error::msg(
            "No XML data found or XML data is malformed",
        ));
    }

    // let channel_start = xml
    //     .find("<channel")
    //     .map_or(0, |i| xml[i..].find('>').map_or(0, |j| i + j + 1));

    // let items_start = xml.find("<item>").unwrap_or(xml.len());

    // let channel_block = &xml[channel_start..items_start];

    // tracing::warn!("CHANNEL BLOCK: {channel_block:#?}");
    // let mut backup_feed: Rss;

    let feed: Rss = match serde_xml_rs::from_reader(&mut xml.as_bytes()) {
        Ok(val) => val,
        Err(err) => {
            // tracing::error!("Unable to parse the feed: {err:#?}");
            if err.to_string().contains("channel") {
                tracing::error!("No data was retrieved");
                return Err(anyhow::Error::msg("Malformed or no Data retrieved"));
            }
            // if err.to_string().contains("atom:link") {
            //     // Move the pointer
            //     let xml_start: &str = xml.rfind("<atom:link").map_or("None", |i| &xml[(i - 1)..]);

            //     // tracing::info!("XML: {}", xml.chars().take(150).collect::<String>());
            //     tracing::info!(
            //         "XML start: {}",
            //         xml_start.chars().take(180).collect::<String>()
            //     );
            //     match serde_xml_rs::from_reader(&mut xml_start.as_bytes()) {
            //         Ok(fed) => return Ok(fed),
            //         Err(err) => {
            //             tracing::info!("Second Pass: {err:#?}");
            //             return Err(anyhow::Error::msg(format!("{err}")));
            //         }
            //     }
            // }
            // modify the file to remove the malformed data
            let mut new_xml = String::new();
            // let mut offending_line = String::new();
            // let mut removed_count = 0;
            for line in xml.lines() {
                if line.contains("<atom:link") {
                    // Remove the line from the file
                    // tracing::warn!("The offending line has been found: {line}");
                    // offending_line = line.trim().to_string();
                    // removed_count += offending_line.len();
                    // removed_count += 1;
                    // tracing::info!(
                    //     "REMOVED COUNT: {removed_count}; Val: {}",
                    //     line[0..15].to_string().trim()
                    // );
                } else {
                    new_xml.push_str(line);
                }
            }

            // tracing::info!("new XML len: {}", new_xml.len());
            // tracing::info!("XML len: {}", xml.len());
            // let amt_removed = xml.len() - new_xml.len();
            // let char_count = offending_line.chars().count();
            // let count_of_dup_field = new_xml.matches("<atom:link").count();

            // tracing::info!("equal?: diff count: {amt_removed}");
            // tracing::info!("equal?: char count: {char_count}");
            // tracing::info!("Count of removed chars: {removed_count}");
            // tracing::info!("Count of duplicate 'atom:link': {}", count_of_dup_field);
            // tracing::warn!("Offending Line: {offending_line}");

            match serde_xml_rs::from_reader(&mut new_xml.as_bytes()) {
                Ok(data) => return Ok(data),
                Err(err) => {
                    tracing::error!("Different Error: {err:#?}");
                    return Err(anyhow::Error::msg(format!(
                        "Unable to parse the XML: {err:#?}"
                    )));
                } // tracing::info!("Second attempt succesful");
            }
        }
    };

    // let var_name = &"None".to_string();
    // let err_ret = String::from("None");
    // tracing::warn!(
    //     "Podcast: {}",
    //     feed.channel.title.as_ref().unwrap_or(var_name)
    // );
    // feed.channel.item.iter().take(1).for_each(|entry| {
    //     tracing::warn!("title: {}", entry.title.as_ref().unwrap_or(&err_ret));
    //     tracing::info!(
    //         "Published: {:#?}",
    //         entry.pubDate.as_ref().unwrap_or(&err_ret)
    //     );
    // });

    Ok(feed)
}

#[async_trait]
pub trait SearchProvider {
    async fn search_podcasts(&self, query: &str) -> anyhow::Result<Vec<PodcastSearchResult>>;
}
