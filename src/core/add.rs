//! Module for adding aliases and groups to the configuration.
//! Provides functions to add aliases and groups, handling cases where they
//! already exist or where groups do not exist.
//!
//! # Functions
//! - `add_alias`: Adds an alias to the configuration.
//! - `add_group`: Adds a group to the configuration.

use super::{Failure, Outcome};
use crate::config::types::{Alias, Config};
use log::info;

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
    // Check if alias already exists
    if config.aliases.contains_key(name) {
        info!("Alias '{}' already exists.", name);
        return Err(Failure::AliasAlreadyExists);
    }

    if group.is_some_and(|g| !config.groups.contains_key(g)) {
        info!("Group '{:?}' does not exist.", group);
        return Err(Failure::GroupDoesNotExist);
    }

    config.aliases.insert(
        name.into(),
        Alias::new(
            command.into(),
            enabled,
            group.map(|g| g.to_string()),
            !enabled, // detailed output needs to be true if alias is disabled, and can be false otherwise
        ),
    );

    info!("Alias '{}' added with command '{}'.", name, command);
    Ok(Outcome::Command(format!("alias {}='{}'", name, command)))
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
    if config.groups.contains_key(name) {
        info!("Group '{}' already exists.", name);
        return Err(Failure::GroupAlreadyExists);
    }

    config.groups.insert(name.into(), enabled);
    info!("Group '{}' added with enabled status '{}'.", name, enabled);

    Ok(Outcome::ConfigChanged)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn add_alias_to_empty_config() {
        let mut config = Config::new();
        let result = add_alias(&mut config, "ll", "ls -la", None, true);
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
        let result = add_alias(&mut config, "ll", "ls -la", None, true);
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
        let result = add_alias(&mut config, "ll", "ls -la", None, false);
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
        let result = add_alias(&mut config, "ll", "ls -la", None, true);
        assert!(result.is_err());
    }

    #[test]
    fn add_alias_to_nonexistent_group() {
        let mut config = Config::new();
        let result = add_alias(&mut config, "ll", "ls -la", Some("nonexistent"), true);
        assert!(result.is_err());
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn add_alias_to_existing_group() {
        let mut config = Config::new();
        config.groups.insert("file_ops".into(), true);
        let result = add_alias(&mut config, "ll", "ls -la", Some("file_ops"), true);
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
        let result = add_group(&mut config, "dev_tools", true);
        assert!(result.is_ok());
        assert!(config.groups.contains_key("dev_tools"));
    }

    #[test]
    fn add_group_to_existing_config() {
        let mut config = Config::new();
        config.groups.insert("utils".into(), true);
        let result = add_group(&mut config, "dev_tools", true);
        assert!(result.is_ok());
        assert!(config.groups.contains_key("dev_tools"));
        assert!(config.groups.contains_key("utils"));
    }

    #[test]
    fn add_existing_group() {
        let mut config = Config::new();
        config.groups.insert("dev_tools".into(), true);
        let result = add_group(&mut config, "dev_tools", true);
        assert!(result.is_err());
        assert_matches!(result, Err(Failure::GroupAlreadyExists));
    }

    #[test]
    fn add_disabled_group() {
        let mut config = Config::new();
        let result = add_group(&mut config, "dev_tools", false);
        assert!(result.is_ok());
        assert_eq!(config.groups.get("dev_tools"), Some(&false));
    }
}
