use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct MongoState {
    #[clap(subcommand)]
    pub mongo: MongoSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MongoSubcommand {
    /// # Presents information in the following format:
    ///
    /// Users:
    ///
    ///   -  ✓ email unique index
    ///
    ///   -  ✓ `created_at` index
    ///
    /// Podcasts:
    ///
    ///    - ✓ `feed_url` unique index
    ///
    ///    - ✓ title text index
    ///
    /// Episodes:
    ///
    ///    - ✓ `podcast_id` index
    ///
    ///     - ✗ `published_at` index missing
    ///
    Check,
    /// Get the mongo status
    Status,
    /// Create whats missing in the db
    ///
    /// This command is the MongoDB equivalent of migration discipline before there are millions of documents
    Reconcile,
}
