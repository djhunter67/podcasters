use crate::diagnostics::{self, DiagSubcommand};

pub fn execute(diag: &diagnostics::DiagState) {
    match &diag.collect {
        DiagSubcommand::Collect => {
            println!("The diagnostics collection has been kicked off");
        }
    }
}
