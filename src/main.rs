mod app;
mod cli;
mod config;
mod core;

use clap::Parser;

use cli::{Cli, Commands};

use config::io::load_config;

use core::Outcome;

use app::add::handle_add;
use app::config_path::determine_config_path;

use log::{LevelFilter, debug};

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

    let path = determine_config_path()
        .expect("Custom config path did not exist and user chose not to use it.");

    let mut config = load_config(path.as_ref()).expect("Failed to load configuration");
    debug!("Loaded configuration: {:?}", config);

    let result = match cli.command {
        // Add new alias or group
        Commands::Add(cmd) => handle_add(cmd, &mut config),
        _ => todo!("command not implemented yet"),
    };

    match result {
        Ok(Outcome::Command(msg)) => println!("Command needs to be run: {}", msg),
        Ok(Outcome::NoChanges) => println!("No changes were made."),
        Ok(Outcome::ConfigChanged) => println!("Configuration needs to be updated."),
        Err(_) => eprintln!("An error occurred."),
    }
}
