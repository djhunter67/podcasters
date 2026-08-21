use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct VerState {
    #[clap(subcommand)]
    pub version: VerSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum VerSubcommand {
    /// ## Version
    /// ## git commit
    /// ## build timestamp
    /// ## Rust compiler version
    /// ## build profile
    /// ## target
    /// ### For Production
    Production,
    /// ## Version
    /// ## git commit
    /// ## build timestamp
    /// ## Rust compiler version
    /// ## build profile
    /// ## target
    /// ### for Debug
    Debug,
}
