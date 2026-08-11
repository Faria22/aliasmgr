use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

pub(crate) mod add;
pub(crate) mod disable;
pub(crate) mod doctor;
pub(crate) mod edit;
pub(crate) mod enable;
pub(crate) mod import;
pub(crate) mod init;
pub(crate) mod interaction;
pub(crate) mod list;
pub(crate) mod remove;
pub(crate) mod rename;
pub(crate) mod selector;
pub(crate) mod sync;

use crate::config::ColorMode;
use add::AddCommand;
use disable::DisableCommand;
use doctor::DoctorCommand;
use edit::EditCommand;
use enable::EnableCommand;
use import::ImportCommand;
use init::InitCommand;
use list::ListCommand;
use remove::RemoveCommand;
use rename::RenameCommand;
use sync::{ShellSyncCommand, SyncCommand};

pub fn validate_tag(tag: &str) -> Result<String, String> {
    if crate::core::validation::is_valid_tag(tag) {
        Ok(tag.to_owned())
    } else {
        Err("tags must not be empty or contain whitespace".into())
    }
}

#[derive(Parser)]
#[command(
    version,
    about,
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(long, global = true, conflicts_with = "no_input")]
    pub force: bool,
    #[arg(long, global = true, conflicts_with = "force")]
    pub no_input: bool,
    #[arg(short, long, global = true, conflicts_with_all = ["quiet", "debug"])]
    pub verbose: bool,
    #[arg(short, long, global = true, conflicts_with_all = ["verbose", "debug"])]
    pub quiet: bool,
    #[arg(long, global = true, conflicts_with_all = ["verbose", "quiet"])]
    pub debug: bool,
    #[arg(long, global = true, value_enum)]
    pub color: Option<ColorMode>,
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn validate_prompt_controls(&self) -> Result<(), clap::Error> {
        if self.force && self.no_input {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "--force cannot be used with --no-input",
            ));
        }
        if self.force && matches!(&self.command, Commands::ShellSync(cmd) if cmd.if_changed) {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "--force cannot be used with --if-changed",
            ));
        }
        if self.force && matches!(&self.command, Commands::Import(cmd) if cmd.skip_existing) {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "--force cannot be used with --skip-existing",
            ));
        }
        if matches!(&self.command, Commands::Edit(cmd) if !cmd.has_changes()) {
            return Err(Self::command().error(
                ErrorKind::MissingRequiredArgument,
                "edit requires a replacement command or at least one metadata option",
            ));
        }
        Ok(())
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new alias
    #[command(visible_alias = "a")]
    Add(AddCommand),
    /// Remove aliases, detach a tag, or clear the catalog
    #[command(visible_alias = "rm")]
    Remove(RemoveCommand),
    /// List aliases as a table or JSON
    #[command(visible_alias = "ls")]
    List(ListCommand),
    /// Enable aliases directly, by filter, or by tag
    #[command(visible_alias = "en")]
    Enable(EnableCommand),
    /// Disable aliases directly, by filter, or by tag
    #[command(visible_alias = "ds")]
    Disable(DisableCommand),
    /// Check catalog correctness and shell compatibility
    #[command(visible_alias = "validate")]
    Doctor(DoctorCommand),
    /// Rename an alias or tag
    #[command(visible_alias = "rn")]
    Rename(RenameCommand),
    /// Edit an alias command or metadata
    #[command(visible_alias = "ed")]
    Edit(EditCommand),
    /// Import aliases from Bash or Zsh files
    #[command(visible_alias = "im")]
    Import(ImportCommand),
    /// Synchronize aliases with the catalog
    Sync(SyncCommand),
    #[command(hide = true)]
    ShellSync(ShellSyncCommand),
    #[command(hide = true)]
    Init(InitCommand),
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn parses_repeated_add_tags() {
        let cli = Cli::try_parse_from([
            "aliasmgr", "add", "ll", "ls -la", "--tag", "shell", "--tag", "files",
        ])
        .unwrap();
        assert!(
            matches!(cli.command, Commands::Add(AddCommand { tag, .. }) if tag == ["shell", "files"])
        );
    }

    #[test]
    fn alias_is_a_regular_add_name() {
        let cli = Cli::try_parse_from(["aliasmgr", "add", "alias", "echo alias"]).unwrap();
        assert!(
            matches!(cli.command, Commands::Add(AddCommand { name, command, .. }) if name == "alias" && command == "echo alias")
        );
    }

    #[test]
    fn removed_move_command_is_rejected() {
        assert!(Cli::try_parse_from(["aliasmgr", "move", "ll", "dev"]).is_err());
    }

    #[test]
    fn list_enabled_flag_is_rejected_because_enabled_is_the_default() {
        assert!(Cli::try_parse_from(["aliasmgr", "list", "--enabled"]).is_err());
    }

    #[test]
    fn tag_validation_rejects_empty_and_whitespace() {
        assert!(validate_tag("").is_err());
        assert!(validate_tag("two words").is_err());
        assert_eq!(validate_tag("Case-Sensitive").unwrap(), "Case-Sensitive");
    }

    #[test]
    fn edit_requires_at_least_one_change() {
        let cli = Cli::try_parse_from(["aliasmgr", "edit", "ll"]).unwrap();
        assert_eq!(
            cli.validate_prompt_controls().unwrap_err().kind(),
            ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn edit_rejects_removed_toggle_flags_and_conflicting_global_states() {
        for args in [
            &["aliasmgr", "edit", "ll", "--toggle-enabled"][..],
            &["aliasmgr", "edit", "ll", "--toggle-global"][..],
            &["aliasmgr", "edit", "ll", "--global", "--no-global"][..],
        ] {
            assert!(Cli::try_parse_from(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn import_parses_multiple_paths_tags_and_collision_policy() {
        let cli = Cli::try_parse_from([
            "aliasmgr",
            "import",
            ".bashrc",
            ".zshrc",
            "--tag",
            "shell",
            "--dry-run",
            "--replace-existing",
        ])
        .unwrap();
        assert!(
            matches!(cli.command, Commands::Import(ImportCommand { paths, tag, dry_run: true, replace_existing: true, .. }) if paths == [PathBuf::from(".bashrc"), PathBuf::from(".zshrc")] && tag == ["shell"])
        );
    }

    #[test]
    fn import_rejects_conflicting_collision_policies() {
        assert!(
            Cli::try_parse_from([
                "aliasmgr",
                "import",
                ".zshrc",
                "--skip-existing",
                "--replace-existing",
            ])
            .is_err()
        );

        let cli =
            Cli::try_parse_from(["aliasmgr", "--force", "import", ".zshrc", "--skip-existing"])
                .unwrap();
        assert_eq!(
            cli.validate_prompt_controls().unwrap_err().kind(),
            ErrorKind::ArgumentConflict
        );
    }
}
