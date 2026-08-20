mod backup;
mod ci;
mod commands;
mod config;
mod deploy;
mod diagnostics;
mod incident;
mod kubernetes;
mod mongo;
mod redis;
mod smoke;
mod version;

use backup::{BackupSubcommand, Creator};
use ci::CiSubcommand;
use clap::Parser;
use commands::{Podcastersctl, PodcastersctlCommands};
use config::ConfigSubcommand;
use deploy::DeploySubcommand;
use diagnostics::{DevSubcommand, DiagSubcommand};
use incident::IncidentSubcommand;
use kubernetes::KubSubcommand;
use mongo::MongoSubcommand;
use redis::{RedisIntegration, RedisSession, RedisSubcommand};
use smoke::{SmokeSubcommand, Staging};
use version::VerSubcommand;

#[allow(clippy::too_many_lines)]
fn main() {
    let args = Podcastersctl::parse();

    match args.commands {
        PodcastersctlCommands::Doctor => {
            println!("Doctor called");
        }
        PodcastersctlCommands::Dev(cmd) => {
            // println!("The DOWN command called: {cmd:#?}");
            match cmd.dev {
                DevSubcommand::Up => {
                    println!("Much to do about the 'UP' command");
                }
                DevSubcommand::Down => {
                    println!("Much to do about the 'DOWN' command");
                }
                DevSubcommand::Status => {
                    println!("Report the status and runtime of various dependencies");
                }
                DevSubcommand::Reset => {
                    println!("Turn off and then back on various dependencies");
                }
            }
        }
        PodcastersctlCommands::Mongo(mongo_arg) => match mongo_arg.mongo {
            MongoSubcommand::Check => {
                println!("Compare the live database w/ the application desired state");
            }
            MongoSubcommand::Status => {
                println!("Check the status the MongoDb instance");
            }
            MongoSubcommand::Reconcile => {
                println!("Create whats missing in the live instance of the Database");
            }
        },
        PodcastersctlCommands::Redis(redis_arg) => match redis_arg.redis {
            RedisSubcommand::Check => {
                println!("Compare the keys in the cache with the expected values");
            }
            RedisSubcommand::Status => {
                println!("Check the status of the cache layer; uptime, number of keys, version");
            }
            RedisSubcommand::Keys(key) => match key.session {
                RedisSession::Session(val) => {
                    println!("Get the keys for the session passed in?; Val: {val:#?}");
                }
            },
            RedisSubcommand::Clear(clear) => match clear.integration {
                RedisIntegration::IntegrationTest => {
                    println!("Clearing the integration test results");
                }
            },
        },
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
