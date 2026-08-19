use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct CiState {
    /// ``podcastersctl dev up``
    #[clap(subcommand)]
    pub ci: CiSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CiSubcommand {
    /// Runs [format, clippy, workspace check, unit tests, integration tests, Mongo health, Redis health, backend smoke test, frontend smoke test]
    Verify,
    /// TBD
    Integration,
}
