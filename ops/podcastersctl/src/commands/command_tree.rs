use super::{
    Podcastersctl, backup_arg, ci_arg, config_arg, deploy_arg, dev_arg, diag_arg, incident_arg,
    kube_arg, mongo_arg, redis_arg, smoke_arg,
};

use crate::{
    commands::{PodcastersctlCommands, version_arg},
    doctor,
};

#[allow(clippy::too_many_lines)]
/// # Result
///
/// - Returns nothing, the functions print to the console
/// # Errors
///
/// - Various error bubble up from database, env, or directory parsing
pub async fn run(args: Podcastersctl) -> anyhow::Result<()> {
    match args.commands {
        PodcastersctlCommands::Doctor => {
            let _: () = match doctor::execute().await {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error in doctor command: {err}");
                }
            };
        }
        PodcastersctlCommands::Dev(cmd) => {
            let _: () = match dev_arg::execute(&cmd) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Mongo(mongo_arg) => {
            let _: () = match mongo_arg::execute(&mongo_arg) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Redis(redis_arg) => {
            let _: () = match redis_arg::execute(&redis_arg) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Ci(ci) => {
            let _: () = match ci_arg::execute(&ci) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Config(config) => {
            let _: () = match config_arg::execute(&config) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Smoke(smoke) => {
            let _: () = match smoke_arg::execute(&smoke) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Deploy(deploy) => {
            let _: () = match deploy_arg::execute(&deploy) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Kubernetes(kube) => {
            let _: () = match kube_arg::execute(&kube) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Backup(backup) => {
            let _: () = match backup_arg::execute(&backup) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Diagnostics(diag) => {
            let _: () = match diag_arg::execute(&diag) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Version(version) => {
            let _: () = match version_arg::execute(&version) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
        PodcastersctlCommands::Incident(incident) => {
            let _: () = match incident_arg::execute(&incident) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                }
            };
        }
    }
    Ok(())
}
