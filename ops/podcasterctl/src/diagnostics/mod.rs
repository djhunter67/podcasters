use clap::{Args, Subcommand};

///$ podcastersctl doctor
///
/// Podcasters Environment
///
/// Rust
///   ✓ nightly-2026-08-18
///   ✓ rustfmt
///   ✓ clippy
///   ✓ rustc-codegen-cranelift
///
/// Docker
///   ✓ daemon reachable
///   ✓ current user may create containers
///
/// MongoDB
///   ✓ `mongodb://127.0.0.1:27017`
///   ✓ ping
///   ✓ database: `podcasters_test`
///
/// Redis
///   ✓ `redis://127.0.0.1:6379`
///   ✓ PONG
///
/// Kubernetes
///   ✓ context: homelab
///   ✓ cluster reachable
///   ✓ nodes: 3/3 Ready
///
/// Podcasters
///   ✓ backend configuration
///   ✓ frontend configuration
///   ✓ API health
///   ✓ frontend health
///
/// Overall: HEALTHY

#[derive(Debug, Args)]
pub struct DevState {
    /// ``podcastersctl dev up``
    #[clap(subcommand)]
    pub dev: DevSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DevSubcommand {
    /// Verify docker,
    /// start development MongoDB,
    /// Start development Redis,
    /// Wait for health checks,
    /// Verify settings,
    /// Start backend,
    /// Start frontend,
    Up,
    /// Stop the docker containers,
    /// Shutdown the frontend and backend servers
    Down,
    /// Current status base on the health check,
    /// Current status of docker,
    /// Docker uptime,
    /// Database uptime,
    /// Frontend server uptime
    /// Backend server uptime
    Status,
    /// Reset Docker,
    /// Reset Frontend
    /// Reset Backend
    /// Reset Database
    /// Reset Cache Layer
    Reset,
}

#[derive(Debug, Args)]
pub struct DiagState {
    /// Produce something like ```podcasters-diagnostics-2026-08-19T113211Z.tar.gz```
    #[clap(subcommand)]
    pub collect: DiagSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum DiagSubcommand {
    /// Containing non-secret information:
    ///
    ///   - Podcasters version.
    ///
    ///   - Git commit.
    ///
    ///   - Rust version.
    ///
    ///   - OS/kernel.
    ///
    ///   - Architecture.
    ///
    ///   - Kubernetes context name.
    ///
    ///   - Kubernetes pod status.
    ///
    ///   - deployment status.
    ///
    ///   - recent Podcasters logs.
    ///
    ///   - health-check output.
    ///
    ///   - Mongo server version.
    ///
    ///   - Redis server version.
    ///
    ///   - configuration keys with values redacted.
    ///
    ///   - resource usage.
    ///
    ///   - networking diagnostics.
    ///
    /// Explicitly exclude:
    ///
    ///   - passwords,
    ///
    ///   - tokens,
    ///
    ///   - connection-string credentials,
    ///
    ///   - cookies,
    ///
    ///   - signing keys.
    ///
    Collect,
}
