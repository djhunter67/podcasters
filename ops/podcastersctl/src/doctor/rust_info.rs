use std::{fmt, fs, io::Read, process::Command};

pub struct RustStatus {
    pub toolchain: String,
    pub rustfmt: String,
    pub clippy: String,
    pub compiler: String,
    pub workspace_root: String,
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

pub fn get_directory_toolchain() -> anyhow::Result<String> {
    let output = Command::new("rustup")
        .args(["show", "active-toolchain"])
        .env_remove(super::RUSTUP_TOOLCHAIN_VAR)
        .output()?;
    let toolchain = String::from_utf8(output.stdout)?;

    Ok(toolchain.trim().to_string())
}

/// cargo fmt --all -- --check
pub fn format_workspace_check() -> anyhow::Result<String> {
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
        Ok(String::from("pass"))
    } else {
        Ok(String::from("fail"))
    }
}

/// cargo clippy --workspace --all-targets --all-features -- -D warnings
pub fn clippy_it() -> anyhow::Result<String> {
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

    if opt.contains("\nerror") {
        Ok(String::from("fail"))
    } else {
        Ok(String::from("pass"))
    }
}

pub fn get_compiler() -> anyhow::Result<String> {
    let root = shared::shell::find_worspace_root()?;

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

    Ok(str_buf
        .split('=')
        .next_back()
        .expect("Fail to next")
        .to_string()
        .trim()
        .trim_matches('"')
        .to_string())
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
