mod docker_info;
mod mongo;
mod redis;
mod rust_info;

use std::fmt;

use shared::shell;

use crate::doctor::mongo::databases;

const RUSTUP_TOOLCHAIN_VAR: &str = "RUSTUP_TOOLCHAIN";
pub async fn execute() -> anyhow::Result<()> {
    // let rust_info = RustStatus::new();

    // RUST
    let mut rust_info = RustStatus::default();
    rust_info.rustfmt =
        rust_info::format_workspace_check().unwrap_or_else(|err| format!("Error: {err:#?}"));
    rust_info.toolchain =
        rust_info::get_directory_toolchain().unwrap_or_else(|err| format!("Error: {err:#?}"));
    rust_info.clippy = rust_info::clippy_it().unwrap_or_else(|err| format!("Error: {err:#?}"));
    rust_info.workspace_root =
        shell::find_worspace_root().unwrap_or_else(|err| format!("Error: {err:#?}"));
    rust_info.compiler = rust_info::get_compiler().unwrap_or_else(|err| format!("Error: {err:#?}"));

    // DOCKER
    let mut docker_info = DockerStatus::default();

    docker_info.user_may_create_containers = docker_info::can_user_create_containers()
        .unwrap_or_else(|err| {
            eprintln!("Error checking Docker status: {err:#?}");
            false
        });

    docker_info.container_reachable = docker_info::is_container_reachable().unwrap_or_else(|err| {
        eprintln!("Error checking Docker status: {err:#?}");
        false
    });

    let mongo_conn = std::env::var("APP_MONGO__URI")?;

    // MONGODB
    let mongo_info = MongoDb {
        connection_str: mongo_conn.clone(),
        ping: mongo::ping(&mongo_conn).await?,
        db_databases: databases(&mongo_conn).await?,
    };

    let redis_conn = std::env::var("APP_REDIS__URI")?;

    // REDIS
    let redis_info = Redis {
        connection_str: redis_conn.clone(),
        ping: redis::ping(&redis_conn).await?,
    };

    // OUTPUT
    println!("{rust_info}");
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
}

#[derive(Default)]
struct DockerStatus {
    container_reachable: bool,
    user_may_create_containers: bool,
}

#[derive(Default)]
struct MongoDb {
    connection_str: String,
    ping: bool,
    db_databases: Vec<String>,
}

#[derive(Default)]
struct Redis {
    connection_str: String,
    ping: bool,
}

impl fmt::Display for Redis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Connection str: {}\nPing: {}",
            self.connection_str, self.ping
        )
    }
}

impl fmt::Display for RustStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Toolchain: {}\nRustfmt: {}\nClippy: {}\nCompiler: {}\nWorkspace Root: {}",
            self.toolchain, self.rustfmt, self.clippy, self.compiler, self.workspace_root
        )
    }
}

impl fmt::Display for DockerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Docker container reachable: {}\nUser may create containers: {}",
            self.container_reachable, self.user_may_create_containers
        )
    }
}

impl fmt::Display for MongoDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Connection str: {}\nPing: {}\nDatabases: {:#?}",
            self.connection_str, self.ping, self.db_databases
        )
    }
}
