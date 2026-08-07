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
use sync::SyncCommand;

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

    /// Sort aliases or groups by name
    Sort(SortCommand),

    /// Initialize aliasmgr
    #[command(hide = true)]
    Init(InitCommand),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::add::AddTarget;
    use crate::cli::remove::RemoveTarget;

    #[test]
    fn parses_shorthand_add_alias() {
        let cli = Cli::try_parse_from(["aliasmgr", "add", "ll", "ls -l", "--disabled"])
            .expect("shorthand add should parse");

        let Commands::Add(cmd) = cli.command else {
            panic!("expected add command");
        };
        assert!(cmd.target.is_none());
        assert_eq!(cmd.alias.name.as_deref(), Some("ll"));
        assert_eq!(cmd.alias.command.as_deref(), Some("ls -l"));
        assert!(cmd.alias.disabled);
    }

    #[test]
    fn parses_explicit_add_alias_with_reserved_name() {
        let cli = Cli::try_parse_from(["aliasmgr", "add", "alias", "group", "echo group"])
            .expect("explicit add alias should parse");

        let Commands::Add(cmd) = cli.command else {
            panic!("expected add command");
        };
        let Some(AddTarget::Alias(args)) = cmd.target else {
            panic!("expected explicit alias target");
        };
        assert_eq!(args.name, "group");
        assert_eq!(args.command, "echo group");
    }

    #[test]
    fn parses_explicit_add_group() {
        let cli = Cli::try_parse_from(["aliasmgr", "add", "group", "tools", "--disabled"])
            .expect("explicit add group should parse");

        let Commands::Add(cmd) = cli.command else {
            panic!("expected add command");
        };
        let Some(AddTarget::Group(args)) = cmd.target else {
            panic!("expected explicit group target");
        };
        assert_eq!(args.name, "tools");
        assert!(args.disabled);
    }

    #[test]
    fn parses_shorthand_remove() {
        let cli = Cli::try_parse_from(["aliasmgr", "remove", "ll"])
            .expect("shorthand remove should parse");

        let Commands::Remove(cmd) = cli.command else {
            panic!("expected remove command");
        };
        assert!(cmd.target.is_none());
        assert_eq!(cmd.name.as_deref(), Some("ll"));
    }

    #[test]
    fn parses_explicit_remove_alias_with_reserved_name() {
        let cli = Cli::try_parse_from(["aliasmgr", "remove", "alias", "all"])
            .expect("explicit remove alias should parse");

        let Commands::Remove(cmd) = cli.command else {
            panic!("expected remove command");
        };
        let Some(RemoveTarget::Alias(args)) = cmd.target else {
            panic!("expected explicit alias target");
        };
        assert_eq!(args.name, "all");
    }

    #[test]
    fn parses_explicit_remove_group_and_all() {
        let group = Cli::try_parse_from(["aliasmgr", "remove", "group", "tools"])
            .expect("explicit remove group should parse");
        let Commands::Remove(group) = group.command else {
            panic!("expected remove command");
        };
        assert!(matches!(group.target, Some(RemoveTarget::Group(_))));

        let all = Cli::try_parse_from(["aliasmgr", "remove", "all"])
            .expect("remove all should continue to parse");
        let Commands::Remove(all) = all.command else {
            panic!("expected remove command");
        };
        assert!(matches!(all.target, Some(RemoveTarget::All)));
    }
}
