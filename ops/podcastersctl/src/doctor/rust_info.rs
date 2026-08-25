use std::{fs, io::Read, process::Command};

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
pub fn clippy_it() -> anyhow::Result<bool> {
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
    // .expect("Failed to get Clippy output");
    let opt = String::from_utf8(output.stderr)?;
    // .expect("Failed to convert ascii to string");

    // println!("Output contains error: {:#?}", opt.contains("\nerror"));

    if opt.contains("\nerror") {
        Ok(false)
    } else {
        Ok(true)
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
        .trim()
        .trim_matches('"')
        .to_string())
}
