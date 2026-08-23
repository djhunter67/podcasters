use super::{
    Podcastersctl, backup_arg, ci_arg, config_arg, deploy_arg, dev_arg, diag_arg, incident_arg,
    kube_arg, mongo_arg, redis_arg, smoke_arg,
};

use crate::{
    commands::{PodcastersctlCommands, version_arg},
    doctor,
};

pub async fn run(args: Podcastersctl) -> anyhow::Result<()> {
    match args.commands {
        PodcastersctlCommands::Doctor => Ok(doctor::execute().await?),
        PodcastersctlCommands::Dev(cmd) => Ok(dev_arg::execute(&cmd)?),
        PodcastersctlCommands::Mongo(mongo_arg) => Ok(mongo_arg::execute(&mongo_arg)?),
        PodcastersctlCommands::Redis(redis_arg) => Ok(redis_arg::execute(&redis_arg)?),
        PodcastersctlCommands::Ci(ci) => Ok(ci_arg::execute(&ci)?),
        PodcastersctlCommands::Config(config) => Ok(config_arg::execute(&config)?),
        PodcastersctlCommands::Smoke(smoke) => Ok(smoke_arg::execute(&smoke)?),
        PodcastersctlCommands::Deploy(deploy) => Ok(deploy_arg::execute(&deploy)?),
        PodcastersctlCommands::Kubernetes(kube) => Ok(kube_arg::execute(&kube)?),
        PodcastersctlCommands::Backup(backup) => Ok(backup_arg::execute(&backup)?),
        PodcastersctlCommands::Diagnostics(diag) => Ok(diag_arg::execute(&diag)?),
        PodcastersctlCommands::Version(version) => Ok(version_arg::execute(&version)?),
        PodcastersctlCommands::Incident(incident) => Ok(incident_arg::execute(&incident)?),
    }
}
