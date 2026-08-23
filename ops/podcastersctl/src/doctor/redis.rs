use redis::aio::ConnectionManager;

pub async fn ping(connection_string: &str) -> anyhow::Result<bool> {
    let client = redis::Client::open(connection_string)?;

    let mut connection = ConnectionManager::new(client).await?;

    let pong: String = redis::cmd("PING").query_async(&mut connection).await?;

    Ok(pong == "PONG")
}
