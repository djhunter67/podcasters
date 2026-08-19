use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct DeployState {
    /// Check the `session` or the `clear` a cache layer session
    #[clap(subcommand)]
    pub deploy: DeploySubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DeploySubcommand {
    /// Status of the curent deployment
    Status,
    /// Checks if `staging` is passing all deployment criteria
    Staging,
    /// Checks if `production` is passing all deployment criteria
    Production,
    /// Rollback the currently deployed instance
    Rollback,
}
