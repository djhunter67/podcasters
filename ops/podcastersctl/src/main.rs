use clap::Parser;
use podcastersctl::commands::{Podcastersctl, command_tree};

#[tokio::main]
async fn main() {
    let args = Podcastersctl::parse();

    match command_tree::run(args).await {
        Ok(()) => (),
        Err(err) => eprint!("{err:#?}"),
    };
}
