mod cli;
mod config;

use clap::Parser;
use cli::Cli;

fn main() {
    let _cli = Cli::parse();
}
