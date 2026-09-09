use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::feed::fetch_feed;

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "rss")]
pub struct Rss {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub version: Option<String>,
    #[serde(rename = "atom:link", default)]
    pub atom_links: Vec<AtomLink>,
    pub title: Option<String>,
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "itunes:image", default)]
    pub image: Image,
    #[serde(rename = "content:encoded")]
    pub content: Option<String>,
    #[serde(default)]
    pub item: Vec<Item>,
    #[serde(rename = "itunes:explicit")]
    pub explicit: Option<String>,
    #[serde(rename = "itunes:type")]
    r#type: Option<String>,
    #[serde(rename = "itunes:subtitle")]
    subtitle: Option<String>,
    #[serde(rename = "itunes:author")]
    author: Option<String>,
    #[serde(rename = "itunes:summary")]
    summary: Option<String>,
    #[serde(rename = "itunes:owner", default)]
    owner: ItunesOwner,
}

#[derive(Debug, Deserialize)]
pub struct AtomLink {
    #[serde(rename = "@href")]
    href: Option<String>,
    #[serde(rename = "@rel")]
    rel: Option<String>,
    #[serde(rename = "@type")]
    r#type: Option<String>,
    xmlns: Option<String>,
    media_type: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ItunesOwner {
    #[serde(rename = "itunes:name")]
    name: Option<String>,
    #[serde(rename = "itunes:email")]
    email: Option<String>,
    #[serde(rename = "itunes:category")]
    category: Option<ItunesCategory>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ItunesCategory {
    text: Option<String>,
    #[serde(rename = "itunes:category")]
    category: Box<Option<Self>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Image {
    pub href: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub title: Option<String>,
    link: Option<String>,
    pub description: Option<String>,
    pub pubDate: Option<String>,
    #[serde(rename = "itunes:episodeType")]
    episode_type: Option<String>,
    #[serde(rename = "itunes:episode")]
    episode_number: Option<String>,
    #[serde(rename = "itunes:subtitle")]
    subtitle: Option<String>,
    #[serde(rename = "itunes:summary")]
    summary: Option<String>,
    #[serde(rename = "itunes:duration")]
    duration: Option<String>,
    #[serde(rename = "itunes:explicit")]
    explicit: Option<String>,
    #[serde(rename = "itunes:image")]
    pub image: Option<Image>,
    #[serde(default)]
    enclosure: Vec<Option<Enclosure>>,
}

#[derive(Debug, Deserialize)]
struct Enclosure {
    #[serde(rename = "@url")]
    url: Option<String>,
    #[serde(rename = "@type")]
    r#type: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct Podcast {
    title: Option<String>,
    description: Option<String>,
    artwork_url: Option<String>,
    feed_url: Option<String>,
    episode_count: u16,
    episodes: Vec<Episode>,
    error: Option<String>,
}

impl Podcast {
    /// # Panics
    ///
    /// This function will panic if the ``reqwest`` fail to acquire a ``200 OK``
    /// # Errors
    ///
    /// This function returns an error if any of the ``Podcast`` values are `None`
    pub async fn new(uri: &str) -> anyhow::Result<Self> {
        let xml = reqwest::get(uri)
            .await
            .expect("Fail to GET")
            .text()
            .await
            .expect("Fail to get XML from the web");

        let pod: Self = match fetch_feed(&xml).await {
            Ok(feed) => {
                let feed = feed.channel;

                let episodes: Vec<Episode> = feed
                    .item
                    .par_iter()
                    .map(|epi| -> Episode {
                        Episode {
                            title: epi.title.clone(),
                            description: epi.description.clone(),
                            guid: epi
                                .enclosure
                                .first()
                                .expect("")
                                .as_ref()
                                .expect("")
                                .r#type
                                .clone(),
                            audio_url: epi
                                .enclosure
                                .first()
                                .expect("")
                                .as_ref()
                                .expect("")
                                .url
                                .clone(),
                            published_at: epi.pubDate.clone(),
                            duration: epi.duration.clone(),
                        }
                    })
                    .collect();

                Self {
                    title: feed.title,
                    description: feed.description,
                    artwork_url: feed.image.href,
                    feed_url: feed
                        .atom_links
                        .first()
                        .expect("No Atom:Link found")
                        .href
                        .clone(),
                    episode_count: u16::try_from(
                        feed.item.len().to_string().parse::<usize>().unwrap_or(0),
                    )
                    .expect("u16 overflow for the number of episodes"),
                    episodes,
                    error: None,
                }
            }
            Err(err) => {
                tracing::error!("Error: {err:#?}");
                return Err(anyhow::anyhow!("{err:#?}"));
            }
        };
        Ok(pod)
    }

    #[must_use = "Show any error"]
    pub fn error(error: &anyhow::Error) -> Self {
        Self {
            error: Some(error.to_string()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Episode {
    title: Option<String>,
    description: Option<String>,
    guid: Option<String>,
    audio_url: Option<String>,
    published_at: Option<String>,
    duration: Option<String>,
}

// impl Episode {
//     pub const fn get_title(&self) -> Cow<'static, &str> {
//         Cow::Borrowed(self.title)
//     }

//     pub const fn get_description(&self) -> Cow<'a, &str> {
//         if let Some(data) = self.description {
//             Cow::Owned(data)
//         } else {
//             Cow::Borrowed(&"None")
//         }
//     }

//     pub const fn get_guid(&self) -> Cow<'a, &str> {
//         if let Some(data) = self.guid {
//             Cow::Owned(data)
//         } else {
//             Cow::Borrowed(&"None")
//         }
//     }

//     pub const fn get_audio_url(&self) -> Cow<'a, &str> {
//         if let Some(data) = self.audio_url {
//             Cow::Owned(data)
//         } else {
//             Cow::Borrowed(&"None")
//         }
//     }

//     pub const fn get_published_at(&self) -> Cow<'a, &str> {
//         if let Some(data) = self.published_at {
//             Cow::Owned(data)
//         } else {
//             Cow::Borrowed(&"None")
//         }
//     }

//     pub const fn get_duration(&self) -> Cow<'a, &str> {
//         if let Some(data) = self.duration {
//             Cow::Owned(data)
//         } else {
//             Cow::Borrowed(&"None")
//         }
//     }
// }
