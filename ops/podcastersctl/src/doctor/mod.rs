mod docker_info;
mod rust_info;

use std::fmt;

use shared::shell;

const RUSTUP_TOOLCHAIN_VAR: &str = "RUSTUP_TOOLCHAIN";
pub fn execute() {
    // let rust_info = RustStatus::new();

    // RUST
    let mut rust_info = RustStatus::new();
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

    docker_info.user_may_create_containers =
        docker_info::is_docker_running().unwrap_or_else(|err| {
            eprintln!("Error checking Docker status: {err:#?}");
            false
        });

    println!("{rust_info}");
    println!("\n\n");
    println!("{docker_info}");
}

struct RustStatus {
    toolchain: String,
    rustfmt: String,
    clippy: String,
    compiler: String,
    workspace_root: String,
}

impl RustStatus {
    pub const fn new() -> Self {
        Self {
            toolchain: String::new(),
            rustfmt: String::new(),
            clippy: String::new(),
            compiler: String::new(),
            workspace_root: String::new(),
        }
    }
}

struct DockerStatus {
    container_reachable: String,
    user_may_create_containers: bool,
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

impl Default for DockerStatus {
    fn default() -> Self {
        Self {
            container_reachable: String::new(),
            user_may_create_containers: false,
        }
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
