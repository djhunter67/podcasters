use std::fs;

use futures::future::join_all;
use tokio::{io::AsyncReadExt, task::JoinHandle};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(filter)
        .init();

    // get_xml_data().await?;

    let xml_list: Vec<String> = vec![
        String::from("https://feeds.megaphone.fm/hubermanlab"),
        String::from("https://api.substack.com/feed/podcast/1136839.rss"),
        String::from("https://lexfridman.com/feed/podcast/"),
        String::from("https://coder.show/rss"),
        String::from("https://feeds.npr.org/510289/podcast.xml"),
        String::from("https://letscast.fm/podcasts/rust-in-production-82281512/feed"),
        String::from("http://feeds.libsyn.com/110110/rss"),
        String::from("https://anchor.fm/s/f39e007c/podcast/rss"),
        String::from("https://feeds.transistor.fm/fork-around-and-find-out"),
        String::from("https://feeds.feedburner.com/dancarlin/history?format=xml"),
        String::from("http://feeds.wnyc.org/radiolab"),
        String::from("https://feeds.megaphone.fm/replyall"),
        String::from("https://feeds.simplecast.com/gvtxUiIf"),
        String::from("https://feeds.megaphone.fm/vergecast"),
        String::from("https://feeds.megaphone.fm/revisionisthistory"),
        String::from("https://feeds.npr.org/510308/podcast.xml"),
        String::from("https://feeds.simplecast.com/dLRotFGk"),
        String::from("https://thewomenintechshow.com/category/podcast/feed/"),
        String::from("https://feeds.megaphone.fm/GLT1412515089"),
        String::from("https://feeds.simplecast.com/qm_9xx0g"),
        String::from("http://rss.art19.com/the-daily"),
        String::from("https://feeds.simplecast.com/mKn_QmLS"),
        String::from("https://feeds.megaphone.fm/thispastweekend"),
        String::from("https://podcastfeeds.nbcnews.com/dateline-nbc"),
        String::from("https://www.thisamericanlife.org/podcast/rss.xml"),
        String::from("https://feeds.simplecast.com/6Qp23t6h"),
        String::from("https://rss.art19.com/smartless"),
        String::from("https://feeds.megaphone.fm/newheights"),
    ];

    parse_xml(&Devel::Xml, xml_list).await;

    Ok(())
}

enum Devel {
    Url,
    Xml,
}

async fn parse_xml(path_forward: &Devel, xml_list: Vec<String>) {
    match path_forward {
        Devel::Xml => {
            let res: Vec<JoinHandle<()>> = (0..27)
                .map(|i| {
                    tokio::spawn(async move {
                        let mut file = tokio::fs::File::open(format!("pod_feed_{i}.xml"))
                            .await
                            .expect("Fail to open file");
                        let mut xml: Vec<u8> = Vec::new();

                        file.read_to_end(&mut xml).await.expect("Fail to read file");

                        // convert the Vec<u8> to a String
                        let xml = String::from_utf8(xml).expect("Fail to convert to String");

                        match podcasting::fetch_feed(&xml).await {
                            Ok(feed) => {
                                tracing::info!("pod_feed_{i}.xml");
                                tracing::warn!(
                                    "Podcast: {}",
                                    feed.channel.title.unwrap_or_else(|| "None".to_string())
                                );
                                feed.channel.item.iter().take(1).for_each(|entry| {
                                    tracing::warn!(
                                        "title: {}",
                                        entry.title.as_ref().map_or("None", |v| v)
                                    );
                                    tracing::info!(
                                        "Published: {:#?}",
                                        entry.pubDate.as_ref().map_or("None", |v| v)
                                    );
                                });
                            }
                            Err(err) => {
                                tracing::info!("pod_feed_{i}.xml -> Error");
                                tracing::error!("Error: {err:#?}");
                            }
                        }
                    })
                })
                .collect();

            join_all(res).await;
        }
        Devel::Url => {
            tracing::info!("Using XML mode");
            let res: Vec<JoinHandle<()>> = xml_list
                .into_iter()
                .map(|url| {
                    tokio::spawn(async move {
                        let xml = reqwest::get(&url)
                            .await
                            .expect("Fail to GET")
                            .text()
                            .await
                            .expect("Fail to get XML from the web");

                        match podcasting::fetch_feed(&xml).await {
                            Ok(feed) => {
                                tracing::info!("xml url: {url}");
                                tracing::warn!(
                                    "Podcast: {}",
                                    feed.channel.title.unwrap_or_else(|| "None".to_string())
                                );
                                feed.channel.item.iter().take(1).for_each(|entry| {
                                    tracing::warn!(
                                        "title: {}",
                                        entry.title.as_ref().map_or("None", |v| v)
                                    );
                                    tracing::info!(
                                        "Published: {:#?}",
                                        entry.pubDate.as_ref().map_or("None", |v| v)
                                    );
                                });
                            }
                            Err(err) => {
                                tracing::error!("Error: {err:#?}");
                            }
                        }
                    })
                })
                .collect();

            join_all(res).await;
        }
    }
}

// #[cfg(feature = "get_xml")]
async fn _get_xml_data() -> anyhow::Result<()> {
    use futures::future::join_all;
    use reqwest::Client;
    let xml_list = [
        "https://feeds.megaphone.fm/hubermanlab",
        "https://api.substack.com/feed/podcast/1136839.rss",
        "https://lexfridman.com/feed/podcast/",
        "https://coder.show/rss",
        "https://feeds.npr.org/510289/podcast.xml",
        "https://letscast.fm/podcasts/rust-in-production-82281512/feed",
        "http://feeds.libsyn.com/110110/rss",
        "https://anchor.fm/s/f39e007c/podcast/rss",
        "https://feeds.transistor.fm/fork-around-and-find-out",
        "https://feeds.feedburner.com/dancarlin/history?format=xml",
        "http://feeds.wnyc.org/radiolab",
        "https://feeds.megaphone.fm/replyall",
        "https://feeds.simplecast.com/gvtxUiIf",
        "https://feeds.megaphone.fm/vergecast",
        "https://feeds.megaphone.fm/revisionisthistory",
        "https://feeds.npr.org/510308/podcast.xml",
        "https://feeds.simplecast.com/dLRotFGk",
        "https://thewomenintechshow.com/category/podcast/feed/",
        "https://feeds.megaphone.fm/GLT1412515089",
        "https://feeds.simplecast.com/qm_9xx0g",
        "http://rss.art19.com/the-daily",
        "https://feeds.simplecast.com/mKn_QmLS",
        "https://feeds.megaphone.fm/thispastweekend",
        "https://podcastfeeds.nbcnews.com/dateline-nbc",
        "https://www.thisamericanlife.org/podcast/rss.xml",
        "https://feeds.simplecast.com/6Qp23t6h",
        "https://rss.art19.com/smartless",
        "https://feeds.megaphone.fm/newheights",
    ];

    let client = Client::new();

    let results = join_all(xml_list.iter().map(|url| {
        let client = &client;

        async move {
            tracing::warn!("Getting URL: {}", url);
            let body = client
                .get(*url)
                .send()
                .await
                .expect("Fail to await")
                .text()
                .await
                .expect("Fail to await");

            Ok::<(String, String), reqwest::Error>((url.to_string(), body))
        }
    }))
    .await;

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok((url, body)) => {
                tracing::info!("saving: {url}");
                fs::write(format!("pod_feed_{i}.xml"), body)?;
            }
            Err(err) => {
                tracing::error!("Unable to get the xml: {err:#?}");
            }
        }
    }

    Ok(())
}
