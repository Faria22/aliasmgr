use clap::{Parser, Subcommand};

pub(crate) mod add;
pub(crate) mod convert;
pub(crate) mod disable;
pub(crate) mod edit;
pub(crate) mod enable;
pub(crate) mod init;
pub(crate) mod list;
pub(crate) mod r#move;
pub(crate) mod remove;
pub(crate) mod rename;

pub(crate) mod interaction;

use add::AddCommand;
use convert::ConvertCommand;
use disable::DisableCommand;
use edit::EditCommand;
use enable::EnableCommand;
use init::InitCommand;
use list::ListCommand;
use r#move::MoveCommand;
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

    /// Move an alias to a different group
    #[command(visible_alias = "mv")]
    Move(MoveCommand),

    /// Synchronize aliases with configuration file
    Sync,

    /// Convert aliases from a .sh file
    Convert(ConvertCommand),

    /// Initialize aliasmgr
    #[command(hide = true)]
    Init(InitCommand),
}
