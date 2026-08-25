use crate::diagnostics::{self, DiagSubcommand};

#[allow(clippy::unnecessary_wraps)]
pub fn execute(diag: &diagnostics::DiagState) -> anyhow::Result<()> {
    match &diag.collect {
        DiagSubcommand::Collect => {
            println!("The diagnostics collection has been kicked off");
        }
    }

    Ok(())
}
