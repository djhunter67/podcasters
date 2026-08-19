mod args;

use args::Podcasterctl;
use clap::Parser;

fn main() {
    let args = Podcasterctl::parse();

    match args.commands {
        args::PodcasterctlCommands::Doctor => {
            println!("Doctor called");
        }
        args::PodcasterctlCommands::Dev(cmd) => {
            // println!("The DOWN command called: {cmd:#?}");
            match cmd.state {
                args::StateSubcommand::Up => {
                    println!("Much to do about the 'UP' command");
                }
                args::StateSubcommand::Down => {
                    println!("Much to do about the 'DOWN' command");
                }
                args::StateSubcommand::Status => {
                    println!("Report the status and runtime of various dependencies");
                }
                args::StateSubcommand::Reset => {
                    println!("Turn off and then back on various dependencies");
                }
            }
        }
        args::PodcasterctlCommands::Database => {
            println!("The Database command executed");
        }
        args::PodcasterctlCommands::Ci => {
            println!("The Continuous Integration command called");
        }
        args::PodcasterctlCommands::Smoke => {
            println!("The Smoke command called");
        }
        args::PodcasterctlCommands::Diagnostics => {
            println!("The Diagnostics command called");
        }
        args::PodcasterctlCommands::Version => {
            println!("The version of the various dependencies");
        }
        args::PodcasterctlCommands::Incident => {
            println!("Parse the log for an ERROR or a PANIC and create a parseable report.");
        }
        args::PodcasterctlCommands::K8s => {
            println!("All kubernetes command relative to the deployment of the Podcaster");
        }
    }
}
