#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod app;
mod catalog;
mod cli;
mod core;

use clap::Parser;

use cli::{Cli, Commands};

use catalog::io::{catalog_path as resolve_catalog_path, load_catalog, save_catalog};

use catalog::types::AliasCatalog;
use core::Outcome;

use app::add::handle_add;
use app::disable::handle_disable;
use app::doctor::handle_doctor;
use app::edit::handle_edit;
use app::enable::handle_enable;
use app::file_path::determine_catalog_path;
use app::init::handle_init;
use app::list::handle_list;
use app::r#move::handle_move;
use app::remove::handle_remove;
use app::rename::handle_rename;
use app::sort::handle_sort;
use app::sync::{handle_shell_sync, handle_sync};

use app::shell::{DEFAULT_SHELL, determine_shell};

use log::{LevelFilter, debug};

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() {
    let cli = Cli::parse();

    // Determine log level based on CLI flags
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

    let mut catalog = AliasCatalog::new();
    let mut catalog_path = None;
    let mut shell = DEFAULT_SHELL;

    let is_doctor = matches!(&cli.command, Commands::Doctor(_));

    if !matches!(cli.command, Commands::Init(_)) {
        shell = determine_shell();
        debug!("Determined shell: {}", shell);

        catalog_path = determine_catalog_path()
            .expect("Custom catalog path did not exist and user chose not to use it.");
        debug!("Using catalog path: {:?}", catalog_path);

        let resolved_catalog_path = resolve_catalog_path(catalog_path.as_ref());
        catalog = match load_catalog(&resolved_catalog_path) {
            Ok(catalog) => catalog,
            Err(error) if is_doctor => {
                eprintln!(
                    "ERROR: Could not load catalog '{}': {error}",
                    resolved_catalog_path.display()
                );
                std::process::exit(1);
            }
            Err(error) => panic!("Failed to load catalog: {error}"),
        };
        debug!("Loaded catalog: {:?}", catalog);
    }

    let result = match cli.command {
        // Add new alias or group
        Commands::Add(cmd) => handle_add(&mut catalog, cmd, &shell),
        Commands::Remove(cmd) => handle_remove(&mut catalog, cmd, &shell),
        Commands::Move(cmd) => handle_move(&mut catalog, cmd),
        Commands::List(cmd) => handle_list(&catalog, cmd, &shell),
        Commands::Rename(cmd) => handle_rename(&mut catalog, cmd),
        Commands::Edit(cmd) => handle_edit(&mut catalog, cmd),
        Commands::Sort(cmd) => handle_sort(&mut catalog, cmd),
        Commands::Enable(cmd) => handle_enable(&mut catalog, cmd, &shell),
        Commands::Disable(cmd) => handle_disable(&mut catalog, cmd, &shell),
        Commands::Doctor(cmd) => handle_doctor(&catalog, cmd, &shell),
        Commands::Sync(cmd) => handle_sync(cmd),
        Commands::ShellSync(cmd) => {
            print!("{}", handle_shell_sync(&catalog, &shell, cmd));
            Ok(Outcome::NoChanges)
        }
        Commands::Init(cmd) => {
            let content = handle_init(cmd);
            debug!("Generated init script content");
            println!("{}", content);
            Ok(Outcome::NoChanges)
        }
    };

    match result {
        Ok(Outcome::NoChanges) => {
            debug!("No changes made to catalog or shell.");
        }
        Ok(Outcome::CatalogChanged) => {
            if save_catalog(&catalog, &resolve_catalog_path(catalog_path.as_ref())).is_err() {
                eprintln!("Failed to save updated catalog.");
                return;
            }
            debug!("New catalog saved.");
        }
        Err(_) => {
            debug!("An error occurred during command execution.");
            if is_doctor {
                std::process::exit(1);
            }
        }
    }
}
