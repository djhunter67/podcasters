use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(filter)
        .init();

    // #[cfg(feature = "get_xml")]
    // get_xml_data().await?;
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

    for xml in xml_list {
        // tracing::info!("Checking file: pod_feed_{val}.xml");
        // let xml = format!("pod_feed_{val}.xml");
        match podcasting::fetch_feed(xml).await {
            Ok(_) => (),
            Err(err) => {
                tracing::error!("Error: {err:#?}");
            }
        }
    }

    // tracing::info!("{podcast:#?}");

    Ok(())
}

// #[cfg(feature = "get_xml")]
async fn get_xml_data() -> anyhow::Result<()> {
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
                tokio::fs::write(format!("pod_feed_{i}.xml"), body).await?;
            }
            Err(err) => {
                tracing::error!("Unable to get the xml: {err:#?}");
            }
        }
    }

    Ok(())
}
