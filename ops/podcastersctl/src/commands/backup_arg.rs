use crate::backup::{self, BackupSubcommand, Creator};

pub fn execute(backup: &backup::BackupState) -> anyhow::Result<()> {
    match &backup.backup {
        BackupSubcommand::Create(to_create) => match to_create.create {
            Creator::Mongodb => {
                println!("Create the backup of the database");
            }
            Creator::Redis => {
                println!("Creating the backup of the cache layer");
            }
            Creator::Configuration => {
                println!("Creating the backup of the application configuration");
            }
            Creator::All => {
                println!("Backup and compress the database, cache layer and the configuration");
            }
        },
        BackupSubcommand::Verify => {
            println!(
                "Copy and then uncompress, untar, and validate that the project builds and passes all test"
            );
        }
        BackupSubcommand::Restore => {
            println!(
                "Restore all of the project, restore secrets, and deploy to the branch `backup-restore`"
            );
        }
    }

    Ok(())
}
