use crate::config::{self, ConfigSubcommand};

#[allow(clippy::unnecessary_wraps)]
pub fn execute(config: &config::ConfigState) -> anyhow::Result<()> {
    match config.config {
        ConfigSubcommand::Validate => {
            println!("Validating the project configuration");
        }
        ConfigSubcommand::Show => {
            println!("Showing the configuration for the project");
        }
    }
    Ok(())
}
