use std::borrow::Cow;

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "rss")]
pub struct Rss {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub version: Option<String>,
    #[serde(rename = "atom:link")]
    pub atom_link: Option<String>,
    pub title: Option<String>,
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "itunes:image")]
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
    #[serde(rename = "itunes:owner")]
    owner: ItunesOwner,
}

#[derive(Debug, Deserialize)]
pub struct ItunesOwner {
    #[serde(rename = "itunes:name")]
    name: Option<String>,
    #[serde(rename = "itunes:email")]
    email: Option<String>,
    #[serde(rename = "itunes:category")]
    category: Option<ItunesCategory>,
}

#[derive(Debug, Deserialize)]
pub struct ItunesCategory {
    text: Option<String>,
    #[serde(rename = "itunes:category")]
    category: Box<Option<Self>>,
}

#[derive(Debug, Deserialize)]
// #[serde(rename = "href")]
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
    enclosure: Option<Enclosure>,
}

#[derive(Debug, Deserialize)]
struct Enclosure {
    #[serde(rename = "@url")]
    url: Option<String>,
    #[serde(rename = "@type")]
    r#type: Option<String>,
}

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Episode<'a> {
    title: &'a str,
    description: Option<&'a str>,
    guid: Option<&'a str>,
    audio_url: Option<&'a str>,
    published_at: Option<&'a str>,
    duration: Option<&'a str>,
}

impl<'a> Episode<'a> {
    pub const fn get_title(&self) -> Cow<'a, &str> {
        Cow::Borrowed(&self.title)
    }

    pub const fn get_description(&self) -> Cow<'a, &str> {
        if let Some(data) = self.description {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&"None")
        }
    }

    pub const fn get_guid(&self) -> Cow<'a, &str> {
        if let Some(data) = self.guid {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&"None")
        }
    }

    pub const fn get_audio_url(&self) -> Cow<'a, &str> {
        if let Some(data) = self.audio_url {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&"None")
        }
    }

    pub const fn get_published_at(&self) -> Cow<'a, &str> {
        if let Some(data) = self.published_at {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&"None")
        }
    }

    pub const fn get_duration(&self) -> Cow<'a, &str> {
        if let Some(data) = self.duration {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&"None")
        }
    }
}
