use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct VerState {
    /// Give every deployment an Identity
    #[clap(subcommand)]
    pub version: VerSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum VerSubcommand {
    /// Returns the version, git commit, build timestamp, Rust compiler version, build profile, and target
    Production,
}
