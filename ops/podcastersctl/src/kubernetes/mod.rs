use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct KubState {
    #[clap(subcommand)]
    pub kubernetes: KubSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum KubSubcommand {
    /// Status of the curent deployment
    Status,
    /// Shows the output `kubectl get pods -A`
    Pods,
    /// Shows the output `kubectl get nodes -A`
    Nodes,
    /// Get the events of the cluster
    Events,
    /// Inspection of the command passed in
    Inspect(Inspector),
}

#[derive(Debug, Args)]
pub struct Inspector {
    pub inspect: String,
}
