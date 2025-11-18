mod cli;
mod config;
mod core;

use clap::Parser;
use cli::{Cli, Commands, add::AddGroupCommands};
use config::io::load_config;
use core::add::{add_alias, add_group};
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

    let mut config = load_config(cli.config.as_ref()).expect("Failed to load configuration");
    debug!("Configuration loaded: {:?}", config);

    match cli.command {
        Commands::Add(cmd) => match cmd.subcommand {
            Some(AddGroupCommands::Group { name, disabled }) => {
                add_group(&mut config, &name, !disabled)
            }
            None => {
                if let (Some(name), Some(command)) = (cmd.name.as_deref(), cmd.command.as_deref()) {
                    add_alias(
                        &mut config,
                        name,
                        command,
                        cmd.group.as_deref(),
                        !cmd.disabled,
                    );
                } else {
                    eprintln!(
                        "Missing required arguments for add. Provide a name and command, or use `add group <name>`."
                    );
                }
            }
        },
        _ => eprintln!("This command is not implemented yet."),
    }
}
