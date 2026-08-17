//! Initialize and return a connection to the ``Postgresql`` database.

use std::{sync::Arc, time::Duration};

use crate::settings::Settings;
use r2d2::Pool;
use r2d2_postgres::{self, postgres::NoTls, PostgresConnectionManager};
use tracing::instrument;

#[must_use]
#[instrument(
    name = "Establish a connection to Postgresq1",
    level = "info",
    target = "demo_web_app",
    skip(settings)
)]
pub fn establish_connection(
    settings: &Settings,
    postgres_pool: PostgresConnectionManager<NoTls>,
) -> Pool<PostgresConnectionManager<NoTls>> {
    let manager = Arc::into_inner(postgres_pool.into()).expect("No connection found");

    Pool::builder()
        .max_size(1)
        .connection_timeout(Duration::from_secs(
            settings.postgres.connection_timeout.into(),
        ))
        .build(manager)
        .expect("Failed to create the connection pool")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use r2d2_postgres::postgres::{config::SslMode, Config};
    use rstest::{fixture, rstest};

    use crate::settings;

    use super::*;

    #[fixture]
    fn manager() -> PostgresConnectionManager<NoTls> {
        let settings = settings::get().unwrap();

        PostgresConnectionManager::new(
            Config::new()
                .user(&settings.postgres.username)
                .password(settings.postgres.password.clone())
                .dbname(&settings.postgres.db)
                .host(&settings.postgres.host)
                .port(settings.postgres.port)
                .application_name(&settings.postgres.app_name)
                .connect_timeout(Duration::from_secs(
                    settings.postgres.connection_timeout.into(),
                ))
                .ssl_mode(SslMode::Disable)
                .options(format!("--work_mem={}", settings.postgres.working_memory).as_str())
                .clone(),
            NoTls,
        )
    }

    #[rstest]
    fn test_can_write_and_read_postgres(manager: PostgresConnectionManager<NoTls>) {
        let pool = establish_connection(&settings::get().unwrap(), manager);

        let mut conn = pool.get().unwrap();

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL
        )",
            &[],
        )
        .unwrap();

        // Insert test value
        let test_name = "Test_Name";

        conn.execute("INSERT INTO test (name) VALUES ($1)", &[&test_name])
            .unwrap();

        // Read the value back
        let result = conn.query("SELECT * FROM test", &[]).unwrap();

        //Drop the database
        conn.execute("DROP TABLE test", &[]).unwrap();

        // Close the connection
        drop(conn);

        assert_eq!(result.len(), 1);
    }

    #[rstest]
    fn test_count_writes_postgres(manager: PostgresConnectionManager<NoTls>) {
        let settings = settings::get().unwrap();
        let pool = establish_connection(&settings, manager);

        let mut conn = pool.get().unwrap();

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS test_1 (
	    id SERIAL PRIMARY KEY,
	    name TEXT NOT NULL
	)",
            &[],
        )
        .unwrap();

        for i in 0..10 {
            let test_name = format!("Test_Name_{i}");

            conn.execute("INSERT INTO test_1 (name) VALUES ($1)", &[&test_name])
                .unwrap();
        }

        // Read the value back
        let result = conn.query("SELECT * FROM test_1", &[]).unwrap();

        //Drop the database
        conn.execute("DROP TABLE test_1", &[]).unwrap();

        // Close the connection
        drop(conn);

        assert_eq!(result.len(), 10);
    }

    #[rstest]
    fn test_can_create_four_tables_postgres(manager: PostgresConnectionManager<NoTls>) {
        let settings = settings::get().unwrap();
        let pool = establish_connection(&settings, manager);

        let mut conn = pool.get().unwrap();

        let tables = vec!["test_11", "test_12", "test_13", "test_14"];

        for table in &tables {
            conn.execute(
                format!(
                    "CREATE TABLE IF NOT EXISTS {table} (
		    id SERIAL PRIMARY KEY,
		    name TEXT NOT NULL
		)",
                )
                .as_str(),
                &[],
            )
            .unwrap();
        }

        // Insert test value
        for table in &tables {
            let test_name = format!("Test_Name_{table}");

            conn.execute(
                format!("INSERT INTO {table} (name) VALUES ($1)").as_str(),
                &[&test_name],
            )
            .unwrap();
        }

        // Read the value back
        let mut count = 0;
        for table in &tables {
            let result = conn
                .query(format!("SELECT * FROM {table}").as_str(), &[])
                .unwrap();
            count += result.len();
        }

        //Drop the database
        for table in tables {
            conn.execute(format!("DROP TABLE {table}").as_str(), &[])
                .unwrap();
        }

        // Close the connection
        drop(conn);

        assert_eq!(count, 4);
    }
}
