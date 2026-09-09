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
    let looks_like_feed = xml.contains("<rss") || xml.contains("<feed");

    if !looks_like_feed {
        return Err(anyhow::anyhow!(
            "Input does not appear to be an RSS or Atom feed",
        ));
    }

    // let channel_start = xml
    //     .find("<channel")
    //     .map_or(0, |i| xml[i..].find('>').map_or(0, |j| i + j + 1));

    // let items_start = xml.find("<item>").unwrap_or(xml.len());

    // let channel_block = &xml[channel_start..items_start];

    // tracing::warn!("CHANNEL BLOCK: {channel_block:#?}");
    // let mut backup_feed: Rss;

    let normalized = normalize_feed_xml(xml);

    // let feed: Rss = serde_xml_rs::from_reader(&mut normalized.as_bytes())
    let feed: Rss = serde_xml_rs::SerdeXml::new()
        .overlapping_sequences(true)
        .from_reader(normalized.as_bytes())
        .map_err(|err| anyhow::anyhow!("Unable to parse XML: {err:#?}"))?;
    // Ok(val) => val,
    // Err(err) => {
    //     if err.to_string().contains("channel") {
    //         tracing::error!("No data was retrieved");
    //         return Err(anyhow::Error::msg("Malformed or no Data retrieved"));
    //     }

    //     // Duplicate <atom:link error workaround
    //     let mut new_xml = String::new();
    //     // Recreate the file without the duplicates
    //     for line in xml.lines() {
    //         if line.contains("<atom:link")
    //             || line.starts_with('x')
    //             || line.contains("media:thumbnail")
    //             || line.contains("podcast:meduim")
    //             || line.contains("media:content")
    //         {
    //         } else {
    //             new_xml.push_str(line);
    //         }
    //     }

    //     if !err.to_string().contains("atom:link") {
    //         tracing::error!("Error prior to retry: {err:#?}");
    //     }
    //     match serde_xml_rs::from_reader(&mut new_xml.as_bytes()) {
    //         Ok(data) => return Ok(data),
    //         Err(err) => {
    //             return Err(anyhow::Error::msg(format!(
    //                 "Unable to parse the XML: {err:#?}"
    //             )));
    //         }
    //     }
    // }
    // };

    Ok(feed)
}

fn normalize_feed_xml(xml: &str) -> String {
    const NAMESPACES: &[(&str, &str)] = &[
        ("media", "http://search.yahoo.com/mrss/"),
        ("atom", "http://www.w3.org/2005/Atom"),
        ("itunes", "http://www.itunes.com/dtds/podcast-1.0.dtd"),
        ("podcast", "https://podcastindex.org/namespace/1.0"),
    ];

    let mut normalized = xml.to_owned();

    for (prefix, uri) in NAMESPACES {
        let usage = format!("<{prefix}:");

        let declaration = format!("xmlns:{prefix}=");

        if normalized.contains(&usage) && !normalized.contains(&declaration) {
            normalized = add_namespace(normalized, prefix, uri);
        }
    }

    normalized
}

fn add_namespace(mut xml: String, prefix: &str, uri: &str) -> String {
    let Some(start) = xml.find("<rss") else {
        return xml;
    };

    let Some(relative_end) = xml[start..].find('>') else {
        return xml;
    };

    let end = start + relative_end;

    let declaration = format!(r#" xmlns:{prefix}="{uri}""#);

    xml.insert_str(end, &declaration);

    xml
}

#[async_trait]
pub trait SearchProvider {
    async fn search_podcasts(&self, query: &str) -> anyhow::Result<Vec<PodcastSearchResult>>;
}
