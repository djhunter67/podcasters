#![allow(dead_code)]
use mongodb::{
    bson::{oid, to_document},
    options::UpdateModifications,
};
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
    #[serde(rename = "pubDate")]
    pub pub_date: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Podcast {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<mongodb::bson::oid::ObjectId>,
    uri: String,
    title: Option<String>,
    description: Option<String>,
    artwork_url: Option<String>,
    feed_url: Option<String>,
    episode_count: u16,
    episodes: Vec<Episode>,
    error: Option<String>,
}

impl From<Podcast> for UpdateModifications {
    fn from(value: Podcast) -> Self {
        let doc = to_document(&value).expect("failed to serialize Podcast");
        Self::Document(doc)
    }
}

impl Podcast {
    /// # Panics
    ///
    /// This function will panic if the ``reqwest`` fail to acquire a ``200 OK``
    /// # Errors
    ///
    /// This function returns an error if any of the ``Podcast`` values are `None`
    pub async fn new(xml: &str) -> anyhow::Result<Self> {
        let pod: Self = match fetch_feed(xml).await {
            Ok(feed) => {
                let feed = feed.channel;

                let episodes: Vec<Episode> = feed
                    .item
                    .par_iter()
                    .map(|epi| -> Episode {
                        Episode {
                            title: epi.title.clone(),
                            description: epi.description.clone(),
                            media_type: epi
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
                            published_at: epi.pub_date.clone(),
                            duration: epi.duration.clone(),
                        }
                    })
                    .collect();

                Self {
                    id: None,
                    uri: String::new(),
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
    pub fn error(error: &str) -> Self {
        Self {
            error: Some(error.to_string()),
            ..Default::default()
        }
    }

    pub fn get_title(&self) -> String {
        self.title
            .as_ref()
            .map_or_else(|| String::from("Not Found"), String::from)
    }

    #[must_use = "Get the OID when a cache misses"]
    pub const fn get_id(&self) -> Option<oid::ObjectId> {
        self.id
    }

    /// # Errors
    ///
    /// No OID found in the passed in Option
    pub fn set_id(&mut self, id: Option<oid::ObjectId>) -> anyhow::Result<()> {
        if let Some(oid) = id {
            self.id = Some(oid);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Unable to set the OID: {id:#?}"))
        }
    }

    pub fn set_uri(&mut self, uri: &str) {
        self.uri = String::from(uri);
    }

    #[must_use = "Required to get the URI"]
    pub fn get_uri(&self) -> String {
        self.uri.clone()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Episode {
    title: Option<String>,
    description: Option<String>,
    media_type: Option<String>,
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

//     pub const fn get_media_type(&self) -> Cow<'a, &str> {
//         if let Some(data) = self.media_type {
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
