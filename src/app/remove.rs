use crate::config::types::Config;

use crate::core::list::{GroupId, get_single_group};
use crate::core::r#move::move_alias;
use crate::core::remove::{remove_alias, remove_aliases, remove_all, remove_group};
use crate::core::{Failure, Outcome};

use super::shell::ShellType;

use crate::cli::interaction::prompt_confirm_remove_all;

use crate::cli::remove::{RemoveCommand, RemoveTarget};

pub fn handle_remove_all(
    config: &mut Config,
    confirmation: impl Fn() -> bool,
) -> Result<Outcome, Failure> {
    if confirmation() {
        remove_all(config)
    } else {
        Ok(Outcome::NoChanges)
    }
}

pub fn handle_remove(
    config: &mut Config,
    cmd: RemoveCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        RemoveTarget::Alias(args) => remove_alias(config, &args.name),
        RemoveTarget::Group(args) => {
            if let Some(name) = &args.name {
                // Remove named group
                let group_id = GroupId::Named(name.clone());
                let aliases = get_single_group(config, &group_id, shell)?;
                remove_group(config, name)?;
                if args.reassign {
                    for alias in aliases {
                        move_alias(config, &alias, &None)?;
                    }
                    Ok(Outcome::ConfigChanged)
                } else {
                    remove_aliases(config, &aliases)
                }
            } else {
                // Remove ungrouped aliases
                let aliases = get_single_group(config, &GroupId::Ungrouped, shell)?;
                remove_aliases(config, &aliases)
            }
        }
        RemoveTarget::All => handle_remove_all(config, prompt_confirm_remove_all),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Alias;
    use assert_matches::assert_matches;

    fn sample_config() -> Config {
        let mut config = Config::new();
        config.groups.insert("files".to_string(), true);
        config.aliases.insert(
            "ls".to_string(),
            Alias::new("ls -la".to_string(), Some("files".to_string()), true, false),
        );
        config.aliases.insert(
            "rm".to_string(),
            Alias::new("rm -rf".to_string(), None, true, false),
        );
        config
    }

    #[test]
    fn test_remove_alias_success() {
        let mut config = sample_config();
        let result = handle_remove(
            &mut config,
            RemoveCommand {
                target: RemoveTarget::Alias(crate::cli::remove::RemoveAliasArgs {
                    name: "ls".to_string(),
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(!config.aliases.contains_key("ls"));
        assert!(config.aliases.contains_key("rm"));
        assert!(config.groups.contains_key("files"));
    }

    #[test]
    fn test_remove_alias_failure() {
        let mut config = sample_config();
        let result = handle_remove(
            &mut config,
            RemoveCommand {
                target: RemoveTarget::Alias(crate::cli::remove::RemoveAliasArgs {
                    name: "nonexistent".to_string(),
                }),
            },
            &ShellType::Bash,
        );
        assert_matches!(result.err(), Some(Failure::AliasDoesNotExist));
    }

    #[test]
    fn test_remove_group_success() {
        let mut config = sample_config();
        let result = handle_remove(
            &mut config,
            RemoveCommand {
                target: RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: Some("files".to_string()),
                    reassign: false,
                }),
            },
            &ShellType::Bash,
        );
        assert_eq!(
            result.unwrap(),
            Outcome::Command("unalias 'ls'".to_string())
        );
        assert!(!config.groups.contains_key("files"));
    }

    #[test]
    fn test_remove_group_failure() {
        let mut config = sample_config();
        let result = handle_remove(
            &mut config,
            RemoveCommand {
                target: RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: Some("nonexistent".to_string()),
                    reassign: false,
                }),
            },
            &ShellType::Bash,
        );
        assert_matches!(result.err(), Some(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_remove_ungrouped_aliases() {
        let mut config = sample_config();
        let result = handle_remove(
            &mut config,
            RemoveCommand {
                target: RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: None,
                    reassign: false,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(!config.aliases.contains_key("rm"));
        assert!(config.aliases.contains_key("ls"));
    }

    #[test]
    fn test_remove_group_with_reassign() {
        let mut config = sample_config();
        let result = handle_remove(
            &mut config,
            RemoveCommand {
                target: RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: Some("files".to_string()),
                    reassign: true,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(!config.groups.contains_key("files"));
        assert!(config.aliases.contains_key("ls"));
        assert!(config.aliases.get("ls").unwrap().group.is_none());
    }

    #[test]
    fn test_remove_all_with_confirmation() {
        let mut config = sample_config();
        let result = handle_remove_all(&mut config, || true);
        assert!(result.is_ok());
        assert!(config.aliases.is_empty());
        assert!(config.groups.is_empty());
    }

    #[test]
    fn test_remove_all_without_confirmation() {
        let mut config = sample_config();
        let result = handle_remove_all(&mut config, || false);
        assert!(result.is_ok());
        assert!(!config.aliases.is_empty());
        assert!(!config.groups.is_empty());
    }
}
