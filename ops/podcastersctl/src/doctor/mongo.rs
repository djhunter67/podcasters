use mongodb::{Client, bson::doc};
use tokio::net::TcpStream;

pub async fn ping(connection_string: &str) -> anyhow::Result<bool> {
    let client = Client::with_uri_str(connection_string).await?;

    let results = client
        .database("admin")
        .run_command(doc! {
            "ping": 1
        })
        .await?;

    Ok(results.contains_key("ok"))
}

pub async fn databases(connection_string: &str) -> anyhow::Result<Vec<String>> {
    let client = match Client::with_uri_str(connection_string).await {
        Ok(val) => val,
        Err(_err) => return Err(anyhow::Error::msg("No connection string found")),
    };

    let list_databases = client.list_database_names().await?;

    let mut return_db_names: Vec<String> = vec![];
    for database in list_databases {
        return_db_names.push(database);
    }

    Ok(return_db_names)
}

pub async fn tcp_reachable(host: &str, port: u16) -> anyhow::Result<bool> {
    Ok(TcpStream::connect((host, port)).await.is_ok())
}
