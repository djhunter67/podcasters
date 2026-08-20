use crate::version::{self, VerSubcommand};

pub fn execute(version: &version::VerState) {
    match &version.version {
        VerSubcommand::Production => {
            println!("The production version information is to be provided");
        }
    }
}
