use std::fs;

use futures::future::join_all;
use podcasting::feed::fetch_feed;
use tokio::{io::AsyncReadExt, task::JoinHandle};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(filter)
        .init();

    let xml_list: Vec<String> = vec![
        // Changelog / developer
        String::from("https://changelog.com/podcast/feed"),
        String::from("https://changelog.com/gotime/feed"),
        String::from("https://changelog.com/practicalai/feed"),
        String::from("https://changelog.com/shipit/feed"),
        String::from("https://changelog.com/jsparty/feed"),
        String::from("https://changelog.com/founderstalk/feed"),
        String::from("https://linuxunplugged.com/rss"),
        String::from("https://selfhosted.show/rss"),
        String::from("https://2.5admins.com/feed/podcast"),
        String::from("https://latenightlinux.com/feed/mp3"),
        String::from("https://talkpython.fm/episodes/rss"),
        String::from("https://pythonbytes.fm/episodes/rss"),
        String::from("https://realpython.com/podcasts/rpp/feed"),
        String::from("https://softwareengineeringdaily.com/feed/podcast/"),
        String::from("https://feeds.transistor.fm/oxide-and-friends"),
        String::from("https://atp.fm/rss"),
        // TWiT
        String::from("https://feeds.twit.tv/twit.xml"),
        String::from("https://feeds.twit.tv/sn.xml"),
        String::from("https://feeds.twit.tv/mbw.xml"),
        String::from("https://feeds.twit.tv/ww.xml"),
        String::from("https://feeds.twit.tv/floss.xml"),
        String::from("https://feeds.twit.tv/aaa.xml"),
        String::from("https://feeds.twit.tv/tnw.xml"),
        String::from("https://feeds.twit.tv/twig.xml"),
        String::from("https://feeds.twit.tv/twiet.xml"),
        String::from("https://feeds.twit.tv/twil.xml"),
        String::from("https://feeds.twit.tv/kh.xml"),
        String::from("https://feeds.twit.tv/tri.xml"),
        String::from("https://feeds.twit.tv/htg.xml"),
        String::from("https://feeds.twit.tv/hop.xml"),
        // NPR
        String::from("https://feeds.npr.org/510318/podcast.xml"),
        String::from("https://feeds.npr.org/510310/podcast.xml"),
        String::from("https://feeds.npr.org/510325/podcast.xml"),
        String::from("https://feeds.npr.org/510282/podcast.xml"),
        String::from("https://feeds.npr.org/510298/podcast.xml"),
        String::from("https://feeds.npr.org/510317/podcast.xml"),
        String::from("https://feeds.npr.org/510313/podcast.xml"),
        String::from("https://feeds.npr.org/344098539/podcast.xml"),
        String::from("https://feeds.npr.org/510307/podcast.xml"),
        // ART19 — raw RSS endpoints, NOT /shows/...
        String::from("https://rss.art19.com/business-movers"),
        String::from("https://rss.art19.com/business-wars"),
        String::from("https://rss.art19.com/business-wars-daily"),
        String::from("https://rss.art19.com/history-daily"),
        String::from("https://rss.art19.com/tides-of-history"),
        String::from("https://rss.art19.com/the-fall-of-rome-podcast"),
        String::from("https://rss.art19.com/once-upon-a-crime"),
        String::from("https://rss.art19.com/true-crime-all-the-time"),
        String::from("https://rss.art19.com/generation-why-podcast"),
        String::from("https://rss.art19.com/suspect"),
        String::from("https://rss.art19.com/perfect-person"),
        String::from("https://rss.art19.com/handsome"),
        String::from("https://rss.art19.com/british-scandal"),
        String::from("https://rss.art19.com/this-is-actually-happening-podcast"),
        String::from("https://rss.art19.com/diss-and-tell"),
        String::from("https://rss.art19.com/project-bluebook"),
        String::from("https://rss.art19.com/colony-the-official-podcast"),
        String::from("https://rss.art19.com/the-here-we-go-podcast"),
        String::from("https://rss.art19.com/inside-voices"),
        String::from("https://rss.art19.com/london-is-blue"),
        String::from("https://rss.art19.com/common-sense-with-dr-ben-carson"),
        String::from("https://rss.art19.com/not-another-d-and-d-podcast"),
        String::from("https://rss.art19.com/man-school-202"),
        String::from("https://rss.art19.com/tig-and-cheryl-true-story"),
        String::from("https://rss.art19.com/last-week-in-ai"),
        String::from("https://rss.art19.com/escaping-the-drift"),
        String::from("https://rss.art19.com/lizness-school"),
        String::from("https://rss.art19.com/business-game-changers"),
        String::from("https://rss.art19.com/the-stephen-mansfield-podcast"),
        String::from("https://rss.art19.com/sup-doc-podcast"),
        String::from("https://rss.art19.com/never-not-funny"),
        String::from("https://rss.art19.com/the-ai-xr-podcast"),
        String::from("https://rss.art19.com/watch-what-happens-live-with-andy-cohen"),
        String::from("https://rss.art19.com/late-night-with-seth-meyers-podcast"),
        String::from("https://rss.art19.com/dr-death"),
        String::from("https://rss.art19.com/american-scandal"),
        String::from("https://rss.art19.com/american-history-tellers"),
        String::from("https://rss.art19.com/against-the-odds"),
        String::from("https://rss.art19.com/over-my-dead-body"),
        String::from("https://rss.art19.com/the-shrink-next-door"),
        String::from("https://rss.art19.com/dying-for-sex"),
        String::from("https://rss.art19.com/even-the-rich"),
        String::from("https://rss.art19.com/imagined-life"),
        String::from("https://rss.art19.com/life-is-short-with-justin-long"),
        String::from("https://rss.art19.com/scamfluencers"),
        String::from("https://rss.art19.com/the-next-big-idea"),
        // Additional independent feeds
        String::from("https://risky.biz/feeds/risky-business"),
        String::from("https://www.sciencefriday.com/feed/podcast/"),
        String::from("https://www.quantamagazine.org/feed/podcast/"),
        String::from("https://feeds.megaphone.fm/darknetdiaries"),
        // More ART19 feeds useful for malformed/legacy-feed testing
        String::from("https://rss.art19.com/en-la-sala"),
        String::from("https://rss.art19.com/not-a-very-good-murderer"),
        String::from(
            "https://rss.art19.com/jackass-the-podcast-with-johnny-knoxville-and-jeff-tremaine",
        ),
        String::from("https://rss.art19.com/the-stephen-mansfield-podcast"),
        String::from("https://rss.art19.com/the-ai-xr-podcast"),
        String::from("https://rss.art19.com/last-week-in-ai"),
        String::from("https://rss.art19.com/escaping-the-drift"),
        String::from("https://rss.art19.com/lizness-school"),
        String::from("https://rss.art19.com/business-game-changers"),
        // Huberman Lab
        String::from("https://feeds.megaphone.fm/hubermanlab"),
    ];
    // get_xml_data(xml_list).await?;

    parse_xml(&Devel::Url, xml_list).await;

    Ok(())
}

enum Devel {
    Url,
    Xml,
}

async fn parse_xml(path_forward: &Devel, xml_list: Vec<String>) {
    match path_forward {
        Devel::Xml => {
            let res: Vec<JoinHandle<()>> = (0..xml_list.len())
                .map(|i| {
                    tokio::spawn(async move {
                        let mut file = tokio::fs::File::open(format!("pod_feed_{i}.xml"))
                            .await
                            .expect("Fail to open file");
                        let mut xml: Vec<u8> = Vec::new();

                        file.read_to_end(&mut xml).await.expect("Fail to read file");

                        // convert the Vec<u8> to a String
                        let xml = String::from_utf8(xml).expect("Fail to convert to String");

                        match fetch_feed(&xml).await {
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
                                        entry.pub_date.as_ref().map_or("None", |v| v)
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

                        match fetch_feed(&xml).await {
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
                                        entry.pub_date.as_ref().map_or("None", |v| v)
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
async fn get_xml_data(xml_list: Vec<String>) -> anyhow::Result<()> {
    use futures::future::join_all;
    use reqwest::Client;

    let client = Client::new();

    let results = join_all(xml_list.iter().map(|url| {
        let client = &client;

        async move {
            tracing::warn!("Getting URL: {}", url);
            let body = client
                .get(url)
                .send()
                .await
                .expect("Fail to await")
                .text()
                .await
                .expect("Fail to await");

            Ok::<(String, String), reqwest::Error>((url.clone(), body))
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
