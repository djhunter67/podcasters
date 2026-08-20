use super::{Podcastersctl, dev, doctor, mongo_arg, redis_arg};

use crate::backup::{BackupSubcommand, Creator};
use crate::ci::CiSubcommand;
use crate::commands::PodcastersctlCommands;
use crate::config::ConfigSubcommand;
use crate::deploy::DeploySubcommand;
use crate::diagnostics::DiagSubcommand;
use crate::incident::IncidentSubcommand;
use crate::kubernetes::KubSubcommand;
use crate::smoke::{SmokeSubcommand, Staging};
use crate::version::VerSubcommand;

#[allow(clippy::too_many_lines)]
pub fn run(args: Podcastersctl) {
    match args.commands {
        PodcastersctlCommands::Doctor => {
            doctor::execute();
        }
        PodcastersctlCommands::Dev(cmd) => dev::execute(cmd),
        PodcastersctlCommands::Mongo(mongo_arg) => mongo_arg::execute(mongo_arg),
        PodcastersctlCommands::Redis(redis_arg) => redis_arg::execute(redis_arg),
        PodcastersctlCommands::Ci(ci) => {
            println!("The Continuous Integration command called");
            match ci.ci {
                CiSubcommand::Verify => {
                    println!("Run the formats and what-not");
                }
                CiSubcommand::Integration => {
                    println!("Run the integrations, not sure what this means, though.");
                }
            }
        }
        PodcastersctlCommands::Config(config) => match config.config {
            ConfigSubcommand::Validate => {
                println!("Validating the project configuration");
            }
            ConfigSubcommand::Show => {
                println!("Showing the configuration for the project");
            }
        },
        PodcastersctlCommands::Smoke(smoke) => match smoke.smoke {
            SmokeSubcommand::Environment(environ) => match environ.staging {
                Staging::Debug => {
                    println!("Debug environment chosen");
                }
                Staging::Production => {
                    println!("Production environment chosen");
                }
            },
        },
        PodcastersctlCommands::Deploy(deploy) => match deploy.deploy {
            DeploySubcommand::Status => {
                println!("Get the status of the mobile and web applications");
            }
            DeploySubcommand::Staging => {
                println!("Get the status of the staging branch or create it if it doesn't exist");
            }
            DeploySubcommand::Production => {
                println!("Deploy the production branch");
            }
            DeploySubcommand::Rollback => {
                println!("Rollback the currently deployed instance for all applications");
            }
        },
        PodcastersctlCommands::Kubernetes(kube) => match kube.kubernetes {
            KubSubcommand::Status => {
                println!("Get the status of the cluster");
            }
            KubSubcommand::Pods => {
                println!("Execute and show the output of `kubectl get pods -A`");
            }
            KubSubcommand::Nodes => {
                println!("Execute and show the output of `kubectl get nodes -A`");
            }
            KubSubcommand::Events => {
                println!("Get the latest events of the cluster");
            }
            KubSubcommand::Inspect(val) => {
                println!("The val passed in: {}", val.inspect);
            }
        },
        PodcastersctlCommands::Backup(backup) => match backup.backup {
            BackupSubcommand::Create(to_create) => match to_create.create {
                Creator::Mongodb => {
                    println!("Create the backup of the database");
                }
                Creator::Redis => {
                    println!("Creating the backup of the cache layer");
                }
                Creator::Configuration => {
                    println!("Creating the backup of the application configuration");
                }
                Creator::All => {
                    println!("Backup and compress the database, cache layer and the configuration");
                }
            },
            BackupSubcommand::Verify => {
                println!(
                    "Copy and then uncompress, untar, and validate that the project builds and passes all test"
                );
            }
            BackupSubcommand::Restore => {
                println!(
                    "Restore all of the project, restore secrets, and deploy to the branch `backup-restore`"
                );
            }
        },
        PodcastersctlCommands::Diagnostics(diag) => match diag.collect {
            DiagSubcommand::Collect => {
                println!("The diagnostics collection has been kicked off");
            }
        },
        PodcastersctlCommands::Version(version) => {
            println!("The version of the various dependencies");
            match version.version {
                VerSubcommand::Production => {
                    println!("The production version information is to be provided");
                }
            }
        }
        PodcastersctlCommands::Incident(incident) => match incident.incident {
            IncidentSubcommand::Asess => {
                println!("Asses the incident");
            }
            IncidentSubcommand::Collect => {
                println!("Collect the error report");
            }
            IncidentSubcommand::Compare => {
                println!("Compare the latest `incident` report w/ the report just prior");
            }
            IncidentSubcommand::Timeline => {
                println!("Prepare and show the timeline of `incident` reports");
            }
        },
    }
}
