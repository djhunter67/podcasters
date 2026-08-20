use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct MongoState {
    #[clap(subcommand)]
    pub mongo: MongoSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MongoSubcommand {
    /// Will show document counts
    Check,
    /// Get the mongo status
    Status,
    /// Create whats missing in the db
    ///
    /// This command is the MongoDB equivalent of migration discipline before there are millions of documents
    Reconcile,
}
