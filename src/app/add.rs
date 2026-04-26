use crate::core::{Failure, Outcome};

use crate::core::add::{add_alias, add_group};
use crate::core::edit::edit_alias;
use crate::core::r#move::move_alias;

use crate::catalog::types::{Alias, AliasCatalog};

use crate::cli::add::{AddCommand, AddTarget};
use crate::cli::interaction::{prompt_create_non_existent_group, prompt_overwrite_existing_alias};

use super::list::format_alias_info;

use super::shell::ShellType;

use log::{error, info};

/// Handle overwriting an existing alias
fn handle_overwrite_existing_alias(
    catalog: &mut AliasCatalog,
    name: &str,
    alias: &Alias,
    overwrite: bool,
    create_group: impl Fn(&str) -> bool,
) -> Result<Outcome, Failure> {
    // If the alias already exists, we check if the user wants to overwrite it
    if overwrite {
        // Move alias to new group if it is different from the previous one
        if alias.group != catalog.aliases.get(name).and_then(|a| a.group.clone()) {
            info!(
                "Moving alias '{}' to group '{:?}'.",
                name,
                alias.group.clone()
            );
            let group = alias.group.clone().map(|g| g.to_string());

            if let Err(Failure::GroupDoesNotExist) = move_alias(catalog, name, &group) {
                // If the group does not exist, we ask the user if they want to create it
                let group = group.expect("group has to be `Some` for this error to arise");
                handle_create_non_existent_group(catalog, &group, create_group(&group))?;

                // Retry moving the alias after creating the group
                move_alias(catalog, name, &Some(group))?;
            }
        }

        // User wants to overwrite the existing alias
        info!("Overwriting existing alias '{}'.", name);
        edit_alias(catalog, name, alias)
    } else {
        // User does not want to overwrite the existing alias
        info!("Not overwriting existing alias '{}'.", name);
        Ok(Outcome::NoChanges)
    }
}

/// Handle adding non-existent group
fn handle_create_non_existent_group(
    catalog: &mut AliasCatalog,
    name: &str,
    create_group: bool,
) -> Result<Outcome, Failure> {
    if create_group {
        // User wants to create the group
        info!("Creating group '{}'.", name);
        add_group(catalog, name, true)
    } else {
        // User does not want to create the group
        info!("Group '{:?}' was not added", name);
        Ok(Outcome::NoChanges)
    }
}

/// Handle add alias
fn handle_add_alias(
    catalog: &mut AliasCatalog,
    name: &str,
    alias: &Alias,
    overwrite: impl Fn(&str) -> bool,
    create_group: impl Fn(&str) -> bool,
) -> Result<Outcome, Failure> {
    match add_alias(catalog, name, alias) {
        // Alias added successfully
        Ok(outcome) => Ok(outcome),

        // Handle errors
        Err(e) => {
            match e {
                // Alias already exists
                Failure::AliasAlreadyExists => {
                    let alias_info = format_alias_info(catalog, name).expect("alias must exist");
                    handle_overwrite_existing_alias(
                        catalog,
                        name,
                        alias,
                        overwrite(&alias_info),
                        create_group,
                    )
                }

                // Group that alias will belong to does not exist
                Failure::GroupDoesNotExist => {
                    let group_name = alias
                        .group
                        .as_ref()
                        .expect("group has to be `Some` for these error to arise");
                    match handle_create_non_existent_group(
                        catalog,
                        group_name,
                        create_group(group_name),
                    ) {
                        // Group created successfully
                        Ok(Outcome::CatalogChanged) => {
                            // Retry adding the alias after creating the group
                            add_alias(catalog, name, alias)?;
                            Ok(Outcome::CatalogChanged)
                        }
                        // User chose not to create the group
                        Ok(Outcome::NoChanges) => {
                            info!(
                                "Alias '{}' was not added due to missing group '{:?}' not being added",
                                name, group_name
                            );
                            Ok(Outcome::NoChanges)
                        }
                        Ok(_) => unreachable!("Unexpected outcome encountered"),
                        Err(e) => panic!("Unexpected error encountered: {:?}", e),
                    }
                }
                _ => unreachable!("Unexpected error encountered: {:?}", e),
            }
        }
    }
}

pub fn is_valid_alias_name(name: &str) -> bool {
    // Alias name must not contain white space or '='
    !name.chars().any(|c| c.is_whitespace()) && !name.contains('=')
}

/// Handle the 'add' command
pub fn handle_add(
    catalog: &mut AliasCatalog,
    cmd: AddCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        // Add alias
        AddTarget::Alias(args) => {
            if args.global && *shell != ShellType::Zsh {
                error!("Global aliases are only supported in zsh.");
                return Err(Failure::UnsupportedGlobalAlias);
            }

            if !is_valid_alias_name(&args.name) {
                error!(
                    "Invalid alias name '{}'. Alias names must not contain whitespace or '='.",
                    args.name
                );
                return Err(Failure::InvalidAliasName);
            }

            let new_alias = Alias::new(args.command, args.group, !args.disabled, args.global);
            handle_add_alias(
                catalog,
                &args.name,
                &new_alias,
                prompt_overwrite_existing_alias,
                prompt_create_non_existent_group,
            )
        }

        // Add group
        AddTarget::Group(args) => add_group(catalog, &args.name, !args.disabled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    static SAMPLE_ALIAS_NAME: &str = "ll";

    fn sample_alias() -> Alias {
        Alias::new("ls -l".into(), None, true, false)
    }

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert(SAMPLE_ALIAS_NAME.into(), sample_alias());
        catalog
    }

    #[test]
    fn test_handle_add_alias_empty_catalog() {
        let mut catalog = AliasCatalog::new();
        let result = handle_add_alias(
            &mut catalog,
            SAMPLE_ALIAS_NAME,
            &sample_alias(),
            |_| false, // No overwrite needed
            |_| false, // No group creation needed
        );
        assert!(result.is_ok());
        assert_eq!(
            catalog.aliases.get(SAMPLE_ALIAS_NAME),
            Some(&sample_alias())
        );
    }

    #[test]
    fn test_handle_add_alias_overwrite_yes() {
        let mut catalog = sample_catalog();

        let mut new_alias = sample_alias();
        new_alias.command = "ls -la".into();

        // Mock user input to overwrite existing alias
        let result = handle_overwrite_existing_alias(
            &mut catalog,
            SAMPLE_ALIAS_NAME,
            &new_alias,
            true,      // Simulate user choosing to overwrite
            |_| false, // No group creation needed
        );

        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get(SAMPLE_ALIAS_NAME), Some(&new_alias));
    }

    #[test]
    fn test_handle_add_alias_no_overwrite() {
        let mut catalog = sample_catalog();

        let mut new_alias = sample_alias();
        new_alias.command = "ls -la".into();

        // Mock user input to not overwrite existing alias
        let result = handle_overwrite_existing_alias(
            &mut catalog,
            SAMPLE_ALIAS_NAME,
            &new_alias,
            false,     // Simulate user choosing not to overwrite
            |_| false, // No group creation needed
        );
        assert!(result.is_ok());
        assert_eq!(
            catalog.aliases.get(SAMPLE_ALIAS_NAME),
            Some(&sample_alias())
        );
    }

    #[test]
    fn test_handle_add_alias_overwrite_alias_move_group() {
        let mut catalog = AliasCatalog::new();
        let mut old_alias = sample_alias();
        old_alias.group = Some("old_group".into());

        let mut new_alias = sample_alias();
        new_alias.command = "ls -la".into();
        new_alias.group = Some("new_group".into());

        catalog.groups.insert("old_group".into(), true);
        catalog.groups.insert("new_group".into(), true);

        // Mock user input to overwrite existing alias and move to new group
        let result = handle_add_alias(
            &mut catalog,
            SAMPLE_ALIAS_NAME,
            &new_alias,
            |_| true,  // Simulate user choosing to overwrite
            |_| false, // No group creation needed
        );

        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get(SAMPLE_ALIAS_NAME), Some(&new_alias));
    }

    #[test]
    fn test_handle_add_alias_overwrite_to_nonexising_group() {
        let mut catalog = AliasCatalog::new();
        let mut old_alias = sample_alias();
        old_alias.group = Some("old_group".into());

        let mut new_alias = sample_alias();
        new_alias.command = "ls -la".into();
        new_alias.group = Some("new_group".into());

        catalog.groups.insert("old_group".into(), true);

        // Mock user input to overwrite existing alias and move to non-existent group
        let result = handle_add_alias(
            &mut catalog,
            SAMPLE_ALIAS_NAME,
            &new_alias,
            |_| true, // Simulate user choosing to overwrite
            |_| true, // Simulate user choosing to create group
        );

        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get(SAMPLE_ALIAS_NAME), Some(&new_alias));
        assert!(catalog.groups.contains_key("new_group"));
    }

    #[test]
    fn test_handle_add_alias_overwrite_to_nonexising_group_no_create() {
        let mut catalog = AliasCatalog::new();

        let mut old_alias = sample_alias();
        old_alias.group = Some("old_group".into());

        catalog
            .aliases
            .insert(SAMPLE_ALIAS_NAME.into(), old_alias.clone());
        catalog.groups.insert("old_group".into(), true);

        let mut new_alias = sample_alias();
        new_alias.command = "ls -la".into();
        new_alias.group = Some("new_group".into());

        // Mock user input to overwrite existing alias and move to non-existent group
        let result = handle_add_alias(
            &mut catalog,
            SAMPLE_ALIAS_NAME,
            &new_alias,
            |_| true,  // Simulate user choosing to overwrite
            |_| false, // Simulate user choosing not to create group
        );

        assert!(result.is_err());
        assert_eq!(catalog.aliases.get(SAMPLE_ALIAS_NAME), Some(&old_alias));
        assert!(!catalog.groups.contains_key("new_group"));
    }

    #[test]
    fn test_handle_add_group_success() {
        let mut catalog = AliasCatalog::new();
        let result = handle_add(
            &mut catalog,
            AddCommand {
                target: AddTarget::Group(crate::cli::add::AddGroupArgs {
                    name: "dev".into(),
                    disabled: false,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_ok());
        assert!(catalog.groups.contains_key("dev"));
    }

    #[test]
    fn test_handle_add_group_existing() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("utils".into(), true);
        let result = handle_add(
            &mut catalog,
            AddCommand {
                target: AddTarget::Group(crate::cli::add::AddGroupArgs {
                    name: "utils".into(),
                    disabled: false,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::GroupAlreadyExists);
        assert!(catalog.groups.contains_key("utils"));
    }

    #[test]
    fn test_handle_add_alias_unsupported_global_in_bash() {
        let mut catalog = AliasCatalog::new();
        let result = handle_add(
            &mut catalog,
            AddCommand {
                target: AddTarget::Alias(crate::cli::add::AddAliasArgs {
                    name: "ll".into(),
                    command: "ls -l".into(),
                    group: None,
                    disabled: false,
                    global: true,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::UnsupportedGlobalAlias);
        assert!(catalog.aliases.get("ll").is_none());
    }

    #[test]
    fn test_handle_add_alias_supported_global_in_zsh() {
        let mut catalog = AliasCatalog::new();
        let result = handle_add(
            &mut catalog,
            AddCommand {
                target: AddTarget::Alias(crate::cli::add::AddAliasArgs {
                    name: "ll".into(),
                    command: "ls -l".into(),
                    group: None,
                    disabled: false,
                    global: true,
                }),
            },
            &ShellType::Zsh,
        );
        assert!(result.is_ok());
        assert_eq!(
            catalog.aliases.get("ll"),
            Some(&Alias::new("ls -l".into(), None, true, true))
        );
    }

    #[test]
    fn test_valid_alias_name() {
        assert!(is_valid_alias_name("ll"));
        assert!(is_valid_alias_name("my_alias"));
        assert!(is_valid_alias_name("valid-alias_123"));
        assert!(!is_valid_alias_name("inavalid alias name"));
        assert!(!is_valid_alias_name("invalid\nalias"));
        assert!(!is_valid_alias_name("invalid\talias"));
        assert!(!is_valid_alias_name("invalid alias"));
        assert!(!is_valid_alias_name("another=invalid"));
        assert!(!is_valid_alias_name("white space"));
    }

    #[test]
    fn test_add_invalid_alias_name_space() {
        let mut catalog = AliasCatalog::new();
        let result = handle_add(
            &mut catalog,
            AddCommand {
                target: AddTarget::Alias(crate::cli::add::AddAliasArgs {
                    name: "invalid alias".into(),
                    command: "ls -l".into(),
                    group: None,
                    disabled: false,
                    global: false,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::InvalidAliasName);
        assert!(catalog.aliases.get("invalid alias").is_none());
    }

    #[test]
    fn test_add_invalid_alias_name_equal_sign() {
        let mut catalog = AliasCatalog::new();
        let result = handle_add(
            &mut catalog,
            AddCommand {
                target: AddTarget::Alias(crate::cli::add::AddAliasArgs {
                    name: "invalid=alias".into(),
                    command: "ls -l".into(),
                    group: None,
                    disabled: false,
                    global: false,
                }),
            },
            &ShellType::Bash,
        );
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::InvalidAliasName);
        assert!(catalog.aliases.get("invalid=alias").is_none());
    }
}
