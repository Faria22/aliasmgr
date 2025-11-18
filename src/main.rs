mod cli;
mod config;
mod core;

use clap::Parser;
use cli::Cli;
use config::io::load_config;
use core::add::add_alias;
use log::LevelFilter;
use log::debug;

fn main() {
    let cli = Cli::parse();

    let level = if cli.quiet {
        LevelFilter::Error
    } else if cli.verbose {
        LevelFilter::Info
    } else if cli.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    };

    env_logger::Builder::new()
        .filter_level(level)
        .parse_default_env()
        .init();

    let config = load_config(cli.config.as_ref()).expect("Failed to load configuration");
    debug!("Configuration loaded: {:?}", config);
}
