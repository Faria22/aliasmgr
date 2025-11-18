mod cli;
mod config;
mod core;

use clap::Parser;
use cli::{Cli, Commands, add::AddTarget};
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
        Commands::Add(cmd) => match cmd.target {
            AddTarget::Alias(args) => {
                add_alias(
                    &mut config,
                    &args.name,
                    &args.command,
                    args.group.as_deref(),
                    !args.disabled,
                );
            }
            AddTarget::Group(args) => add_group(&mut config, &args.name, !args.disabled),
        },
        _ => eprintln!("This command is not implemented yet."),
    }
}
