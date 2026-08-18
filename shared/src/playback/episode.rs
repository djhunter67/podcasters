use std::time::Duration;

pub struct EpisodeSummary {
    pub id: EpisodeId,
    pub podcast_id: PodcastId,
    pub title: String,
    pub description: Option<String>,
    pub duration: Option<Duration>,
    pub published_at: DateTime<Utc>,
    pub artwork_url: Option<Url>,
}

pub struct EpisodeId {}
pub struct PodcastId {}
pub enum Url {}
pub struct DateTime<T>
where
    T: Sized,
{
    item: T,
}

pub enum Utc {}
