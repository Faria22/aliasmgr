//! Module for adding aliases and groups to the configuration.
//! Provides functions to add aliases and groups, handling cases where they
//! already exist or where groups do not exist.
//!
//! # Functions
//! - `add_alias`: Adds an alias to the configuration.
//! - `add_group`: Adds a group to the configuration.

use crate::config::types::{Alias, Config};
use crate::core::edit::edit_alias;
use crate::core::{Failure, Outcome};
use log::info;

/// Handles the case where an alias already exists.
/// Prompts the user to overwrite the existing alias.
///
/// # Arguments
/// - `config`: The configuration to modify.
/// - `name`: The name of the existing alias.
/// - `command`: The command for the alias.
///
/// # Returns
/// - `Outcome`: Result of the alias addition attempt.
/// - `Failure`: Error encountered during the process.
fn handle_existing_alias(
    config: &mut Config,
    name: &str,
    command: &str,
) -> Result<Outcome, Failure> {
    println!("Alias '{}' already exists.", name);
    println!("Would you like to overwrite it? (y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() == "y" {
        info!("Overwriting alias '{}'.", name);
        return edit_alias(config, name, command);
    } else if input.trim().to_lowercase() != "n" && !input.trim().is_empty() {
        eprintln!("Invalid input. Alias '{}' was not modified.", name);
        return Err(Failure::InvalidInput(input.trim().to_string()));
    }

    info!("Alias '{}' was not modified.", name);
    Ok(Outcome::NoChanges)
}

/// Handles the case where a specified group does not exist.
/// Prompts the user to create the group.
///
/// # Arguments
/// - `config`: The configuration to modify.
/// - `alias`: The name of the alias to add.
/// - `command`: The command for the alias.
/// - `group`: The name of the non-existing group.
/// - `enabled`: Whether the alias should be enabled.
///
/// # Returns
/// - `Outcome`: Result of the alias addition attempt.
/// - `Failure`: Error encountered during the process.
fn handle_non_existing_group(
    config: &mut Config,
    alias: &str,
    command: &str,
    group: &str,
    enabled: bool,
) -> Result<Outcome, Failure> {
    println!("Group '{}' does not exist.", group);
    println!("Would you like to create it? (y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() == "y" {
        info!("Creating group '{}'.", group);
        add_group(config, group, enabled)?;
        return add_alias(config, alias, command, Some(group), enabled);
    }

    if input.trim().to_lowercase() == "n" {
        info!(
            "Alias '{}' was not added due to missing group '{}' not being added",
            alias, group
        );
        return Ok(Outcome::NoChanges);
    }

    eprintln!("Invalid input. Alias '{}' was not added.", alias);
    return Err(Failure::InvalidInput(input.trim().to_string()));
}

/// Adds an alias to the configuration.
///
/// # Arguments
/// - `config`: The configuration to modify.
/// - `name`: The name of the alias.
/// - `command`: The command for the alias.
/// - `group`: An optional group for the alias.
/// - `enabled`: Whether the alias should be enabled.
///
/// # Returns
/// - `Outcome`: Result of the alias addition attempt.
/// - `Failure`: Error encountered during the process.
pub fn add_alias(
    config: &mut Config,
    name: &str,
    command: &str,
    group: Option<&str>,
    enabled: bool,
) -> Result<Outcome, Failure> {
    add_alias_to_config(config, name, command, group, enabled).or_else(|e| match e {
        Failure::AliasAlreadyExists => handle_existing_alias(config, name, command),
        Failure::GroupDoesNotExist => handle_non_existing_group(
            config,
            name,
            command,
            group.expect("group cannot be None in this arm"),
            enabled,
        ),
        _ => unreachable!("Unexpected error when adding alias"),
    })
}

/// Adds a group to the configuration.
///
/// # Arguments
/// - `config`: The configuration to modify.
/// - `name`: The name of the group.
/// - `enabled`: Whether the group should be enabled.
///
/// # Returns
/// - `ReturnStatus`: Result of the group addition attempt.
/// - `ReturnError`: Error encountered during the process.
pub fn add_group(config: &mut Config, name: &str, enabled: bool) -> Result<Outcome, Failure> {
    add_group_to_config(config, name, enabled).or_else(|e| match e {
        Failure::GroupAlreadyExists => {
            println!("Group '{}' already exists. No changes made.", name);
            return Ok(Outcome::NoChanges);
        }
        _ => unreachable!("Unexpected error when adding group"),
    })
}

/// Internal function to add an alias to the configuration.
///
/// # Arguments
/// - `config`: The configuration to modify.
/// - `alias`: The name of the alias.
/// - `command`: The command for the alias.
/// - `group`: An optional group for the alias.
/// - `enabled`: Whether the alias should be enabled.
///
///# Returns
/// - `Result<Outcome, Failure>`: Ok if the alias was added successfully, Err otherwise.
fn add_alias_to_config(
    config: &mut Config,
    alias: &str,
    command: &str,
    group: Option<&str>,
    enabled: bool,
) -> Result<Outcome, Failure> {
    // Check if alias already exists
    if config.aliases.contains_key(alias) {
        info!("Alias '{}' already exists.", alias);
        return Err(Failure::AliasAlreadyExists);
    }

    if group.is_some_and(|g| !config.groups.contains_key(g)) {
        info!("Group '{:?}' does not exist.", group);
        return Err(Failure::GroupDoesNotExist);
    }

    config.aliases.insert(
        alias.into(),
        Alias::new(
            command.into(),
            enabled,
            group.map(|g| g.to_string()),
            if enabled { false } else { true },
        ),
    );

    info!("Alias '{}' added with command '{}'.", alias, command);
    Ok(Outcome::Command(format!("alias {}='{}'", alias, command)))
}

/// Internal function to add a group to the configuration.
///
/// # Arguments
/// - `config`: The configuration to modify.
/// - `group`: The name of the group.
/// - `enabled`: Whether the group should be enabled.
///
/// # Returns
/// - `Result<Outcome, Failure>`: Ok if the group was added successfully, Err otherwise.
fn add_group_to_config(
    config: &mut Config,
    group: &str,
    enabled: bool,
) -> Result<Outcome, Failure> {
    if config.groups.contains_key(group) {
        info!("Group '{}' already exists.", group);
        return Err(Failure::GroupAlreadyExists);
    }

    config.groups.insert(group.into(), enabled);
    info!("Group '{}' added with enabled status '{}'.", group, enabled);

    Ok(Outcome::ConfigChanged)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_alias_to_empty_config() {
        let mut config = Config::new();
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, true);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new("ls -la".into(), true, None, false))
        );
    }

    #[test]
    fn add_alias_to_existing_config() {
        let mut config = Config::new();
        config.aliases.insert(
            "gs".into(),
            Alias::new("git status".into(), true, Some("git".into()), false),
        );
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, true);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("gs"),
            Some(&Alias::new(
                "git status".into(),
                true,
                Some("git".into()),
                false
            ))
        );
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new("ls -la".into(), true, None, false))
        );
    }

    #[test]
    fn add_disabled_alias() {
        let mut config = Config::new();
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, false);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new("ls -la".into(), false, None, true))
        );
    }

    #[test]
    fn add_existing_alias() {
        let mut config = Config::new();
        config
            .aliases
            .insert("ll".into(), Alias::new("ls -l".into(), true, None, false));
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, true);
        assert!(result.is_err());
    }

    #[test]
    fn add_alias_to_nonexistent_group() {
        let mut config = Config::new();
        let result = add_alias_to_config(&mut config, "ll", "ls -la", Some("nonexistent"), true);
        assert!(result.is_err());
        assert!(matches!(result, Err(Failure::GroupDoesNotExist)));
    }

    #[test]
    fn add_alias_to_existing_group() {
        let mut config = Config::new();
        config.groups.insert("file_ops".into(), true);
        let result = add_alias_to_config(&mut config, "ll", "ls -la", Some("file_ops"), true);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new(
                "ls -la".into(),
                true,
                Some("file_ops".into()),
                false
            ))
        );
        assert!(config.groups.contains_key("file_ops"))
    }

    #[test]
    fn add_group_to_new_config() {
        let mut config = Config::new();
        let result = add_group_to_config(&mut config, "dev_tools", true);
        assert!(result.is_ok());
        assert!(config.groups.contains_key("dev_tools"));
    }

    #[test]
    fn add_group_to_existing_config() {
        let mut config = Config::new();
        config.groups.insert("utils".into(), true);
        let result = add_group_to_config(&mut config, "dev_tools", true);
        assert!(result.is_ok());
        assert!(config.groups.contains_key("dev_tools"));
        assert!(config.groups.contains_key("utils"));
    }

    #[test]
    fn add_existing_group() {
        let mut config = Config::new();
        config.groups.insert("dev_tools".into(), true);
        let result = add_group_to_config(&mut config, "dev_tools", true);
        assert!(result.is_err());
        assert!(matches!(result, Err(Failure::GroupAlreadyExists)));
    }

    #[test]
    fn add_disabled_group() {
        let mut config = Config::new();
        let result = add_group_to_config(&mut config, "dev_tools", false);
        assert!(result.is_ok());
        assert_eq!(config.groups.get("dev_tools"), Some(&false));
    }
}
