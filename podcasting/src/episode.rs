use std::borrow::Cow;

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "rss")]
pub struct Rss {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    title: String,
    #[serde(default)]
    item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    title: String,
    enclosure: Option<Enclosure>,
    #[serde(default)]
    description: Option<String>,
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
