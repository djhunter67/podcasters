use mongodb::{Client, bson::doc};

pub fn connection_str() -> anyhow::Result<String> {
    Ok(String::from("Unknown"))
}

pub async fn ping(connection_string: &str) -> anyhow::Result<bool> {
    let client = Client::with_uri_str(connection_string).await?;

    client.database("admin").run_command(doc! {
        "ping": 1
    });

    Ok(true)
}

pub async fn databases(connection_string: &str) -> anyhow::Result<Vec<String>> {
    let client = Client::with_uri_str(connection_string).await?;

    let list_databases = client.list_database_names().await?;

    for database in list_databases.iter() {
        println!("Database: {database:#?}");
    }

    Ok(vec![])
}
