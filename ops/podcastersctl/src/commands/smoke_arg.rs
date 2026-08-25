use crate::smoke::{self, SmokeSubcommand, Staging};

#[allow(clippy::unnecessary_wraps)]
pub fn execute(smoke: &smoke::SmokeState) -> anyhow::Result<()> {
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

    Ok(())
}
