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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::cli::add::AddTarget;
    use crate::cli::disable::DisableTarget;
    use crate::cli::enable::EnableTarget;
    use crate::cli::remove::RemoveTarget;
    use crate::cli::rename::RenameTarget;

    #[test]
    fn parses_shorthand_add_alias() {
        let cli = Cli::try_parse_from(["aliasmgr", "add", "ll", "ls -l", "--disabled"])
            .expect("shorthand add should parse");

        assert!(matches!(
            cli.command,
            Commands::Add(AddCommand {
                target: None,
                alias,
            }) if alias.name.as_deref() == Some("ll")
                && alias.command.as_deref() == Some("ls -l")
                && alias.disabled
        ));
    }

    #[test]
    fn parses_explicit_add_alias_with_reserved_name() {
        let cli = Cli::try_parse_from(["aliasmgr", "add", "alias", "group", "echo group"])
            .expect("explicit add alias should parse");

        assert!(matches!(
            cli.command,
            Commands::Add(AddCommand {
                target: Some(AddTarget::Alias(args)),
                ..
            }) if args.name == "group" && args.command == "echo group"
        ));
    }

    #[test]
    fn parses_explicit_add_group() {
        let cli = Cli::try_parse_from(["aliasmgr", "add", "group", "tools", "--disabled"])
            .expect("explicit add group should parse");

        assert!(matches!(
            cli.command,
            Commands::Add(AddCommand {
                target: Some(AddTarget::Group(args)),
                ..
            }) if args.name == "tools" && args.disabled
        ));
    }

    #[test]
    fn parses_shorthand_remove() {
        let cli = Cli::try_parse_from(["aliasmgr", "remove", "ll"])
            .expect("shorthand remove should parse");

        assert!(matches!(
            cli.command,
            Commands::Remove(RemoveCommand {
                target: None,
                name: Some(name),
            }) if name == "ll"
        ));
    }

    #[test]
    fn parses_explicit_remove_alias_with_reserved_name() {
        let cli = Cli::try_parse_from(["aliasmgr", "remove", "alias", "all"])
            .expect("explicit remove alias should parse");

        assert!(matches!(
            cli.command,
            Commands::Remove(RemoveCommand {
                target: Some(RemoveTarget::Alias(args)),
                ..
            }) if args.name == "all"
        ));
    }

    #[test]
    fn parses_explicit_remove_group_and_all() {
        let group = Cli::try_parse_from(["aliasmgr", "remove", "group", "tools"])
            .expect("explicit remove group should parse");
        assert!(matches!(
            group.command,
            Commands::Remove(RemoveCommand {
                target: Some(RemoveTarget::Group(_)),
                ..
            })
        ));

        let all = Cli::try_parse_from(["aliasmgr", "remove", "all"])
            .expect("remove all should continue to parse");
        assert!(matches!(
            all.command,
            Commands::Remove(RemoveCommand {
                target: Some(RemoveTarget::All),
                ..
            })
        ));
    }

    #[test]
    fn parses_non_interactive_reassigned_alias_actions() {
        let enable = Cli::try_parse_from([
            "aliasmgr",
            "remove",
            "group",
            "tools",
            "--reassign",
            "--enable-reassigned",
        ])
        .expect("enable-reassigned should parse with reassign");
        assert!(matches!(
            enable.command,
            Commands::Remove(RemoveCommand {
                target: Some(RemoveTarget::Group(args)),
                ..
            }) if args.reassign && args.enable_reassigned && !args.disable_reassigned
        ));

        let disable = Cli::try_parse_from([
            "aliasmgr",
            "remove",
            "group",
            "tools",
            "--reassign",
            "--disable-reassigned",
        ])
        .expect("disable-reassigned should parse with reassign");
        assert!(matches!(
            disable.command,
            Commands::Remove(RemoveCommand {
                target: Some(RemoveTarget::Group(args)),
                ..
            }) if args.reassign && !args.enable_reassigned && args.disable_reassigned
        ));
    }

    #[test]
    fn reassigned_alias_actions_require_reassign_and_conflict() {
        assert!(
            Cli::try_parse_from([
                "aliasmgr",
                "remove",
                "group",
                "tools",
                "--enable-reassigned",
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "aliasmgr",
                "remove",
                "group",
                "tools",
                "--reassign",
                "--enable-reassigned",
                "--disable-reassigned",
            ])
            .is_err()
        );
    }

    #[test]
    fn parses_shorthand_and_explicit_enable() {
        let shorthand = Cli::try_parse_from(["aliasmgr", "enable", "ll"])
            .expect("shorthand enable should parse");
        assert!(matches!(
            shorthand.command,
            Commands::Enable(EnableCommand {
                target: None,
                name: Some(name),
            }) if name == "ll"
        ));

        let explicit = Cli::try_parse_from(["aliasmgr", "enable", "group", "tools"])
            .expect("explicit enable group should parse");
        assert!(matches!(
            explicit.command,
            Commands::Enable(EnableCommand {
                target: Some(EnableTarget::Group(args)),
                ..
            }) if args.name == "tools"
        ));
    }

    #[test]
    fn parses_shorthand_and_explicit_disable() {
        let shorthand = Cli::try_parse_from(["aliasmgr", "disable", "ll"])
            .expect("shorthand disable should parse");
        assert!(matches!(
            shorthand.command,
            Commands::Disable(DisableCommand {
                target: None,
                name: Some(name),
            }) if name == "ll"
        ));

        let explicit = Cli::try_parse_from(["aliasmgr", "disable", "alias", "group"])
            .expect("explicit disable alias should parse");
        assert!(matches!(
            explicit.command,
            Commands::Disable(DisableCommand {
                target: Some(DisableTarget::Alias(args)),
                ..
            }) if args.name == "group"
        ));
    }

    #[test]
    fn parses_shorthand_and_explicit_rename() {
        let shorthand = Cli::try_parse_from(["aliasmgr", "rename", "ll", "list"])
            .expect("shorthand rename should parse");
        assert!(matches!(
            shorthand.command,
            Commands::Rename(RenameCommand {
                target: None,
                old_name: Some(old_name),
                new_name: Some(new_name),
            }) if old_name == "ll" && new_name == "list"
        ));

        let explicit = Cli::try_parse_from(["aliasmgr", "rename", "group", "tools", "commands"])
            .expect("explicit rename group should parse");
        assert!(matches!(
            explicit.command,
            Commands::Rename(RenameCommand {
                target: Some(RenameTarget::Group(args)),
                ..
            }) if args.old_name == "tools" && args.new_name == "commands"
        ));
    }

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
    fn parses_list_output_format_and_defaults_to_human() {
        let default = Cli::try_parse_from(["aliasmgr", "list"]).unwrap();
        assert!(matches!(
            default.command,
            Commands::List(ListCommand {
                format: list::OutputFormat::Human,
                ..
            })
        ));

        let json = Cli::try_parse_from(["aliasmgr", "list", "--format", "json"]).unwrap();
        assert!(matches!(
            json.command,
            Commands::List(ListCommand {
                format: list::OutputFormat::Json,
                ..
            })
        ));
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
