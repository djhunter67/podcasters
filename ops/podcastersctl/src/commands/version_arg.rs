use std::{fmt, io::Read, path, process::Command};

use shared::shell;

use crate::version::{self, VerSubcommand};

#[derive(Debug)]
struct Version {
    frontend: String,
    backend: String,
    commit_hash: String,
    build_time: String,
    rustc: String,
    rustup_target: String,
}

impl Version {
    pub const fn new(
        frontend: String,
        backend: String,
        commit_hash: String,
        build_time: String,
        rustc: String,
        rustup_target: String,
    ) -> Self {
        Self {
            frontend,
            backend,
            commit_hash,
            build_time,
            rustc,
            rustup_target,
        }
    }
}

pub fn execute(version: &version::VerState) {
    match &version.version {
        // Show version of the crate
        // Show the git commit hash
        // Time of the build in the target directory
        // The rustc version
        // The current rustup target
        VerSubcommand::Production => {
            // println!("The production version information is to be provided");
            let b_ver = get_versioned("backend").unwrap_or_else(|err| format!("Error: {err:#?}"));
            let f_ver = get_versioned("frontend").unwrap_or_else(|err| format!("Error: {err:#?}"));
            let git_com = match get_commit_hash() {
                Ok(val) => val,
                Err(err) => {
                    eprintln!("{err:#?}");
                    String::new()
                }
            };

            let result = Version::new(
                f_ver,
                b_ver,
                git_com,
                String::new(),
                String::new(),
                String::new(),
            );

            println!("{result}");
        }
        VerSubcommand::Debug => {
            println!("The current version information");
        }
    }
}

fn get_versioned(app: &str) -> anyhow::Result<String> {
    // std::env!("CARGO_PKG_VERSION").to_string()
    let workspace_root = shell::find_worspace_root()?;
    let mut version: String = String::new();

    let path = path::Path::new(&workspace_root).read_dir()?;
    for dir in path {
        let dir = dir?;
        // println!("File: {}", dir.dir_name().to_string_lossy());
        if dir.file_name().eq(app) {
            for file in dir.path().read_dir()? {
                let file = file?;

                if file.file_name().eq("Cargo.toml") {
                    // println!("files: {}", file?.file_name().to_string_lossy());
                    let mut file_buf: Vec<u8> = vec![];

                    // Two directories down is the target file
                    let mut cargo = std::fs::File::open(
                        path::Path::new(&workspace_root)
                            .join(dir.file_name())
                            .join(file.file_name()),
                    )?;

                    cargo.read_to_end(&mut file_buf)?;

                    for line in file_buf.split(|lines| *lines == b'\n') {
                        // println!("{:#?}", String::from_utf8_lossy(line));

                        if String::from_utf8_lossy(line).contains("version") {
                            let versioned = line
                                .split(|letter| letter.eq(&b'='))
                                .next_back()
                                .expect("failed to iter the line");

                            version = String::from_utf8_lossy(versioned)
                                .trim()
                                .trim_matches('\"')
                                .to_string();

                            // println!(
                            // "\nVERSION: {:#?}\n",
                            // String::from_utf8_lossy(versioned).trim().trim_matches('\"')
                            // );
                            break; // The target line is at the top of the file
                        }
                    }
                }
            }
        }
    }

    Ok(version)
}

fn get_commit_hash() -> anyhow::Result<String> {
    let command = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

    let com_hash: String = String::from_utf8_lossy(&command.stdout).trim().to_string();

    Ok(com_hash)
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Frontend: {}\nBackend: {}\nCommit Hash: {}\nBuild Time: {}\nRustc: {}\nRustup Target: {}",  self.frontend, self.backend, self.commit_hash, self.build_time, self.rustc, self.rustup_target)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use rstest::rstest;

    use crate::commands::version_arg::get_commit_hash;

    use super::get_versioned;

    #[rstest]
    fn test_backend_version() {
        assert!(get_versioned("backend").unwrap().starts_with('0'));
    }

    #[rstest]
    fn test_frontend_version() {
        assert!((get_versioned("frontend")).unwrap().starts_with('0'));
    }

    #[rstest]
    fn test_git_hash() {
        assert!(get_commit_hash().unwrap().len().gt(&10));
    }
}
