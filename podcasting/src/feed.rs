use std::borrow::Cow;

use serde::Deserialize;

use crate::episode::Episode;

#[derive(Debug, Deserialize)]
#[serde(rename = "rss")]
pub struct PodcastFeed<'a> {
    title: &'a str,
    description: Option<&'a str>,
    artwork_url: Option<&'a str>,
    author: Option<&'a str>,
    feed_url: &'a str,
    episodes: Vec<Episode<'a>>,
}

impl<'a> PodcastFeed<'a> {
    pub const fn get_title(&self) -> Cow<'a, &str> {
        Cow::Borrowed(&self.title)
    }

    pub const fn get_description(&self) -> Cow<'a, &str> {
        if let Some(data) = self.description {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&&"None")
        }
    }

    pub const fn get_artwork_url(&self) -> Cow<'a, &str> {
        if let Some(data) = self.artwork_url {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&&"None")
        }
    }

    pub const fn get_author(&self) -> Cow<'a, &str> {
        if let Some(data) = self.author {
            Cow::Owned(data)
        } else {
            Cow::Borrowed(&&"None")
        }
    }
}

#[derive(Debug)]
pub struct PodcastSearchResult {}
