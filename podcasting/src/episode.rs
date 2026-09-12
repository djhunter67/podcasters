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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
                            duration: match epi
                                .duration
                                .as_ref()
                                .expect("Unable to acquire the duration")
                                .clone()
                                .parse::<u16>()
                            {
                                Ok(num) => Some(num),
                                Err(err) => {
                                    tracing::error!("The duration is not in seconds: {err:#?}");
                                    None
                                }
                            },
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
                }
            }
            Err(err) => {
                tracing::error!("Error: {err:#?}");
                return Err(anyhow::anyhow!("{err:#?}"));
            }
        };
        Ok(pod)
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

    pub fn limit_episode(&mut self, limit: u16) {
        if !limit.eq(&0) {
            self.episodes = episode_lim(&self.episodes, limit).to_vec();
        }
    }

    /// Placeholder
    ///
    /// This function is a placeholder for an actual episode ID
    pub fn get_episode_by_id(&mut self, id: u16) {
        // Episodes are not ID'd

        if usize::from(id) > self.episodes.len() {
            self.episodes = vec![];
            return;
        }

        let mut episode: Episode = Episode::default();
        for (i, pod) in self.episodes.iter().enumerate() {
            if i.eq(&<u16 as Into<usize>>::into(id)) {
                episode = pod.clone();

                break;
            }
        }

        self.episodes = [episode].to_vec();
    }
}

fn episode_lim(epis: &[Episode], limit: u16) -> &[Episode] {
    let limit = usize::from(limit).min(epis.len());

    &epis[..limit]
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub struct Episode {
    title: Option<String>,
    description: Option<String>,
    media_type: Option<String>,
    audio_url: Option<String>,
    published_at: Option<String>,
    duration: Option<u16>,
}

#[cfg(test)]
mod tests {
    use crate::episode;

    use super::*;

    fn episode(title: &str) -> Episode {
        Episode {
            title: Some(title.to_string()),
            description: Some(format!("Description for {title}")),
            media_type: Some("audio/mpeg".to_string()),
            audio_url: Some(format!("https://example.com/{title}.mp3")),
            published_at: Some("2026-09-12".to_string()),
            duration: Some(120),
        }
    }

    #[test]
    fn episode_lim_returns_requested_number_of_episodes() {
        let episodes = vec![episode("one"), episode("two"), episode("three")];

        let result = episode_lim(&episodes, 2);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title.as_deref(), Some("one"));
        assert_eq!(result[1].title.as_deref(), Some("two"));
    }

    #[test]
    fn episode_lim_returns_all_when_limit_equals_length() {
        let episodes = vec![episode("one"), episode("two"), episode("three")];

        let result = episode_lim(&episodes, 3);

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn episode_lim_returns_all_when_limit_exceeds_length() {
        let episodes = vec![episode("one"), episode("two")];

        let result = episode_lim(&episodes, 10);

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn episode_lim_returns_empty_vec_when_limit_is_zero() {
        let episodes = vec![episode("one"), episode("two")];

        let result = episode_lim(&episodes, 0);

        assert_eq!(result, [] as [episode::Episode; 0]);
    }

    #[test]
    fn episode_lim_returns_empty_vec_for_empty_input() {
        let episodes: Vec<Episode> = Vec::new();

        let result = episode_lim(&episodes, 5);

        assert_eq!(result, [] as [episode::Episode; 0]);
    }

    #[test]
    fn episode_lim_preserves_episode_order() {
        let episodes = vec![episode("first"), episode("second"), episode("third")];

        let result = episode_lim(&episodes, 3);

        assert_eq!(result[0].title.as_deref(), Some("first"));
        assert_eq!(result[1].title.as_deref(), Some("second"));
        assert_eq!(result[2].title.as_deref(), Some("third"));
    }

    #[test]
    fn episode_lim_returns_first_two_episodes() {
        let episodes = vec![episode("one"), episode("two"), episode("three")];

        let expected = vec![episode("one"), episode("two")];

        assert_eq!(episode_lim(&episodes, 2), expected);
    }

    #[test]
    fn episode_lim_returns_requested_slice() {
        let episodes = vec![episode("one"), episode("two"), episode("three")];

        let result = episode_lim(&episodes, 2);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title.as_deref(), Some("one"));
        assert_eq!(result[1].title.as_deref(), Some("two"));
    }
}
