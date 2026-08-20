use super::{
    Podcastersctl, backup_arg, ci_arg, config_arg, deploy_arg, dev_arg, diag_arg, doctor_arg,
    incident_arg, kube_arg, mongo_arg, redis_arg, smoke_arg, version_arg,
};

use crate::commands::PodcastersctlCommands;

pub fn run(args: Podcastersctl) {
    match args.commands {
        PodcastersctlCommands::Doctor => doctor_arg::execute(),
        PodcastersctlCommands::Dev(cmd) => dev_arg::execute(&cmd),
        PodcastersctlCommands::Mongo(mongo_arg) => mongo_arg::execute(&mongo_arg),
        PodcastersctlCommands::Redis(redis_arg) => redis_arg::execute(&redis_arg),
        PodcastersctlCommands::Ci(ci) => ci_arg::execute(&ci),
        PodcastersctlCommands::Config(config) => config_arg::execute(&config),
        PodcastersctlCommands::Smoke(smoke) => smoke_arg::execute(&smoke),
        PodcastersctlCommands::Deploy(deploy) => deploy_arg::execute(&deploy),
        PodcastersctlCommands::Kubernetes(kube) => kube_arg::execute(&kube),
        PodcastersctlCommands::Backup(backup) => backup_arg::execute(&backup),
        PodcastersctlCommands::Diagnostics(diag) => diag_arg::execute(&diag),
        PodcastersctlCommands::Version(version) => version_arg::execute(&version),
        PodcastersctlCommands::Incident(incident) => incident_arg::execute(&incident),
    }
}
