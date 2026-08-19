use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct ConfigState {
    /// Check the `session` or the `clear` a cache layer session
    #[clap(subcommand)]
    pub config: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    /// Validate the configuration for the project
    Validate,
    /// Show the configuration for the project
    Show,
}
