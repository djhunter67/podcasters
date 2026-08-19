use clap::{Parser, Subcommand};

use crate::{ci, diagnostics, version};

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
    pub commands: PodcasterctlCommands,
}

#[derive(Debug, Subcommand)]
pub enum PodcasterctlCommands {
    /// Start and stop as necessary to diagnose
    Doctor,
    /// Project State
    Dev(diagnostics::DevState),

    /// Database information
    Database,

    /// Continuous Integration
    Ci(ci::CiState),

    /// Health Check
    Smoke,

    /// Produce a diagnostics file
    Diagnostics(diagnostics::DiagState),

    /// Show the version of various dependencies
    Version(version::VerState),

    /// Produce a report of the current ``Production Assessment``
    Incident,

    /// Kubernetes commands
    K8s,
}
