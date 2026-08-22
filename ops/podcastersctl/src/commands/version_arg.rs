use std::{fmt, process::Command};

use shared::shell;

use crate::version::{self, VerSubcommand};

#[derive(Debug)]
struct Version {
    frontend: String,
    backend: String,
    models: String,
    commit_hash: String,
    front_build_time: String,
    back_build_time: String,
    models_build_time: String,
    rustc: String,
    rustup_target: String,
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
            let m_ver =
                shell::get_versioned("models").unwrap_or_else(|err| format!("Error: {err:#?}"));
            let git_com = match get_commit_hash() {
                Ok(val) => val,
                Err(err) => {
                    eprintln!("{err:#?}");
                    String::new()
                }
            };

            let rustc = get_rustc_version().unwrap_or_else(|err| format!("Error: {err:#?}"));

            let front_build_time =
                get_build_time(false, "frontend").unwrap_or_else(|err| format!("Error: {err:#?}"));

            let back_build_time =
                get_build_time(false, "backend").unwrap_or_else(|err| format!("Error: {err:#?}"));

            let models_build_time =
                get_build_time(false, "libmodels.d") // since a bin wont be built
                    .unwrap_or_else(|err| format!("Error: {err:#?}"));

            let rustup_tgt = get_rustup_tgt().unwrap_or_else(|err| format!("Error: {err:#?}"));

            let result = Version {
                frontend: f_ver,
                backend: b_ver,
                models: m_ver,
                commit_hash: git_com,
                front_build_time,
                back_build_time,
                models_build_time,
                rustc,
                rustup_target: rustup_tgt,
            };

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
            "Frontend: {}\nBackend: {}\nModels: {}\nCommit Hash: {}\nFrontend Build Time: {}\nBackend Build Time: {}\nModels Build Time: {}\nRustc: {}\nRustup Target: {}",
            self.frontend,
            self.backend,
            self.models,
            self.commit_hash,
            self.front_build_time,
            self.back_build_time,
            self.models_build_time,
            self.rustc,
            self.rustup_target
        )
    }
}

fn get_rustc_version() -> anyhow::Result<String> {
    let command = Command::new("rustc").arg("--version").output()?;

    let output = String::from_utf8_lossy(&command.stdout);

    // println!("Output: {output}");

    Ok(output.trim().to_string())
}

fn get_build_time(prod: bool, app: &str) -> anyhow::Result<String> {
    let root = shell::find_worspace_root()?;
    let target = if prod { "release" } else { "debug" };

    let command = Command::new("ls")
        .args(["-ght", format!("target/{target}").as_str()])
        .current_dir(root)
        .output()?;

    let result = String::from_utf8_lossy(&command.stdout);

    for line in result.lines() {
        if line.split(' ').next_back().expect("Faild to split").eq(app) {
            let target_line: Vec<String> = line
                .split(' ')
                .map(std::string::ToString::to_string)
                .collect();

            let target_line = target_line.iter().rev().take(4).collect::<Vec<&String>>();

            let target_line = target_line
                .iter()
                .skip(1)
                .take(3)
                .rev()
                .collect::<Vec<&&String>>();

            let target_line: String = target_line
                .iter()
                .map(|val| val.as_str())
                .collect::<Vec<&str>>()
                .iter()
                .map(|val| val.to_string() + &String::from(" "))
                .collect::<Vec<String>>()
                .concat();

            return Ok(target_line);
        }
    }
    Ok(String::from("Unknown"))
}

fn get_rustup_tgt() -> anyhow::Result<String> {
    let command = Command::new("rustup")
        .args(["show", "active-toolchain"])
        .output()?;

    let result = String::from_utf8_lossy(&command.stdout)
        .split(' ')
        .next()
        .unwrap_or("Error")
        .to_string();

    Ok(result)
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
