use askama::Template;

#[derive(Template)]
#[template(path = "pages/index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub version: String,
    pub genres: Vec<&'static str>,
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            title: String::from("HOME"),
            version: std::env!("CARGO_PKG_VERSION").to_string(),
            genres: [
                "Arts",
                "Business",
                "Comedy",
                "Education",
                "Fiction",
                "Government",
                "Health & Fitness",
                "History",
                "Kids & Family",
                "Leisure",
                "Music",
                "News",
                "Religion & Spirituality",
                "Science",
                "Society & Culture",
                "Sports",
                "Technology",
                "True Crime",
                "TV & Film",
            ]
            .to_vec(),
        }
    }
}

#[derive(Template)]
#[template(path = "pages/podcasts.html")]
pub struct PodcastTemplate {
    pub title: String,
    pub version: String,
}

impl Default for PodcastTemplate {
    fn default() -> Self {
        Self {
            title: String::from("PODCASTS"),
            version: std::env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[derive(Template)]
#[template(path = "pages/discover.html")]
pub struct DiscoveryTemplate {
    pub title: String,
    pub version: String,
}

impl Default for DiscoveryTemplate {
    fn default() -> Self {
        Self {
            title: String::from("PODCASTS"),
            version: std::env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}
