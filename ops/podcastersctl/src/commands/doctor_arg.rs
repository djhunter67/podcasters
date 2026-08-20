use std::{env, fmt, fs, io::Read, process::Command};

const RUSTUP_TOOLCHAIN_VAR: &str = "RUSTUP_TOOLCHAIN";
pub fn execute() {
    println!("Doctor called");

    // let rust_info = RustStatus::new();

    // RUST
    // Get the Rust toolchain
    let rust_info = RustStatus::new()
        .format_workspace()
        .expect("Issues formatting")
        .format_workspace_check()
        .expect("Issuse checking the format")
        .get_directory_toolchain()
        .expect("Fail to get toolchain")
        .clippy_it()
        .expect("Failed to activate clippy")
        .get_workspace_root()
        .expect("Failure to query workspace root")
        .get_compiler()
        .expect("Failure to query compiler");

    // Get the output of rustfmt

    // Get the output of clippy

    // Get the current compiler

    println!("{rust_info}");

    // DOCKER
    // Is the daemon reachable?

    // Can the current user create containers

    // MONGODB
    // Is MONGODB reachable

    // Get the result of PING

    // Does the database for mongo testing exist? database: podcasters_test

    //REDIS
    // Is REDIS reachable

    // Get the result of PING

    // KUBERNETES
    // Get the kubernetes context

    // Is the cluster reachable

    // How many nodes are there versus how many nodes expected

    // PODCASTERS
    // Backend configuration

    // Frontend configuration

    // API health

    // frontend health

    // OVERALL report
}

struct RustStatus {
    toolchain: String,
    rustfmt: String,
    clippy: String,
    // Name of the compiler ex. cranelift
    compiler: String,
    workspace_root: String,
}

impl RustStatus {
    pub const fn new() -> Self {
        Self {
            toolchain: String::new(),
            rustfmt: String::new(),
            clippy: String::new(),
            // Name of the compiler ex. cranelift
            compiler: String::new(),
            workspace_root: String::new(),
        }
    }

    fn get_directory_toolchain(mut self) -> anyhow::Result<Self> {
        let output = Command::new("rustup")
            .args(["show", "active-toolchain"])
            .env_remove(RUSTUP_TOOLCHAIN_VAR)
            .output()?;
        let toolchain = String::from_utf8(output.stdout)?;

        self.toolchain = toolchain.trim().to_string();
        Ok(self)
    }

    /// cargo fmt --all -- --check
    fn format_workspace_check(mut self) -> anyhow::Result<Self> {
        let output = Command::new("cargo")
            .args([
                "fmt",
                "--all",
                "--",
                "--color",
                "never",
                "--error-on-unformatted",
            ])
            .output()?;
        let otpt = String::from_utf8(output.stdout)?;
        self.rustfmt = otpt;
        Ok(self)
    }

    fn format_workspace(mut self) -> anyhow::Result<Self> {
        let output = Command::new("cargo").args(["fmt", "--all"]).output()?;
        let otpt = String::from_utf8(output.stdout)?;

        self.rustfmt = otpt;
        Ok(self)
    }

    /// cargo clippy --workspace --all-targets --all-features -- -D warnings
    fn clippy_it(mut self) -> anyhow::Result<Self> {
        let output = Command::new("cargo")
            .args([
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .output()?;
        let otpt = String::from_utf8(output.stdout)?;

        self.clippy = otpt;

        Ok(self)
    }

    fn get_workspace_root(mut self) -> anyhow::Result<Self> {
        // get the workspace root

        self.workspace_root = find_worspace_root()?;

        Ok(self)
    }

    fn get_compiler(mut self) -> anyhow::Result<Self> {
        let root = find_worspace_root()?;

        let path = std::path::Path::new(&root);

        for file in path.read_dir()? {
            let directory = file?.file_name();
            println!("File: {}", directory.to_string_lossy());
            if directory.eq(".cargo") {
                let next_file = path.join(directory);

                // get the file
            }
        }

        self.compiler = "TBD".to_string();

        Ok(self)
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

fn find_worspace_root() -> anyhow::Result<String> {
    let wkspc_root = env::current_dir()?;

    let mut workspace_root = String::new();

    for root in wkspc_root.ancestors() {
        let dir_files = fs::read_dir(root)?;

        for file in dir_files {
            let entry = file?;

            if entry.file_name().eq("Cargo.toml") {
                let mut file_buf: [u8; 12] = [0; 12]; // Only need the first line

                let mut file = fs::File::open(root.join(entry.file_name()))?;

                let () = file.read_exact(&mut file_buf)?;

                let contents = String::from_utf8_lossy(&file_buf).to_string();
                drop(file);

                contents.split('\n').next().map_or_else(
                    || {
                        eprintln!("Unable to parse the first line");
                        ""
                    },
                    |first_part| first_part,
                );

                if contents.contains("[workspace]") {
                    workspace_root = root.to_string_lossy().to_string();
                    break;
                }
            }
        }
    }

    Ok(workspace_root)
}
