use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct BackupState {
    #[clap(subcommand)]
    pub backup: BackupSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum BackupSubcommand {
    /// Tar the entire project and compress at the maximum compression
    Create(Creation),
    /// Copy and then uncompress, untar, and validate that the project builds and passes all test
    Verify,
    /// Restore all of the project, restore secrets, and deploy to the branch `backup-restore`
    Restore,
}

#[derive(Debug, Args)]
pub struct Creation {
    #[clap(subcommand)]
    pub create: Creator,
}

#[derive(Debug, Subcommand)]
pub enum Creator {
    /// Creat a backup of MongoDB
    Mongodb,
    /// Create a backup of Redis
    Redis,
    /// Backup all of the configuration
    Configuration,
    /// `MongoDb`, `Redis`, Application Config
    All,
}
