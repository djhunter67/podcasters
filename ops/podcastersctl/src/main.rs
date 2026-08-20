mod backup;
mod ci;
mod commands;
mod config;
mod deploy;
mod diagnostics;
mod incident;
mod kubernetes;
mod mongo;
mod redis;
mod smoke;
mod version;

use clap::Parser;
use commands::{Podcastersctl, command_tree};

fn main() {
    let args = Podcastersctl::parse();

    command_tree::run(args);
}
