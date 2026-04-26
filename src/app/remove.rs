use crate::catalog::types::AliasCatalog;

use crate::core::list::get_aliases_from_single_group;
use crate::core::r#move::move_alias;
use crate::core::remove::{remove_alias, remove_aliases, remove_all, remove_group};
use crate::core::{Failure, Outcome};

use super::shell::ShellType;

use crate::cli::interaction::prompt_confirm_remove_all;

use crate::cli::remove::{RemoveCommand, RemoveTarget};

pub fn handle_remove_all(
    catalog: &mut AliasCatalog,
    shell: &ShellType,
    confirmation: impl Fn() -> bool,
) -> Result<Outcome, Failure> {
    if confirmation() {
        remove_all(catalog, shell)
    } else {
        Ok(Outcome::NoChanges)
    }
}

pub fn handle_remove(
    catalog: &mut AliasCatalog,
    cmd: RemoveCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        RemoveTarget::Alias(args) => remove_alias(catalog, &args.name),
        RemoveTarget::Group(args) => {
            if let Some(name) = &args.name {
                // Remove named group
                let group_id = Some(name.clone());
                let aliases = get_aliases_from_single_group(catalog, group_id.as_deref(), shell)?;
                remove_group(catalog, name)?;
                if args.reassign {
                    for alias in aliases {
                        move_alias(catalog, &alias, &None)?;
                    }
                    Ok(Outcome::CatalogChanged)
                } else {
                    remove_aliases(catalog, &aliases)
                }
            } else {
                // Remove ungrouped aliases
                let aliases = get_aliases_from_single_group(catalog, None, shell)?;
                remove_aliases(catalog, &aliases)
            }
        }
        RemoveTarget::All => handle_remove_all(catalog, shell, prompt_confirm_remove_all),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;
    use assert_matches::assert_matches;

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("files".to_string(), true);
        catalog.aliases.insert(
            "ls".to_string(),
            Alias::new("ls -la".to_string(), Some("files".to_string()), true, false),
        );
        catalog.aliases.insert(
            "rm".to_string(),
            Alias::new("rm -rf".to_string(), None, true, false),
        );
        catalog
    }

    #[test]
    fn test_remove_alias_success() {
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
            RemoveCommand {
                target: RemoveTarget::Alias(crate::cli::remove::RemoveAliasArgs {
                    name: "ls".to_string(),
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(!catalog.aliases.contains_key("ls"));
        assert!(catalog.aliases.contains_key("rm"));
        assert!(catalog.groups.contains_key("files"));
    }

    #[test]
    fn test_remove_alias_failure() {
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
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
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
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
        assert!(!catalog.groups.contains_key("files"));
    }

    #[test]
    fn test_remove_group_failure() {
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
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
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
            RemoveCommand {
                target: RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: None,
                    reassign: false,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(!catalog.aliases.contains_key("rm"));
        assert!(catalog.aliases.contains_key("ls"));
    }

    #[test]
    fn test_remove_group_with_reassign() {
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
            RemoveCommand {
                target: RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: Some("files".to_string()),
                    reassign: true,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(!catalog.groups.contains_key("files"));
        assert!(catalog.aliases.contains_key("ls"));
        assert!(catalog.aliases.get("ls").unwrap().group.is_none());
    }

    #[test]
    fn test_remove_all_with_confirmation() {
        let mut catalog = sample_catalog();
        let result = handle_remove_all(&mut catalog, &ShellType::Bash, || true);
        assert!(result.is_ok());
        assert!(catalog.aliases.is_empty());
        assert!(catalog.groups.is_empty());
    }

    #[test]
    fn test_remove_all_without_confirmation() {
        let mut catalog = sample_catalog();
        let result = handle_remove_all(&mut catalog, &ShellType::Bash, || false);
        assert!(result.is_ok());
        assert!(!catalog.aliases.is_empty());
        assert!(!catalog.groups.is_empty());
    }
}
