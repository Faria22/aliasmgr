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
    /// Location of the configuration file
    #[arg()]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Only show errors
    #[arg(short, long)]
    pub quiet: bool,

    /// Show debug information
    #[arg(long)]
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
