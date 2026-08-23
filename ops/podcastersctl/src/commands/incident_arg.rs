use crate::incident::{self, IncidentSubcommand};

pub fn execute(incident: &incident::IncidentState) -> anyhow::Result<()> {
    match &incident.incident {
        IncidentSubcommand::Asess => {
            println!("Asses the incident");
            // ExitCode::Success
        }
        IncidentSubcommand::Collect => {
            println!("Collect the error report");
            // ExitCode::Success
        }
        IncidentSubcommand::Compare => {
            println!("Compare the latest `incident` report w/ the report just prior");
            // ExitCode::Success
        }
        IncidentSubcommand::Timeline => {
            println!("Prepare and show the timeline of `incident` reports");
            // ExitCode::Success
        }
    }

    Ok(())
}
