use crate::smoke::{self, SmokeSubcommand, Staging};

pub fn execute(smoke: &smoke::SmokeState) {
    match &smoke.smoke {
        SmokeSubcommand::Environment(environ) => match environ.staging {
            Staging::Debug => {
                println!("Debug environment chosen");
            }
            Staging::Production => {
                println!("Production environment chosen");
            }
        },
    }
}
