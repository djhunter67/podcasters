use crate::kubernetes::{self, KubSubcommand};

pub fn execute(kube: &kubernetes::KubState) {
    match &kube.kubernetes {
        KubSubcommand::Status => {
            println!("Get the status of the cluster");
        }
        KubSubcommand::Pods => {
            println!("Execute and show the output of `kubectl get pods -A`");
        }
        KubSubcommand::Nodes => {
            println!("Execute and show the output of `kubectl get nodes -A`");
        }
        KubSubcommand::Events => {
            println!("Get the latest events of the cluster");
        }
        KubSubcommand::Inspect(val) => {
            println!("The val passed in: {}", val.inspect);
        }
    }
}
