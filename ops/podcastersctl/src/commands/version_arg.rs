use std::{fmt, process::Command};

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
            let b_ver =
                shell::get_versioned("backend").unwrap_or_else(|err| format!("Error: {err:#?}"));
            let f_ver =
                shell::get_versioned("frontend").unwrap_or_else(|err| format!("Error: {err:#?}"));
            let git_com = match get_commit_hash() {
                Ok(val) => val,
                Err(err) => {
                    eprintln!("{err:#?}");
                    String::new()
                }
            };

            let rustc = get_rustc_version().unwrap_or_else(|err| format!("Error: {err:#?}"));

            let build_time = get_build_time().unwrap_or_else(|err| format!("Error: {err:#?}"));

            let result = Version::new(f_ver, b_ver, git_com, build_time, rustc, String::new());

            println!("{result}");
        }
        VerSubcommand::Debug => {
            println!("The current version information");
        }
    }
}

fn get_commit_hash() -> anyhow::Result<String> {
    let command = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

    let com_hash: String = String::from_utf8_lossy(&command.stdout).trim().to_string();

    Ok(com_hash)
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Frontend: {}\nBackend: {}\nCommit Hash: {}\nBuild Time: {}\nRustc: {}\nRustup Target: {}",
            self.frontend,
            self.backend,
            self.commit_hash,
            self.build_time,
            self.rustc,
            self.rustup_target
        )
    }
}

fn get_rustc_version() -> anyhow::Result<String> {
    let command = Command::new("rustc").arg("--version").output()?;

    let output = String::from_utf8_lossy(&command.stdout);

    // println!("Output: {output}");

    Ok(output.to_string())
}

fn get_build_time() -> anyhow::Result<String> {
    Ok(String::from("Yesterday"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {

    use rstest::rstest;

    use crate::commands::version_arg::get_commit_hash;

    #[rstest]
    fn test_git_hash() {
        assert!(get_commit_hash().unwrap().len().gt(&10));
    }
}
