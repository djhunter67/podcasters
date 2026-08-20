use crate::deploy::{self, DeploySubcommand};

pub fn execute(deploy: &deploy::DeployState) {
    match &deploy.deploy {
        DeploySubcommand::Status => {
            println!("Get the status of the mobile and web applications");
        }
        DeploySubcommand::Staging => {
            println!("Get the status of the staging branch or create it if it doesn't exist");
        }
        DeploySubcommand::Production => {
            println!("Deploy the production branch");
        }
        DeploySubcommand::Rollback => {
            println!("Rollback the currently deployed instance for all applications");
        }
    }
}
