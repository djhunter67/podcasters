mod docker_info;
mod mongo;
mod redis;
mod rust_info;

use shared::shell;
use std::{fmt, hash::Hash};

use crate::doctor::mongo::{databases, tcp_reachable};

const RUSTUP_TOOLCHAIN_VAR: &str = "RUSTUP_TOOLCHAIN";
const MONGO_CONN_URI: &str = "APP_MONGO__URI";
const REDIS_CONN_URI: &str = "APP_REDIS__URI";
const DB_HOST: &str = "10.20.20.205";
const CACHE_HOST: &str = "10.20.20.202";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

pub async fn execute() -> anyhow::Result<()> {
    // RUST

    tokio::task::spawn_blocking(|| {
        let mut rust_info = RustStatus::default();
        let mut rust_errors: Vec<String> = vec![];
        let mut error: String;
        (rust_info.rustfmt, error) = colorize(rust_info::format_workspace_check());
        rust_errors.push(error);
        (rust_info.toolchain, error) = colorize(rust_info::get_directory_toolchain());
        rust_errors.push(error);
        (rust_info.clippy, error) = colorize(rust_info::clippy_it());
        rust_errors.push(error);
        (rust_info.workspace_root, error) = colorize(shell::find_worspace_root());
        rust_errors.push(error);
        (rust_info.compiler, error) = colorize(rust_info::get_compiler());
        rust_errors.push(error);

        rust_info.errors = filter_errors(&rust_errors);

        println!("{rust_info}");
    })
    .await
    .expect("Unable to spawn block");

    // DOCKER
    let mut docker_error: Vec<String> = vec![];
    let mut docker_info = DockerStatus::default();

    let (create, create_error) = colorize(docker_info::can_user_create_containers());

    docker_info.user_may_create_containers = create;
    docker_error.push(create_error);

    let (reachable, reachable_error) = colorize(docker_info::is_container_reachable());

    docker_info.container_reachable = reachable;
    docker_error.push(reachable_error);

    docker_info.errors = filter_errors(&docker_error);

    // MONGODB
    let mut db_errors: Vec<String> = vec![];
    let mongo_conn: String = match std::env::var(MONGO_CONN_URI) {
        Ok(val) => val,
        Err(err) => {
            db_errors.push(err.to_string());
            // eprintln!("Env Error: {err}");
            String::from("Environment Variable Not Found")
        }
    };

    let (connection_str, con_error) = &colorize(sanitize_pw(&mongo_conn));
    db_errors.push(con_error.clone());
    let (tcp, ping, db_databases) = tokio::join!(
        tcp_reachable(DB_HOST, 27017),
        mongo::ping(&mongo_conn),
        databases(&mongo_conn)
    );

    let ((tcp, tcp_error), (ping, ping_error)): (&(String, String), &(String, String)) =
        (&colorize(tcp), &colorize(ping));
    db_errors.push(tcp_error.clone());
    db_errors.push(ping_error.clone());

    let mongo_info = MongoDb {
        connection_str,
        tcp_reachable: tcp,
        ping,
        db_databases: match db_databases {
            Ok(val) => val,
            Err(err) => {
                db_errors.push(err.to_string());
                vec![String::from("Unknown")]
            }
        },
        errors: filter_errors(&db_errors),
    };

    // REDIS

    let mut cache_errors: Vec<String> = vec![];
    let redis_conn = match std::env::var(REDIS_CONN_URI) {
        Ok(val) => val,
        Err(err) => {
            cache_errors.push(err.to_string());

            String::from("Environment Variable Not Found")
        }
    };

    let (tcp, ping) = tokio::join!(tcp_reachable(CACHE_HOST, 6379), redis::ping(&redis_conn));

    let (connection_str, error) = &colorize(sanitize_pw(&redis_conn));
    cache_errors.push(error.clone());

    let (tcp_reachable, error) = &colorize(tcp);
    cache_errors.push(error.clone());

    let (ping, error) = &colorize(ping);
    cache_errors.push(error.clone());

    let redis_info = Redis {
        connection_str,
        tcp_reachable,
        ping,
        errors: filter_errors(&cache_errors),
    };

    // OUTPUT

    println!("\n");
    println!("{docker_info}");
    println!("\n");
    println!("{mongo_info}");
    println!("\n");
    println!("{redis_info}");

    Ok(())
}

#[derive(Default)]
struct RustStatus {
    toolchain: String,
    rustfmt: String,
    clippy: String,
    compiler: String,
    workspace_root: String,
    errors: Vec<String>,
}

#[derive(Default)]
struct DockerStatus {
    container_reachable: String,
    user_may_create_containers: String,
    errors: Vec<String>,
}

#[derive(Default)]
struct MongoDb<'a> {
    connection_str: &'a str,
    tcp_reachable: &'a str,
    ping: &'a str,
    db_databases: Vec<String>,
    errors: Vec<String>,
}

#[derive(Default)]
struct Redis<'a> {
    connection_str: &'a str,
    tcp_reachable: &'a str,
    ping: &'a str,
    errors: Vec<String>,
}

impl fmt::Display for Redis<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Connection str: {}\nTCP reachable: {}\nPing: {}\nCache Errors: {:#?}",
            self.connection_str, self.tcp_reachable, self.ping, self.errors
        )
    }
}

impl fmt::Display for RustStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Toolchain: {}\nRustfmt: {}\nClippy: {}\nCompiler: {}\nWorkspace Root: {}\nRust Errors: {:#?}",
            self.toolchain,
            self.rustfmt,
            self.clippy,
            self.compiler,
            self.workspace_root,
            self.errors
        )
    }
}

impl fmt::Display for DockerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Docker container reachable: {}\nUser may create containers: {}\nDocker Errors: {:#?}",
            self.container_reachable, self.user_may_create_containers, self.errors
        )
    }
}

impl fmt::Display for MongoDb<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Connection str: {}\nTCP reachable: {}\nPing: {}\nDatabases: {:#?}\nDB Errors: {:#?}",
            self.connection_str, self.tcp_reachable, self.ping, self.db_databases, self.errors
        )
    }
}
pub fn sanitize_pw(connection_string: &str) -> anyhow::Result<String> {
    let (beginning, _end): (&str, &str) =
        if let Some((val, val_2)) = connection_string.rsplit_once('@') {
            (val, val_2)
        } else {
            // eprintln!("Incorrect string passed to the sanitizer");
            return Ok(String::from(
                "Incorrect string passed to password sanitizer",
            ));
        };

    let substring: &str = if let Some(val) = beginning.rsplit(':').next() {
        val
    } else {
        eprintln!("Unable the split the string");
        return Err(anyhow::Error::msg("Unable the split the string"));
    };
    // .expect("Failed to next back");

    Ok(connection_string.replace(substring, "******"))
}

fn colorize<T, E>(val: Result<T, E>) -> (String, String)
where
    T: Clone + fmt::Debug + Hash + ToString + Send + Sync,
    E: fmt::Debug + 'static + Sized + fmt::Display,
{
    let mut error = String::new();
    let mut val: String = match val {
        Ok(v) => v.to_string(),
        Err(err) => {
            // eprintln!("Error in coloring: {err:#?}");
            error = err.to_string();
            "Unknown".to_string()
        }
    };

    if val.eq("Unknown") || val.eq("false") || val.eq("fail") {
        val = format!("{RED}{val}{RESET}");
    } else {
        val = format!("{GREEN}{val}{RESET}");
    }
    (val, error)
}

fn filter_errors(errors: &[String]) -> Vec<String> {
    // let mut empty: bool = false;
    // for error in &errors {
    //     empty = (*error).is_empty();
    // }

    errors
        .iter()
        .filter(|val| !val.is_empty())
        .cloned()
        .collect::<Vec<String>>()

    // if !empty {
    //     return errors;
    // }
    // vec![]
}
