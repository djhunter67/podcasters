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

    /// `podcastersctl mongo status`
    /// `podcastersctl mongo check`
    /// `podcastersctl mongo reconcile`
    /// check could:
    ///     users
    ///         ✓ ``email`` unique index
    ///         ✓ ``created_at`` index podcasts
    ///         ✓ ``feed_url`` unique index
    ///         ✓ ``title`` text index episodes
    ///         ✓ ``podcast_id`` index
    ///         ✗ ``published_at`` index missing
    pub database: String,

    /// ``podcasterctl ci verify``
    pub ci: String,

    /// ``podcasterctls smoke --environment staging``
    ///     Initially check:
    ///         - Backend /health
    ///         - Frontend /health
    ///         - Mongo connectivity through backend
    ///         - Redis connectivity
    ///         - API version
    ///         - Build version
    pub smoke: String,

    /// ``podcasterctl diagnostics collect``
    /// produce something like: podcasters-diagnostics-2026-08-19T113211Z.tar.gz
    /// Containing non-secret information:
    ///     - Podcasters version.
    ///     - Git commit.
    ///     - Rust version.
    ///     - OS/kernel.
    ///     - Architecture.
    ///     - Kubernetes context name.
    ///     - Kubernetes pod status.
    ///     - deployment status.
    ///     - recent Podcasters logs.
    ///     - health-check output.
    ///     - Mongo server version.
    ///     - Redis server version.
    ///     - configuration keys with values redacted.
    ///     - resource usage.
    ///     - networking diagnostics.
    pub diagnostics: String,

    /// ``podcasterctl version production``
    /// {
    ///    "``version``": "0.1.0",
    ///    "``commit``": "768bc15",
    ///    "``built_at``": "2026-08-19T11:22:18Z",
    ///    "``rustc``": "nightly-2026-08-18",
    ///    "``target``": "x86_64-unknown-linux-gnu"
    /// }
    pub version: String,

    /// ``podcasterctl incident assess``
    ///PRODUCTION ASSESSMENT
    ///
    ///Backend
    ///  6 ``desired``
    ///  4 ``healthy``
    ///  2 ``CrashLoopBackOff``
    ///
    ///Frontend
    ///  4/4 ``healthy``
    ///
    ///MongoDB
    ///  ``healthy``
    ///  ``latency`` 11ms
    ///
    ///Redis
    ///  ``healthy``
    ///  ``hit rate`` 93%
    ///
    ///Feed workers
    ///  ``queue depth``: 187,122
    ///  ``oldest job``: 41m
    ///
    ///Primary finding:
    ///  ``feed-worker`` deployment is unhealthy
    ///
    ///Suggested next command:
    ///  ``podcastersctl kubernetes inspect feed-worker``
    pub incident: String,

    pub kubernetes: String,
}
