use std::{fmt, fs, io::Read, process::Command};

use shared::shell;

const RUSTUP_TOOLCHAIN_VAR: &str = "RUSTUP_TOOLCHAIN";
pub fn execute() {
    // let rust_info = RustStatus::new();

    // RUST
    // Get the Rust toolchain
    let rust_info = RustStatus::new()
        // .format_workspace()
        // .expect("Issues formatting")
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

    println!("{rust_info}");
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
        if otpt.is_empty() {
            self.rustfmt = String::from("pass");
        } else {
            self.rustfmt = String::from("fail");
        }

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
        let opt = String::from_utf8(output.stderr)?;

        // println!("Output contains error: {:#?}", opt.contains("\nerror"));

        self.clippy = if opt.contains("\nerror") {
            String::from("fail")
        } else {
            String::from("pass")
        };

        Ok(self)
    }

    fn get_workspace_root(mut self) -> anyhow::Result<Self> {
        // get the workspace root

        self.workspace_root = shell::find_worspace_root()?;

        Ok(self)
    }

    fn get_compiler(mut self) -> anyhow::Result<Self> {
        let root = shell::find_worspace_root()?;

        let path = std::path::Path::new(&root);

        let mut final_path = String::new();
        'outer: for file in path.read_dir()? {
            let directory = file?.file_name();
            if directory.eq(".cargo") {
                let next_file = path.join(directory);
                for file in next_file.read_dir()? {
                    let file_found = file?.file_name();
                    if file_found.eq("config.toml") {
                        final_path = next_file.join(file_found).to_string_lossy().to_string();
                        break 'outer;
                    }
                }
            }
        }

        let mut file_buf = Vec::new();

        let file_handle = fs::File::open(final_path);

        file_handle?.read_to_end(&mut file_buf)?;

        let mut str_buf = String::new();
        for mut line in file_buf.split(|chars| *chars == b'\n') {
            // println!("line: {}", String::from_utf8_lossy(line));
            if String::from_utf8_lossy(line).contains("codegen-backend") {
                let _ = line.read_to_string(&mut str_buf);
            }
        }

        self.compiler = str_buf
            .split('=')
            .next_back()
            .expect("Fail to next")
            .to_string()
            .trim()
            .trim_matches('"')
            .to_string();

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
