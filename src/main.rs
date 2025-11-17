mod cli;
mod config;

use clap::Parser;
use cli::Cli;
use config::io::load_config;

fn main() {
    let cli = Cli::parse();
    let config = load_config(&cli).expect("Failed to load configuration");
    println!("Configuration loaded: {:?}", config);
}
