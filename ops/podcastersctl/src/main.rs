use clap::Parser;
use podcastersctl::commands::{Podcastersctl, command_tree};

fn main() {
    let args = Podcastersctl::parse();

    command_tree::run(args);
}
