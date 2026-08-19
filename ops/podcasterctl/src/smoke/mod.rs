use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct SmokeState {
    #[clap(subcommand)]
    pub smoke: SmokeSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum SmokeSubcommand {
    /// Validate the configuration for the project
    /// - Backend `/health`
    /// - Frontend `/health`
    /// - Mongo connectivity through the backend
    /// - Redis connectivity
    /// - API version
    /// - Build version
    Environment(SmokeEnvironment),
}

#[derive(Debug, Args)]
pub struct SmokeEnvironment {
    #[clap(subcommand)]
    pub staging: Staging,
}

#[derive(Debug, Subcommand)]
pub enum Staging {
    Production,
    Debug,
}
