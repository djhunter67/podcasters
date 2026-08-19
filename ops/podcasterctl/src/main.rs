mod args;

use args::Podcasterctl;
use clap::Parser;

fn main() {
    let args = Podcasterctl::parse();

    println!("{args:#?}");
}
