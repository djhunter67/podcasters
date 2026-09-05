#[derive(Debug)]
pub struct Episode<'a> {
    title: &'a str,
    description: Option<&'a str>,
    guid: Option<&'a str>,
    audio_url: Option<&'a str>,
    published_at: Option<&'a str>,
    duration: Option<&'a str>,
}
