use crate::core::{Failure, Outcome};

use crate::core::add::{add_alias, add_group};
use crate::core::edit::edit_alias;
use crate::core::r#move::move_alias;

use crate::config::types::Config;

use crate::cli::add::{AddCommand, AddTarget};
use crate::cli::interaction::{prompt_create_non_existent_group, prompt_overwrite_existing_alias};

use log::info;

/// Handle overwriting an existing alias
fn handle_overwrite_existing_alias(
    config: &mut Config,
    name: &str,
    command: &str,
    group: Option<&str>,
    enabled: bool,
    overwrite: bool,
    create_group: impl Fn(&str) -> bool,
) -> Result<Outcome, Failure> {
    // If the alias already exists, we check if the user wants to overwrite it
    if overwrite {
        // Move alias to new group if it is different from the previous one
        if group != config.aliases.get(name).and_then(|a| a.group.as_deref()) {
            info!("Moving alias '{}' to group '{:?}'.", name, group);
            let group = group.map(|g| g.to_string());

            if let Err(Failure::GroupDoesNotExist) = move_alias(config, name, &group) {
                // If the group does not exist, we ask the user if they want to create it
                let group = group.expect("group has to be `Some` for this error to arise");
                handle_create_non_existent_group(config, &group, create_group(&group))?;

                // Retry moving the alias after creating the group
                move_alias(config, name, &Some(group))?;
            }
        }

        // User wants to overwrite the existing alias
        info!("Overwriting existing alias '{}'.", name);
        let command = edit_alias(config, name, command)?;

        // Update enabled status if it is different from the previous one
        if enabled != config.aliases.get(name).unwrap().enabled {
            let alias = config.aliases.get_mut(name).unwrap();
            alias.enabled = enabled;
            info!(
                "Setting alias '{}' enabled status to '{}'.",
                name, alias.enabled
            );
        }

        // Returns command to edit the alias in the shell
        Ok(command)
    } else {
        // User does not want to overwrite the existing alias
        info!("Not overwriting existing alias '{}'.", name);
        Ok(Outcome::NoChanges)
    }
}

/// Handle adding non-existent group
fn handle_create_non_existent_group(
    config: &mut Config,
    name: &str,
    create_group: bool,
) -> Result<Outcome, Failure> {
    if create_group {
        // User wants to create the group
        info!("Creating group '{}'.", name);
        add_group(config, name, true)
    } else {
        // User does not want to create the group
        info!("Group '{:?}' was not added", name);
        Ok(Outcome::NoChanges)
    }
}

/// Handle add alias
fn handle_add_alias(
    config: &mut Config,
    name: &str,
    command: &str,
    group: Option<&str>,
    enabled: bool,
    overwrite: impl Fn(&str) -> bool,
    create_group: impl Fn(&str) -> bool,
) -> Result<Outcome, Failure> {
    match add_alias(config, name, command, group, enabled) {
        // Alias added successfully
        Ok(outcome) => Ok(outcome),

        // Handle errors
        Err(e) => {
            match e {
                // Alias already exists
                Failure::AliasAlreadyExists => handle_overwrite_existing_alias(
                    config,
                    name,
                    command,
                    group,
                    enabled,
                    overwrite(name),
                    // Closure to create non-existent group if needed
                    create_group,
                ),

                // Group that alias will belong to does not exist
                Failure::GroupDoesNotExist => {
                    let group_name =
                        group.expect("group has to be `Some` for these error to arise");
                    match handle_create_non_existent_group(
                        config,
                        group_name,
                        create_group(group_name),
                    ) {
                        // Group created successfully
                        Ok(Outcome::ConfigChanged) => {
                            // Retry adding the alias after creating the group
                            add_alias(config, name, command, Some(group_name), enabled)?;
                            Ok(Outcome::ConfigChanged)
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

/// Handle the 'add' command
pub fn handle_add(config: &mut Config, cmd: AddCommand) -> Result<Outcome, Failure> {
    match cmd.target {
        // Add alias
        AddTarget::Alias(args) => handle_add_alias(
            config,
            &args.name,
            &args.command,
            args.group.as_deref(),
            !args.disabled,
            prompt_overwrite_existing_alias,
            prompt_create_non_existent_group,
        ),

        // Add group
        AddTarget::Group(args) => add_group(config, &args.name, !args.disabled),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Alias;
    use assert_matches::assert_matches;

    #[test]
    fn test_handle_add_alias_success() {
        let mut config = Config::new();
        let result = handle_add_alias(
            &mut config,
            "ll",
            "ls -la",
            None,
            true,
            |_| false, // No overwrite needed
            |_| false, // No group creation needed
        );
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new("ls -la".into(), true, None, false))
        );
    }

    #[test]
    fn test_handle_add_alias_overwrite_yes() {
        let mut config = Config::new();
        config
            .aliases
            .insert("ll".into(), Alias::new("ls -l".into(), true, None, false));

        // Mock user input to overwrite existing alias
        let result = handle_overwrite_existing_alias(
            &mut config,
            "ll",
            "ls -la",
            None,
            true,
            true,      // Simulate user choosing to overwrite
            |_| false, // No group creation needed
        );

        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new("ls -la".into(), true, None, false))
        );
    }

    #[test]
    fn test_handle_add_alias_overwrite_no() {
        let mut config = Config::new();
        config
            .aliases
            .insert("ll".into(), Alias::new("ls -l".into(), true, None, false));

        // Mock user input to not overwrite existing alias
        let result = handle_overwrite_existing_alias(
            &mut config,
            "ll",
            "ls -la",
            None,
            true,
            false,     // Simulate user choosing not to overwrite
            |_| false, // No group creation needed
        );
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new("ls -l".into(), true, None, false))
        );
    }

    #[test]
    fn test_handle_add_alias_overwrite_alias_move_group() {
        let mut config = Config::new();
        config.aliases.insert(
            "ll".into(),
            Alias::new("ls -l".into(), true, Some("old_group".into()), false),
        );
        config.groups.insert("old_group".into(), true);
        config.groups.insert("new_group".into(), true);

        // Mock user input to overwrite existing alias and move to new group
        let result = handle_add_alias(
            &mut config,
            "ll",
            "ls -la",
            Some("new_group"),
            true,
            |_| true,  // Simulate user choosing to overwrite
            |_| false, // No group creation needed
        );

        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new(
                "ls -la".into(),
                true,
                Some("new_group".into()),
                false
            ))
        );
    }

    #[test]
    fn test_handle_add_alias_overwrite_to_nonexising_group() {
        let mut config = Config::new();
        config.aliases.insert(
            "ll".into(),
            Alias::new("ls -l".into(), true, Some("old_group".into()), false),
        );
        config.groups.insert("old_group".into(), true);

        // Mock user input to overwrite existing alias and move to non-existent group
        let result = handle_add_alias(
            &mut config,
            "ll",
            "ls -la",
            Some("new_group"),
            true,
            |_| true, // Simulate user choosing to overwrite
            |_| true, // Simulate user choosing to create group
        );

        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new(
                "ls -la".into(),
                true,
                Some("new_group".into()),
                false
            ))
        );
        assert!(config.groups.contains_key("new_group"));
    }

    #[test]
    fn test_handle_add_alias_overwrite_to_nonexising_group_no_create() {
        let mut config = Config::new();
        config.aliases.insert(
            "ll".into(),
            Alias::new("ls -l".into(), true, Some("old_group".into()), false),
        );
        config.groups.insert("old_group".into(), true);

        // Mock user input to overwrite existing alias and move to non-existent group
        let result = handle_add_alias(
            &mut config,
            "ll",
            "ls -la",
            Some("new_group"),
            true,
            |_| true,  // Simulate user choosing to overwrite
            |_| false, // Simulate user choosing not to create group
        );

        assert!(result.is_err());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new(
                "ls -l".into(),
                true,
                Some("old_group".into()),
                false
            ))
        );
        assert!(!config.groups.contains_key("new_group"));
    }

    #[test]
    fn test_handle_overwrite_alias_different_enabled_status() {
        let mut config = Config::new();
        config
            .aliases
            .insert("ll".into(), Alias::new("ls -l".into(), false, None, true));

        // Mock user input to overwrite existing alias and change enabled status
        let result = handle_overwrite_existing_alias(
            &mut config,
            "ll",
            "ls -la",
            None,
            true,      // New enabled status
            true,      // Simulate user choosing to overwrite
            |_| false, // No group creation needed
        );

        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new("ls -la".into(), true, None, true))
        );
    }

    #[test]
    fn test_handle_add_alias_create_group_no_overwrite() {
        let mut config = Config::new();

        // Mock user input to not create non-existent group
        let result = handle_add_alias(
            &mut config,
            "ll",
            "ls -la",
            Some("utils"),
            true,
            |_| false, // No overwrite needed
            |_| false, // Simulate user choosing not to create group
        );

        assert!(result.is_ok());
        assert!(!config.aliases.contains_key("ll"));
    }

    #[test]
    fn test_handle_add_alias_create_group() {
        let mut config = Config::new();

        // Mock user input to create non-existent group
        let result = handle_add_alias(
            &mut config,
            "ll",
            "ls -la",
            Some("utils"),
            true,
            |_| false, // No overwrite needed
            |_| true,  // Simulate user choosing to create group
        );

        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new(
                "ls -la".into(),
                true,
                Some("utils".into()),
                false
            ))
        );
    }

    #[test]
    fn test_handle_add_group_success() {
        let mut config = Config::new();
        let result = handle_add(
            &mut config,
            AddCommand {
                target: AddTarget::Group(crate::cli::add::AddGroupArgs {
                    name: "dev".into(),
                    disabled: false,
                }),
            },
        );
        assert!(result.is_ok());
        assert!(config.groups.contains_key("dev"));
    }

    #[test]
    fn test_handle_add_group_existing() {
        let mut config = Config::new();
        config.groups.insert("utils".into(), true);
        let result = handle_add(
            &mut config,
            AddCommand {
                target: AddTarget::Group(crate::cli::add::AddGroupArgs {
                    name: "utils".into(),
                    disabled: false,
                }),
            },
        );
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::GroupAlreadyExists);
        assert!(config.groups.contains_key("utils"));
    }
}
