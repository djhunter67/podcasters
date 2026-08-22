mod docker_info;
mod rust_info;

use shared::shell;

const RUSTUP_TOOLCHAIN_VAR: &str = "RUSTUP_TOOLCHAIN";
pub fn execute() {
    // let rust_info = RustStatus::new();

    // RUST
    let mut rust_info = rust_info::RustStatus::new();
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

    println!("{rust_info}");
}

struct DockerStatus {
    container_reachable: String,
    user_may_create_containers: bool,
}

impl Default for DockerStatus {
    fn default() -> Self {
        Self {
            container_reachable: String::new(),
            user_may_create_containers: false,
        }
    }
}

fn is_docker_running() -> anyhow::Result<bool> {
    let output = std::process::Command::new("docker")
        .arg("info")
        .output()
        .expect("Failed to execute docker command");

    // output.status.success()

    println!("Docker info output: {:?}", output.stdout);

    Ok(output.status.success())
}
