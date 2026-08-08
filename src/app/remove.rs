use crate::catalog::types::AliasCatalog;

use crate::core::list::get_aliases_from_single_group;
use crate::core::r#move::move_alias;
use crate::core::remove::{remove_alias, remove_aliases, remove_all, remove_group};
use crate::core::{Failure, Outcome};

use super::shell::ShellType;

use crate::cli::interaction::{
    prompt_alias_or_group, prompt_confirm_remove_all, prompt_reassign_group_aliases,
};

use crate::cli::remove::{RemoveCommand, RemoveTarget};

use super::resource::{ResourceType, resolve_resource_type};

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

fn handle_remove_group(
    catalog: &mut AliasCatalog,
    name: Option<&str>,
    reassign: bool,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    if let Some(name) = name {
        let aliases = get_aliases_from_single_group(catalog, Some(name), shell)?;
        remove_group(catalog, name)?;
        if reassign {
            for alias in aliases {
                move_alias(catalog, &alias, &None)?;
            }
            Ok(Outcome::CatalogChanged)
        } else {
            remove_aliases(catalog, &aliases)
        }
    } else {
        let aliases = get_aliases_from_single_group(catalog, None, shell)?;
        remove_aliases(catalog, &aliases)
    }
}

fn handle_remove_shorthand(
    catalog: &mut AliasCatalog,
    name: &str,
    shell: &ShellType,
    choose_alias: impl FnOnce(&str) -> bool,
    reassign_group: impl FnOnce(&str) -> bool,
) -> Result<Outcome, Failure> {
    match resolve_resource_type(catalog, name, choose_alias) {
        ResourceType::Alias => remove_alias(catalog, name),
        ResourceType::Group => {
            handle_remove_group(catalog, Some(name), reassign_group(name), shell)
        }
    }
}

pub fn handle_remove(
    catalog: &mut AliasCatalog,
    cmd: RemoveCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        Some(RemoveTarget::Alias(args)) => remove_alias(catalog, &args.name),
        Some(RemoveTarget::Group(args)) => {
            handle_remove_group(catalog, args.name.as_deref(), args.reassign, shell)
        }
        Some(RemoveTarget::All) => handle_remove_all(catalog, shell, prompt_confirm_remove_all),
        None => handle_remove_shorthand(
            catalog,
            cmd.name
                .as_deref()
                .expect("clap requires a name when no subcommand is used"),
            shell,
            |name| prompt_alias_or_group(name, "removed"),
            prompt_reassign_group_aliases,
        ),
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
                target: Some(RemoveTarget::Alias(crate::cli::remove::RemoveAliasArgs {
                    name: "ls".to_string(),
                })),
                name: None,
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
                target: Some(RemoveTarget::Alias(crate::cli::remove::RemoveAliasArgs {
                    name: "nonexistent".to_string(),
                })),
                name: None,
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
                target: Some(RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: Some("files".to_string()),
                    reassign: false,
                })),
                name: None,
            },
            &ShellType::Bash,
        );
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(!catalog.groups.contains_key("files"));
    }

    #[test]
    fn test_remove_group_failure() {
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
            RemoveCommand {
                target: Some(RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: Some("nonexistent".to_string()),
                    reassign: false,
                })),
                name: None,
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
                target: Some(RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: None,
                    reassign: false,
                })),
                name: None,
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
                target: Some(RemoveTarget::Group(crate::cli::remove::GroupRemoveArgs {
                    name: Some("files".to_string()),
                    reassign: true,
                })),
                name: None,
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(!catalog.groups.contains_key("files"));
        assert!(catalog.aliases.contains_key("ls"));
        assert!(catalog.aliases.get("ls").unwrap().group.is_none());
    }

    #[test]
    fn test_remove_shorthand_alias_without_prompt() {
        let mut catalog = sample_catalog();
        let result = handle_remove(
            &mut catalog,
            RemoveCommand {
                target: None,
                name: Some("rm".to_string()),
            },
            &ShellType::Bash,
        );

        assert!(result.is_ok());
        assert!(!catalog.aliases.contains_key("rm"));
        assert!(catalog.groups.contains_key("files"));
    }

    #[test]
    fn test_remove_shorthand_group_without_resource_prompt() {
        let mut catalog = sample_catalog();
        let result = handle_remove_shorthand(
            &mut catalog,
            "files",
            &ShellType::Bash,
            |_| panic!("a sole group should not prompt for a resource"),
            |name| {
                assert_eq!(name, "files");
                false
            },
        );

        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(!catalog.groups.contains_key("files"));
        assert!(!catalog.aliases.contains_key("ls"));
    }

    #[test]
    fn test_remove_shorthand_group_with_reassign() {
        let mut catalog = sample_catalog();
        let result = handle_remove_shorthand(
            &mut catalog,
            "files",
            &ShellType::Bash,
            |_| panic!("a sole group should not prompt for a resource"),
            |name| {
                assert_eq!(name, "files");
                true
            },
        );

        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(!catalog.groups.contains_key("files"));
        assert!(catalog.aliases.contains_key("ls"));
        assert!(catalog.aliases.get("ls").unwrap().group.is_none());
    }

    #[test]
    fn test_remove_shorthand_collision_choose_alias() {
        let mut catalog = sample_catalog();
        catalog.aliases.insert(
            "files".to_string(),
            Alias::new("find .".to_string(), None, true, false),
        );

        let result = handle_remove_shorthand(
            &mut catalog,
            "files",
            &ShellType::Bash,
            |name| {
                assert_eq!(name, "files");
                true
            },
            |_| panic!("choosing the alias should not prompt for reassignment"),
        );

        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(!catalog.aliases.contains_key("files"));
        assert!(catalog.groups.contains_key("files"));
        assert!(catalog.aliases.contains_key("ls"));
    }

    #[test]
    fn test_remove_shorthand_collision_choose_group() {
        let mut catalog = sample_catalog();
        catalog.aliases.insert(
            "files".to_string(),
            Alias::new("find .".to_string(), None, true, false),
        );

        let result = handle_remove_shorthand(
            &mut catalog,
            "files",
            &ShellType::Bash,
            |name| {
                assert_eq!(name, "files");
                false
            },
            |name| {
                assert_eq!(name, "files");
                false
            },
        );

        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(catalog.aliases.contains_key("files"));
        assert!(!catalog.groups.contains_key("files"));
        assert!(!catalog.aliases.contains_key("ls"));
    }

    #[test]
    fn test_remove_shorthand_missing_defaults_to_alias_failure() {
        let mut catalog = sample_catalog();
        let result = handle_remove_shorthand(
            &mut catalog,
            "nonexistent",
            &ShellType::Bash,
            |_| panic!("a missing resource should not prompt for a resource"),
            |_| panic!("a missing resource should not prompt for reassignment"),
        );

        assert_matches!(result, Err(Failure::AliasDoesNotExist));
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
