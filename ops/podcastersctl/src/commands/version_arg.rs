use std::{fmt, process::Command};

use crate::version::{self, VerSubcommand};

#[derive(Debug)]
struct Version {
    workspace: String,
    frontend: String,
    backend: String,
    commit_hash: String,
    build_time: String,
    rustc: String,
    rustup_target: String,
}

impl Version {
    pub const fn new(
        workspace: String,
        frontend: String,
        backend: String,
        commit_hash: String,
        build_time: String,
        rustc: String,
        rustup_target: String,
    ) -> Self {
        Self {
            workspace,
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
            println!("The production version information is to be provided");
            let ver = get_crate_version();
            let git_com = match get_commit_hash() {
                Ok(val) => val,
                Err(err) => {
                    eprintln!("{err:#?}");
                    String::new()
                }
            };

            let result = Version::new(
                ver,
                String::new(),
                String::new(),
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

fn get_crate_version() -> String {
    std::env!("CARGO_PKG_VERSION").to_string()
}

fn get_commit_hash() -> anyhow::Result<String> {
    let command = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

    let com_hash: String = String::from_utf8_lossy(&command.stdout).trim().to_string();

    Ok(com_hash)
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Workspace: {}\nFrontend: {}\nBackend: {}\nCommit Hash: {}\nBuild Time: {}\nRustc: {}\nRustup Target: {}", self.workspace, self.frontend, self.backend, self.commit_hash, self.build_time, self.rustc, self.rustup_target)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use rstest::rstest;

    use crate::commands::version_arg::get_commit_hash;

    use super::get_crate_version;

    #[rstest]
    fn test_crate_version() {
        assert!(get_crate_version().starts_with('0'));
    }

    #[rstest]
    fn test_git_hash() {
        assert!(get_commit_hash().unwrap().len().gt(&10));
    }
}
