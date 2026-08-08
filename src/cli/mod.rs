use clap::{Parser, Subcommand};

pub(crate) mod add;
pub(crate) mod disable;
pub(crate) mod edit;
pub(crate) mod enable;
pub(crate) mod init;
pub(crate) mod list;
pub(crate) mod r#move;
pub(crate) mod remove;
pub(crate) mod rename;
pub(crate) mod sort;
pub(crate) mod sync;

pub(crate) mod interaction;

use add::AddCommand;
use disable::DisableCommand;
use edit::EditCommand;
use enable::EnableCommand;
use init::InitCommand;
use list::ListCommand;
use r#move::MoveCommand;
use remove::RemoveCommand;
use rename::RenameCommand;
use sort::SortCommand;
use sync::{ShellSyncCommand, SyncCommand};

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
    #[command(visible_alias = "ds")]
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

    /// Synchronize aliases with catalog file
    Sync(SyncCommand),

    /// Generate shell commands that reconcile aliases in the current terminal
    #[command(hide = true)]
    ShellSync(ShellSyncCommand),

    /// Sort aliases or groups by name
    Sort(SortCommand),

    /// Initialize aliasmgr
    #[command(hide = true)]
    Init(InitCommand),
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_init_without_automatic_sync() {
        let cli = Cli::try_parse_from(["aliasmgr", "init", "bash", "--no-auto-sync"])
            .expect("init option should parse");
        let Commands::Init(cmd) = cli.command else {
            panic!("expected init command");
        };
        assert!(cmd.no_auto_sync);
    }

    #[test]
    fn parses_internal_conditional_shell_sync() {
        let cli = Cli::try_parse_from(["aliasmgr", "shell-sync", "--if-changed"])
            .expect("internal shell sync should parse");
        let Commands::ShellSync(cmd) = cli.command else {
            panic!("expected shell sync command");
        };
        assert!(cmd.if_changed);
        assert!(!cmd.force);
    }
}
