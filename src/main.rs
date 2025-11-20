mod app;
mod cli;
mod config;
mod core;

use clap::Parser;

use cli::{Cli, Commands};

use config::io::{load_config, save_config};

use core::Outcome;

use app::COMMAND_HEADER;
use app::add::handle_add;
use app::config_path::determine_config_path;
use app::r#move::handle_move;
use app::shell::determine_shell;

use core::sync::generate_alias_script_content;

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
    debug!("Using config path: {:?}", path);

    let shell = determine_shell();
    debug!("Determined shell: {:?}", shell);

    let mut config = load_config(path.as_ref()).expect("Failed to load configuration");
    debug!("Loaded configuration: {:?}", config);

    let result = match cli.command {
        // Add new alias or group
        Commands::Add(cmd) => handle_add(&mut config, cmd),
        Commands::Sync => Ok(Outcome::Command(generate_alias_script_content(&config))),
        Commands::Move(cmd) => handle_move(&mut config, cmd),
        _ => todo!("command not implemented yet"),
    };

    match result {
        Ok(Outcome::Command(msg)) => {
            debug!("Generated command output: {}", msg);
            save_config(&config, path.as_ref()).expect("Failed to save configuration");
            println!("{}\n{}", COMMAND_HEADER, msg); // Display the command header and generated command for the shell to load
        }
        Ok(Outcome::NoChanges) => println!("No changes were made."),
        Ok(Outcome::ConfigChanged) => {
            save_config(&config, path.as_ref()).expect("Failed to save configuration");
            debug!("New configuration saved.");
        }
        Err(_) => eprintln!("An error occurred."),
    }
}
