use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod add;
mod disable;
mod edit;
mod enable;
mod list;
mod remove;
mod rename;

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

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Only show errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Show debug information
    #[arg(long, global = true)]
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
