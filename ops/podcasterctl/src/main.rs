mod ci;
mod commands;
mod diagnostics;
mod version;

use ci::CiSubcommand;
use clap::Parser;
use commands::{Podcasterctl, PodcasterctlCommands};
use diagnostics::{DevSubcommand, DiagSubcommand};
use version::VerSubcommand;

fn main() {
    let args = Podcasterctl::parse();

    match args.commands {
        PodcasterctlCommands::Doctor => {
            println!("Doctor called");
        }
        PodcasterctlCommands::Dev(cmd) => {
            // println!("The DOWN command called: {cmd:#?}");
            match cmd.dev {
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
            }
        }
        PodcasterctlCommands::Database => {
            println!("The Database command executed");
        }
        PodcasterctlCommands::Ci(ci) => {
            println!("The Continuous Integration command called");
            match ci.ci {
                CiSubcommand::Verify => {
                    println!("Run the formats and what-not");
                }
                CiSubcommand::Integration => {
                    println!("Run the integrations, not sure what this means, though.");
                }
            }
        }
        PodcasterctlCommands::Smoke => {
            println!("The Smoke command called");
        }
        PodcasterctlCommands::Diagnostics(diag) => match diag.collect {
            DiagSubcommand::Collect => {
                println!("The diagnostics collection has been kicked off");
            }
        },
        PodcasterctlCommands::Version(version) => {
            println!("The version of the various dependencies");
            match version.version {
                VerSubcommand::Production => {
                    println!("The production version information is to be provided");
                }
            }
        }
        PodcasterctlCommands::Incident => {
            println!("Parse the log for an ERROR or a PANIC and create a parseable report.");
        }
        PodcasterctlCommands::K8s => {
            println!("All kubernetes command relative to the deployment of the Podcaster");
        }
    }
}
