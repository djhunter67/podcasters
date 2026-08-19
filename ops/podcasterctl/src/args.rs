use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Podcasterctl {
    /// Rust
    ///   - ✓ nightly-2026-08-18
    ///   - ✓ rustfmt
    ///   - ✓ clippy
    ///   - ✓ rustc-codegen-cranelift

    ///     `Docker`
    ///
    /// ✓ daemon reachable
    /// ✓ current user may create containers

    /// ``MongoDB``
    ///
    /// ✓ ``mongodb://127.0.0.1:27017``
    /// ✓ ping
    /// ✓ database: ``podcasters_test``
    /// Redis
    ///
    /// ✓ ``redis://127.0.0.1:6379``
    /// ✓ PONG
    /// Kubernetes
    ///
    /// ✓ context: homelab
    /// ✓ cluster reachable
    /// ✓ nodes: 3/3 Ready
    /// Podcasters
    ///
    /// ✓ backend configuration
    /// ✓ frontend configuration
    /// ✓ API health
    /// ✓ frontend health
    #[clap(subcommand)]
    pub doctor: PodcasterctlCommands,
}

#[derive(Debug, Subcommand)]
pub enum PodcasterctlCommands {
    /// Project State
    Dev(ProjectState),

    /// Database information
    Database,

    /// Continuous Integration
    Ci,

    /// Health Check
    Smoke,

    /// Produce a diagnostics file
    Diagnostics,

    /// Show the version of various dependencies
    Version,

    /// Produce a report of the current ``Production Assessment``
    Incident,

    /// Kubernetes commands
    K8s,
}

#[derive(Debug, Args)]
pub struct ProjectState {
    /// ``podcastersctl dev up``
    #[clap(subcommand)]
    state: StateSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum StateSubcommand {
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
