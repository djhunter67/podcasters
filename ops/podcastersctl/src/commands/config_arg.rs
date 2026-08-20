use crate::config::{self, ConfigSubcommand};

pub fn execute(config: &config::ConfigState) {
    match config.config {
        ConfigSubcommand::Validate => {
            println!("Validating the project configuration");
        }
        ConfigSubcommand::Show => {
            println!("Showing the configuration for the project");
        }
    }
}
