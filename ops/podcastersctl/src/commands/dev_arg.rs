use crate::diagnostics::{self, DevSubcommand};

pub fn execute(cmd: &diagnostics::DevState) -> anyhow::Result<()> {
    match &cmd.dev {
        DevSubcommand::Up => {
            println!("Much to do about the 'UP' command");
        }
        DevSubcommand::Down => {
            println!("Much to do about the 'DOWN' command");
        }
        DevSubcommand::Status => {
            println!("Report the status and runtime of various dependencies");
        }
        DevSubcommand::Reset => {
            println!("Turn off and then back on various dependencies");
        }
    };
    Ok(())
}
