#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod app;
mod cli;
mod config;
mod core;

use clap::Parser;
use std::path::PathBuf;

use cli::{Cli, Commands};

use config::io::{load_config, save_config};

use config::types::Config;
use core::Outcome;

use app::add::handle_add;
use app::config_path::determine_config_path;
use app::edit::handle_edit;
use app::enable::handle_enable;
use app::init::handle_init;
use app::list::handle_list;
use app::r#move::handle_move;
use app::remove::handle_remove;
use app::rename::handle_rename;
use app::sort::handle_sort;
use core::sync::generate_alias_script_content;

use app::shell::{DEFAULT_SHELL, determine_shell, send_alias_deltas_to_shell};

use log::{LevelFilter, debug};

#[cfg_attr(coverage_nightly, coverage(off))]
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
        .format_timestamp(None)
        .format_target(false)
        .filter_level(level)
        .parse_default_env()
        .init();

    let mut config = Config::new();
    let mut path: Option<PathBuf> = None;
    let mut shell = DEFAULT_SHELL;

    if !matches!(cli.command, Commands::Init(_)) {
        shell = determine_shell();
        debug!("Determined shell: {}", shell);

        path = determine_config_path()
            .expect("Custom config path did not exist and user chose not to use it.");
        debug!("Using config path: {:?}", path);

        config = load_config(path.as_ref()).expect("Failed to load configuration");
        debug!("Loaded configuration: {:?}", config);
    }

    let result = match cli.command {
        // Add new alias or group
        Commands::Add(cmd) => handle_add(&mut config, cmd, &shell),
        Commands::Sync => Ok(Outcome::Command(generate_alias_script_content(
            &config, shell,
        ))),
        Commands::Move(cmd) => handle_move(&mut config, cmd),
        Commands::List(cmd) => handle_list(&config, cmd, &shell),
        Commands::Remove(cmd) => handle_remove(&mut config, cmd, &shell),
        Commands::Rename(cmd) => handle_rename(&mut config, cmd),
        Commands::Edit(cmd) => handle_edit(&mut config, cmd),
        Commands::Sort(cmd) => handle_sort(&mut config, cmd),
        Commands::Enable(cmd) => handle_enable(&mut config, cmd, &shell),
        Commands::Init(cmd) => {
            let content = handle_init(cmd);
            debug!("Generated init script content");
            println!("{}", content);
            Ok(Outcome::NoChanges)
        }
        _ => todo!("command not implemented yet"),
    };

    match result {
        Ok(Outcome::Command(msg)) => {
            debug!("Generated command output: {}", msg);
            save_config(&config, path.as_ref()).expect("Failed to save configuration");
            send_alias_deltas_to_shell(&msg);
        }
        Ok(Outcome::NoChanges) => {
            debug!("No changes made to configuration or shell.");
        }
        Ok(Outcome::ConfigChanged) => {
            if save_config(&config, path.as_ref()).is_err() {
                eprintln!("Failed to save updated configuration.");
                return;
            }
            debug!("New configuration saved.");
        }
        Err(_) => debug!("An error occurred during command execution."),
    }
}
