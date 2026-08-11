#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod app;
mod catalog;
mod cli;
mod config;
mod core;

use clap::Parser;

use cli::interaction::InteractionMode;
use cli::{Cli, Commands};
use config::load_config;

use catalog::io::{catalog_path as resolve_catalog_path, load_catalog, save_catalog};

use catalog::types::AliasCatalog;
use core::Outcome;

use app::CommandOutcome;
use app::add::handle_add;
use app::disable::handle_disable;
use app::doctor::handle_doctor;
use app::edit::handle_edit;
use app::enable::handle_enable;
use app::file_path::determine_catalog_path;
use app::import::handle_import;
use app::init::handle_init;
use app::list::handle_list;
use app::remove::handle_remove;
use app::rename::handle_rename;
use app::sync::{handle_shell_sync, handle_sync};

use app::shell::{DEFAULT_SHELL, determine_shell};

use log::{LevelFilter, debug};

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() {
    let cli = Cli::parse();
    if let Err(error) = cli.validate_prompt_controls() {
        error.exit();
    }
    let quiet = cli.quiet;
    let interaction_mode = if cli.force {
        InteractionMode::Force
    } else if cli.no_input {
        InteractionMode::NoInput
    } else {
        InteractionMode::Interactive
    };

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

    let config = match load_config() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("ERROR: {error:#}");
            std::process::exit(1);
        }
    };
    let colors_enabled = cli.color.unwrap_or(config.color).enabled();

    let mut catalog = AliasCatalog::new();
    let mut catalog_path = None;
    let mut shell = DEFAULT_SHELL;

    let is_doctor = matches!(&cli.command, Commands::Doctor(_));

    if !matches!(cli.command, Commands::Init(_)) {
        shell = determine_shell();
        debug!("Determined shell: {}", shell);

        catalog_path = determine_catalog_path(interaction_mode)
            .expect("Custom catalog path did not exist and user chose not to use it.");
        debug!("Using catalog path: {:?}", catalog_path);

        let resolved_catalog_path = resolve_catalog_path(catalog_path.as_ref());
        catalog = match load_catalog(&resolved_catalog_path) {
            Ok(catalog) => catalog,
            Err(error) => {
                eprintln!(
                    "ERROR: Could not load catalog '{}': {error}",
                    resolved_catalog_path.display()
                );
                std::process::exit(1);
            }
        };
        debug!("Loaded catalog: {:?}", catalog);
    }

    let result = match cli.command {
        Commands::Add(cmd) => {
            handle_add(&mut catalog, cmd, &shell, interaction_mode).map(CommandOutcome::from)
        }
        Commands::Remove(cmd) => handle_remove(&mut catalog, cmd, interaction_mode),
        Commands::List(cmd) => {
            handle_list(&catalog, cmd, &shell, &config, colors_enabled).map(CommandOutcome::from)
        }
        Commands::Rename(cmd) => handle_rename(&mut catalog, cmd),
        Commands::Edit(cmd) => handle_edit(&mut catalog, cmd, &shell).map(CommandOutcome::from),
        Commands::Import(cmd) => handle_import(&mut catalog, cmd, interaction_mode, cli.force),
        Commands::Enable(cmd) => handle_enable(&mut catalog, cmd),
        Commands::Disable(cmd) => handle_disable(&mut catalog, cmd),
        Commands::Doctor(_) => handle_doctor(&catalog, &shell, quiet).map(CommandOutcome::from),
        Commands::Sync(_) => handle_sync().map(CommandOutcome::from),
        Commands::ShellSync(cmd) => {
            print!(
                "{}",
                handle_shell_sync(&catalog, &shell, cmd, interaction_mode)
            );
            Ok(CommandOutcome::from(Outcome::NoChanges))
        }
        Commands::Init(cmd) => {
            let content = handle_init(cmd);
            debug!("Generated init script content");
            println!("{}", content);
            Ok(CommandOutcome::from(Outcome::NoChanges))
        }
    };

    match result {
        Ok(CommandOutcome { outcome, message }) => {
            match outcome {
                Outcome::NoChanges => debug!("No changes made to catalog or shell."),
                Outcome::CatalogChanged => {
                    if save_catalog(&catalog, &resolve_catalog_path(catalog_path.as_ref())).is_err()
                    {
                        eprintln!("Failed to save updated catalog.");
                        return;
                    }
                    debug!("New catalog saved.");
                }
            }

            if let Some(message) = message
                && !quiet
            {
                println!("{message}");
            }
        }
        Err(error) => {
            debug!("An error occurred during command execution.");
            if !is_doctor {
                eprintln!("ERROR: {error}");
            }
            std::process::exit(1);
        }
    }
}
