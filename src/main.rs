mod cli;
mod config;
mod core;

use clap::Parser;
use cli::Cli;
use config::io::load_config;
use core::add::add_alias;

fn main() {
    let cli = Cli::parse();
    let config = load_config(cli.config.as_ref()).expect("Failed to load configuration");
    println!("Configuration loaded: {:?}", config);
}
