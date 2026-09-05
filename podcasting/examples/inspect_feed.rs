#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "https://some-real-podcast/feed.xml";

    let podcast = podcasting::fetch_feed(url).await?;

    println!("{podcast:#?}");

    Ok(())
}
