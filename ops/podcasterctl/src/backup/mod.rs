use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct BackupState {
    #[clap(subcommand)]
    pub backup: BackupSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum BackupSubcommand {
    /// Tar the entire project and compress at the maximum compression
    Create,
    /// Copy and then uncompress, untar, and validate that the project builds and passes all test
    Verify,
    /// Restore all of the project, restore secrets, and deploy to the branch `backup-restore`
    Restore,
}
