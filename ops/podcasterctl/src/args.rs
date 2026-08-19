use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Podcasterctl {
    /// Rust
    /// ✓ nightly-2026-08-18
    /// ✓ rustfmt
    /// ✓ clippy
    /// ✓ rustc-codegen-cranelift
    /// Docker
    ///
    /// ✓ daemon reachable
    /// ✓ current user may create containers
    /// MongoDB
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
    pub doctor: String,

    /// ``podcastersctl dev up``
    /// ``podcastersctl dev down``
    /// ``podcastersctl dev status``
    /// ``podcastersctl dev reset``
    pub dev: String,
}
