use clap::{Command, CommandFactory, FromArgMatches, Parser, Subcommand, error::ErrorKind};

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
    /// Control when colored output is used
    #[arg(
        short = 'c',
        long,
        global = true,
        value_enum,
        help_heading = "Global Options"
    )]
    pub color: Option<ColorMode>,
    /// Suppress normal command output
    #[arg(short, long, global = true, conflicts_with_all = ["verbose", "debug"], help_heading = "Global Options")]
    pub quiet: bool,
    /// Show informational diagnostics
    #[arg(short, long, global = true, conflicts_with_all = ["quiet", "debug"], help_heading = "Global Options")]
    pub verbose: bool,
    /// Show debug diagnostics
    #[arg(short = 'D', long, global = true, conflicts_with_all = ["verbose", "quiet"], help_heading = "Global Options")]
    pub debug: bool,
    /// Automatically accept confirmation prompts
    #[arg(short = 'y', long, global = true, conflicts_with_all = ["no", "no_input"], help_heading = "Global Options")]
    pub yes: bool,
    /// Automatically decline confirmation prompts
    #[arg(short = 'n', long, global = true, conflicts_with_all = ["yes", "no_input"], help_heading = "Global Options")]
    pub no: bool,
    /// Never prompt; fail when confirmation is required
    #[arg(short = 'N', long, global = true, conflicts_with_all = ["yes", "no"], help_heading = "Global Options")]
    pub no_input: bool,
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn parse() -> Self {
        let matches = Self::command().get_matches();
        Self::from_arg_matches(&matches).unwrap_or_else(|error| error.exit())
    }

    pub fn command() -> Command {
        <Self as CommandFactory>::command()
    }

    pub fn validate_prompt_controls(&self) -> Result<(), clap::Error> {
        let controls = [
            ("--yes", self.yes),
            ("--no", self.no),
            ("--no-input", self.no_input),
        ]
        .into_iter()
        .filter_map(|(name, enabled)| enabled.then_some(name))
        .collect::<Vec<_>>();
        if controls.len() > 1 {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                format!("{} cannot be used with {}", controls[0], controls[1]),
            ));
        }
        if self.yes && matches!(&self.command, Commands::Import(cmd) if cmd.skip_existing) {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "--yes cannot be used with --skip-existing",
            ));
        }
        if self.no && matches!(&self.command, Commands::Import(cmd) if cmd.replace_existing) {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "--no cannot be used with --replace-existing",
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
    use std::collections::HashMap;
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
            &["aliasmgr", "edit", "ll", "-b"][..],
            &["aliasmgr", "edit", "ll", "--global", "--no-global"][..],
        ] {
            assert!(Cli::try_parse_from(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn help_options_are_ordered_by_meaning() {
        fn options(command: &mut Command, heading: &str) -> Vec<String> {
            command.build();
            let help = command.render_help().to_string();
            let mut current_heading = None;
            let mut options = Vec::new();
            for line in help.lines() {
                if let Some(name) = line.strip_suffix(':') {
                    current_heading = Some(name);
                    continue;
                }
                if current_heading != Some(heading) || !line.starts_with("  ") {
                    continue;
                }
                if let Some(long) = line
                    .split_whitespace()
                    .find_map(|part| part.strip_prefix("--"))
                {
                    let long = long.trim_end_matches([',', '>']);
                    if !matches!(long, "help" | "version") {
                        options.push(long.to_owned());
                    }
                }
            }
            options
        }

        let mut command = Cli::command();
        command.build();
        assert_eq!(
            options(&mut command, "Global Options"),
            [
                "color", "quiet", "verbose", "debug", "yes", "no", "no-input"
            ]
        );
        assert_eq!(
            options(command.find_subcommand_mut("add").unwrap(), "Options"),
            ["global", "tag", "description", "disabled"]
        );
        assert_eq!(
            options(command.find_subcommand_mut("edit").unwrap(), "Options"),
            [
                "add-tag",
                "remove-tag",
                "description",
                "clear-description",
                "global",
                "no-global",
            ]
        );
        assert_eq!(
            options(command.find_subcommand_mut("import").unwrap(), "Options"),
            ["dry-run", "skip-existing", "replace-existing", "tag"]
        );
        assert_eq!(
            options(command.find_subcommand_mut("list").unwrap(), "Options"),
            ["tag", "disabled", "all", "global", "columns", "format"]
        );
        assert_eq!(
            options(
                command
                    .find_subcommand_mut("remove")
                    .unwrap()
                    .find_subcommand_mut("alias")
                    .unwrap(),
                "Options",
            ),
            ["pattern", "tag"]
        );
    }

    #[test]
    fn import_parses_multiple_paths_tags_and_collision_policy() {
        let cli = Cli::try_parse_from([
            "aliasmgr", "import", ".bashrc", ".zshrc", "-t", "shell", "-d", "-r",
        ])
        .unwrap();
        assert!(
            matches!(cli.command, Commands::Import(ImportCommand { paths, tag, dry_run: true, replace_existing: true, .. }) if paths == [PathBuf::from(".bashrc"), PathBuf::from(".zshrc")] && tag == ["shell"])
        );
    }

    #[test]
    fn flags_have_help_and_unique_short_options_in_every_context() {
        fn assert_command(command: &Command) {
            let mut shorts = HashMap::new();
            for argument in command
                .get_arguments()
                .filter(|argument| !argument.is_positional())
            {
                assert!(
                    argument.get_help().is_some(),
                    "--{} in {} needs help text",
                    argument.get_long().unwrap_or(argument.get_id().as_str()),
                    command.get_name()
                );
                if let Some(short) = argument.get_short() {
                    assert!(
                        shorts.insert(short, argument.get_id()).is_none(),
                        "duplicate -{short} in {}",
                        command.get_name()
                    );
                }
                if argument.is_global_set() {
                    assert_eq!(argument.get_help_heading(), Some("Global Options"));
                }
            }
            for subcommand in command.get_subcommands() {
                assert_command(subcommand);
            }
        }

        let mut command = Cli::command();
        command.build();
        assert_command(&command);
    }

    #[test]
    fn short_options_match_the_documented_contract() {
        fn short(command: &Command, long: &str) -> Option<char> {
            command
                .get_arguments()
                .find(|argument| argument.get_long() == Some(long))
                .and_then(|argument| argument.get_short())
        }

        let mut command = Cli::command();
        command.build();
        for (long, expected) in [
            ("color", 'c'),
            ("debug", 'D'),
            ("no", 'n'),
            ("no-input", 'N'),
            ("quiet", 'q'),
            ("verbose", 'v'),
            ("yes", 'y'),
        ] {
            assert_eq!(short(&command, long), Some(expected));
        }

        let add = command.find_subcommand("add").unwrap();
        assert_eq!(short(add, "description"), Some('d'));
        assert_eq!(short(add, "disabled"), None);
        assert_eq!(short(add, "global"), Some('g'));
        assert_eq!(short(add, "tag"), Some('t'));

        let edit = command.find_subcommand("edit").unwrap();
        assert_eq!(short(edit, "add-tag"), Some('a'));
        assert_eq!(short(edit, "clear-description"), None);
        assert_eq!(short(edit, "description"), Some('d'));
        assert_eq!(short(edit, "global"), Some('g'));
        assert_eq!(short(edit, "no-global"), None);
        assert_eq!(short(edit, "remove-tag"), Some('r'));
        assert!(short(edit, "toggle-enabled").is_none());
        assert!(short(edit, "toggle-global").is_none());

        let list = command.find_subcommand("list").unwrap();
        assert_eq!(short(list, "columns"), None);
        assert_eq!(short(list, "disabled"), Some('d'));
        assert_eq!(short(list, "format"), Some('f'));
        assert_eq!(short(list, "global"), Some('g'));
        assert_eq!(short(list, "tag"), Some('t'));
        assert!(short(list, "enabled").is_none());

        let import = command.find_subcommand("import").unwrap();
        assert_eq!(short(import, "dry-run"), Some('d'));
        assert_eq!(short(import, "skip-existing"), Some('s'));
        assert_eq!(short(import, "replace-existing"), Some('r'));
        assert_eq!(short(import, "tag"), Some('t'));

        let remove_alias = command
            .find_subcommand("remove")
            .unwrap()
            .find_subcommand("alias")
            .unwrap();
        assert_eq!(short(remove_alias, "pattern"), Some('p'));
        assert_eq!(short(remove_alias, "tag"), Some('t'));
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

        for args in [
            &["aliasmgr", "--yes", "import", ".zshrc", "--skip-existing"][..],
            &["aliasmgr", "--no", "import", ".zshrc", "--replace-existing"][..],
        ] {
            let cli = Cli::try_parse_from(args).unwrap();
            assert_eq!(
                cli.validate_prompt_controls().unwrap_err().kind(),
                ErrorKind::ArgumentConflict
            );
        }
    }
}
