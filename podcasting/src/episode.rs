use std::borrow::Cow;

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "rss")]
pub struct Rss<'a, 'de: 'a> {
    pub channel: Channel<'a>,
}

#[derive(Debug, Deserialize)]
pub struct Channel<'de, 'a> {
    #[serde(rename = "atom:link")]
    pub atom_link: &'de str,
    pub title: &'a str,
    pub language: &'a str,
    pub copyright: &'a str,
    pub description: &'a str,
    pub image: Image<'a>,
    pub itunes: Itunes<'a>,
    #[serde(rename = "content:encodedt")]
    pub content: &'a str,
    #[serde(default)]
    pub item: Vec<Item<'a>>,
}

#[derive(Debug, Deserialize)]
pub struct Itunes<'a> {
    #[serde(rename = "itunes:explicit")]
    pub explicit: bool,
    #[serde(rename = "itunes:type")]
    r#type: &'a str,
    #[serde(rename = "itunes:subtitle")]
    subtitle: &'a str,
    #[serde(rename = "itunes:author")]
    author: &'a str,
    #[serde(rename = "itunes:summary")]
    summary: &'a str,
    #[serde(rename = "itunes:owner")]
    owner: ItunesOwner<'a>,
}

#[derive(Debug, Deserialize)]
pub struct ItunesOwner<'a> {
    #[serde(rename = "itunes:name")]
    name: &'a str,
    #[serde(rename = "itunes:email")]
    email: &'a str,
    #[serde(rename = "itunes:image")]
    image: &'a str,
    #[serde(rename = "itunes:category")]
    category: Option<ItunesCategory<'a>>,
}

#[derive(Debug, Deserialize)]
pub struct ItunesCategory<'a> {
    text: &'a str,
    #[serde(rename = "itunes:category")]
    category: Box<Option<Self>>,
}

#[derive(Debug, Deserialize)]
pub struct Image<'a> {
    pub url: &'a str,
    pub title: &'a str,
    pub link: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct Item<'a> {
    title: &'a str,
    link: &'a str,
    description: Option<String>,
    pubDate: &'a str,
    #[serde(rename = "itunes:episodeType")]
    episode_type: &'a str,
    #[serde(rename = "itunes:episode")]
    episode_number: u16,
    #[serde(rename = "itunes:subtitle")]
    subtitle: &'a str,
    #[serde(rename = "itunes:summary")]
    summary: &'a str,
    #[serde(rename = "itunes:duration")]
    Duration: u16,
    #[serde(rename = "itunes:explicit")]
    explicit: bool,
    #[serde(default)]
    enclosure: Option<Enclosure>,
}

#[derive(Debug, Deserialize)]
struct Enclosure {
    #[serde(rename = "@url")]
    url: String,
    #[serde(rename = "@type")]
    r#type: String,
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
