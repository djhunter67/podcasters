use crate::episode::Episode;

#[derive(Debug)]
pub struct PodcastFeed<'a> {
    title: &'a str,
    description: Option<&'a str>,
    artwork_url: Option<&'a str>,
    author: Option<&'a str>,
    feed_url: &'a str,
    episodes: Vec<Episode<'a>>,
}
