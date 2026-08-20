pub mod command_tree;
mod dev;
mod doctor;
mod mongo_arg;
mod redis_arg;
use clap::{Parser, Subcommand};

use crate::{
    backup, ci, config, deploy, diagnostics, incident, kubernetes, mongo, redis, smoke, version,
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Podcastersctl {
    #[clap(subcommand)]
    pub commands: PodcastersctlCommands,
}

#[derive(Debug, Subcommand)]
pub enum PodcastersctlCommands {
    /// Start and stop as necessary to diagnose
    Doctor,
    /// Project State
    Dev(diagnostics::DevState),

    /// `MongoDb` information
    Mongo(mongo::MongoState),

    /// `Redis` information
    Redis(redis::RedisState),

    /// Continuous Integration
    Ci(ci::CiState),

    /// Project configuration
    Config(config::ConfigState),

    /// Health Check
    Smoke(smoke::SmokeState),

    /// Deploy, Check, or Rollback the application
    Deploy(deploy::DeployState),

    /// Kubernetes commands
    Kubernetes(kubernetes::KubState),

    /// Backup various parts of the application
    Backup(backup::BackupState),

    /// Produce a diagnostics file
    Diagnostics(diagnostics::DiagState),

    /// Show the version of various dependencies
    Version(version::VerState),

    /// Produce a report of the current ``Production Assessment``
    Incident(incident::IncidentState),
}

#[cfg(test)]
mod tests {
    use crate::commands;
    use clap::Parser as _;
    use rstest::rstest;

    #[rstest]
    #[case(&["podcastersctl", "doctor"])]
    #[case(&["podcastersctl", "dev", "up"])]
    #[case(&["podcastersctl", "dev", "down"])]
    #[case(&["podcastersctl", "dev", "status"])]
    #[case(&["podcastersctl", "dev", "reset"])]
    #[case(&["podcastersctl", "mongo", "check"])]
    #[case(&["podcastersctl", "mongo", "status"])]
    #[case(&[
        "podcastersctl",
        "mongo",
        "reconcile"
    ])]
    #[case(&["podcastersctl", "redis", "check"])]
    #[case(&["podcastersctl", "redis", "status"])]
    #[case(&["podcastersctl", "ci", "verify"])]
    #[case(&[
        "podcastersctl",
        "ci",
        "integration"
    ])]
    #[case(&[
        "podcastersctl",
        "config",
        "validate"
    ])]
    #[case(&[
        "podcastersctl",
        "config",
        "show"
    ])]
    #[case(&[
        "podcastersctl",
        "deploy",
        "status"
    ])]
    #[case(&[
        "podcastersctl",
        "deploy",
        "staging"
    ])]
    #[case(&[
        "podcastersctl",
        "deploy",
        "production"
    ])]
    #[case(&[
        "podcastersctl",
        "deploy",
        "rollback"
    ])]
    #[case(&[
        "podcastersctl",
        "kubernetes",
        "status"
    ])]
    #[case(&[
        "podcastersctl",
        "kubernetes",
        "pods"
    ])]
    #[case(&[
        "podcastersctl",
        "kubernetes",
        "nodes"
    ])]
    #[case(&[
        "podcastersctl",
        "kubernetes",
        "events"
    ])]
    #[case(&[
        "podcastersctl",
        "backup",
        "create",
	"mongodb"
    ])]
    #[case(&[
        "podcastersctl",
        "backup",
        "create",
	"redis"
    ])]
    #[case(&[
        "podcastersctl",
        "backup",
        "create",
	"configuration"
    ])]
    #[case(&[
        "podcastersctl",
        "backup",
        "create",
	"all"
    ])]
    #[case(&[
        "podcastersctl",
        "backup",
        "verify"
    ])]
    #[case(&[
        "podcastersctl",
        "backup",
        "restore"
    ])]
    #[case(&[
        "podcastersctl",
        "diagnostics",
        "collect"
    ])]
    #[case(&[
        "podcastersctl",
        "version",
        "production"
    ])]
    #[case(&["podcastersctl", "incident", "asess"])]
    #[case(&["podcastersctl", "incident", "collect"])]
    #[case(&["podcastersctl", "incident", "compare"])]
    #[case(&["podcastersctl", "incident", "timeline"])]
    fn valid_command_line_parses(#[case] args: &[&str]) {
        let result = commands::Podcastersctl::try_parse_from(args);

        assert!(
            result.is_ok(),
            "Expected command to parse: {args:?}\n\
             Error: {result:#?}"
        );
    }

    #[rstest]
    #[case(&[
    "podcasterctl",
    "not-a-command"
])]
    #[case(&[
    "podcasterctl",
    "mongo",
    "not-a-command"
])]
    #[case(&[
    "podcasterctl",
    "deploy",
    "not-a-command"
])]
    #[case(&[
    "podcasterctl",
    "kubernetes",
    "not-a-command"
])]
    fn invalid_command_line_is_rejected(#[case] args: &[&str]) {
        let result = commands::Podcastersctl::try_parse_from(args);

        assert!(result.is_err(), "Expected command to fail: {args:?}");
    }
}
