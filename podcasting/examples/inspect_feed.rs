use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(filter)
        .init();

    let url = "https://feeds.megaphone.fm/hubermanlab";

    let podcast = podcasting::fetch_feed(url).await?;

    tracing::info!("{podcast:#?}");

    Ok(())
}
