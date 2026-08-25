use crate::ci::{self, CiSubcommand};

#[allow(clippy::unnecessary_wraps)]
pub fn execute(ci: &ci::CiState) -> anyhow::Result<()> {
    match ci.ci {
        CiSubcommand::Verify => {
            println!("Run the formats and what-not");
        }
        CiSubcommand::Integration => {
            println!("Run the integrations, not sure what this means, though.");
        }
    }

    Ok(())
}
