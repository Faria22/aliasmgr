use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub(crate) mod add;
pub(crate) mod disable;
pub(crate) mod edit;
pub(crate) mod enable;
pub(crate) mod list;
pub(crate) mod remove;
pub(crate) mod rename;

use add::AddCommand;
use disable::DisableCommand;
use edit::EditCommand;
use enable::EnableCommand;
use list::ListCommand;
use remove::RemoveCommand;
use rename::RenameCommand;

#[derive(Parser)]
#[command(
    version,
    about,
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Custom location of the configuration file
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// Increase output verbosity
    #[arg(
        short,
        long,
        global = true,
        conflicts_with_all = ["quiet", "debug"]
    )]
    pub verbose: bool,

    /// Silence all output except errors
    #[arg(
        short,
        long,
        global = true,
        conflicts_with_all = ["verbose", "debug"]
    )]
    pub quiet: bool,

    /// Enable debug logging
    #[arg(
        long,
        global = true,
        conflicts_with_all = ["verbose", "quiet"]
    )]
    pub debug: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new alias or alias group
    #[command(visible_alias = "a")]
    Add(AddCommand),

    /// Remove an existing alias or alias group
    #[command(visible_alias = "rm")]
    Remove(RemoveCommand),

    /// List aliases
    #[command(visible_alias = "ls")]
    List(ListCommand),

    /// Enable an alias or alias group
    #[command(visible_alias = "en")]
    Enable(EnableCommand),

    /// Disable an alias or alias group
    #[command(visible_alias = "dis")]
    Disable(DisableCommand),

    /// Rename an existing alias or alias group
    #[command(visible_alias = "rn")]
    Rename(RenameCommand),

    /// Edit an existing alias
    #[command(visible_alias = "ed")]
    Edit(EditCommand),

    /// Synchronize aliases with configuration file
    Sync,

    /// Initialize aliasmgr
    Init,
}
