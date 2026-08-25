use crate::mongo::{self, MongoSubcommand};

#[allow(clippy::unnecessary_wraps)]
pub fn execute(mongo_arg: &mongo::MongoState) -> anyhow::Result<()> {
    match &mongo_arg.mongo {
        MongoSubcommand::Check => {
            println!("Compare the live database w/ the application desired state");
        }
        MongoSubcommand::Status => {
            println!("Check the status the MongoDb instance");
        }
        MongoSubcommand::Reconcile => {
            println!("Create whats missing in the live instance of the Database");
        }
    }

    Ok(())
}
