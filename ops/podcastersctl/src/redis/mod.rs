use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct RedisState {
    /// Check the `session` or the `clear` a cache layer session
    #[clap(subcommand)]
    pub redis: RedisSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum RedisSubcommand {
    /// Presents information regarding the cache layer
    Check,
    /// Cache layer uptime, key count, memory, hit rate, and server version.
    Status,
    /// Shows the output of ```redis-cli KEYS \*``` for the session provided
    Keys(Session),
    /// Destructive command that employs explicit safeguards
    Clear(Integration),
}

#[derive(Debug, Args)]
pub struct Session {
    #[clap(subcommand)]
    pub session: RedisSession,
}

#[derive(Debug, Args)]
pub struct Integration {
    #[clap(subcommand)]
    pub integration: RedisIntegration,
}

#[derive(Debug, Subcommand)]
pub enum RedisSession {
    /// Get the keys of the `session` provided
    Session(SessionVal),
}

#[derive(Debug, Args)]
pub struct SessionVal {
    val: String,
}

#[derive(Debug, Subcommand)]
pub enum RedisIntegration {
    /// Remove the Integration test results
    IntegrationTest,
}
