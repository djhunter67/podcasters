use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct IncidentState {
    /// Check the `session` or the `clear` a cache layer session
    #[clap(subcommand)]
    pub incident: IncidentSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum IncidentSubcommand {
    /// Assess the incident
    Asess,
    /// Collect the error report
    Collect,
    /// Compare the latest `incident` report w/ the report just prior
    Compare,
    /// Prepare and show the timeline of `incident` reports
    Timeline,
}
