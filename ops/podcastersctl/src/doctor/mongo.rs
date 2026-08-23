use mongodb::{Client, bson::doc};

pub async fn ping(connection_string: &str) -> anyhow::Result<bool> {
    let client = Client::with_uri_str(connection_string).await?;

    let results = client
        .database("admin")
        .run_command(doc! {
            "ping": 1
        })
        .await?;

    // for result in &results {
    //     println!("Result: {result:#?}");
    // }

    if results.contains_key("ok") {
        return Ok(true);
    }

    Ok(false)
}

pub async fn databases(connection_string: &str) -> anyhow::Result<Vec<String>> {
    let client = Client::with_uri_str(connection_string).await?;

    let list_databases = client.list_database_names().await?;

    // for database in &list_databases {
    // println!("Database: {database:#?}");
    // }

    Ok(list_databases)
}
