use crate::ci::{self, CiSubcommand};

pub fn execute(ci: &ci::CiState) {
    match ci.ci {
        CiSubcommand::Verify => {
            println!("Run the formats and what-not");
        }
        CiSubcommand::Integration => {
            println!("Run the integrations, not sure what this means, though.");
        }
    }
}
