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
pub fn add_alias(config: &mut Config, name: &str, alias: &Alias) -> Result<Outcome, Failure> {
    // Check if alias already exists
    if config.aliases.contains_key(name) {
        info!("Alias '{}' already exists.", name);
        return Err(Failure::AliasAlreadyExists);
    }

    if let Some(group_name) = &alias.group
        && !config.groups.contains_key(group_name)
    {
        info!("Group '{:?}' does not exist.", alias.group);
        return Err(Failure::GroupDoesNotExist);
    }

    config.aliases.insert(name.into(), alias.clone());

    info!("Alias '{}' added with command '{}'.", name, alias.command);
    Ok(Outcome::Command(format!("{}", add_alias_str(name, alias))))
}

pub fn add_alias_str(name: &str, alias: &Alias) -> String {
    format!(
        "alias{} -- '{}'='{}'",
        if alias.global { " -g" } else { "" },
        name,
        alias.command
    )
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

    fn test_alias() -> Alias {
        // No group, enabled, not detailed, not global
        Alias::new("ls -la".into(), None, true, false)
    }

    #[test]
    fn add_alias_to_empty_config() {
        let mut config = Config::new();
        let result = add_alias(&mut config, "ll", &test_alias());
        assert!(result.is_ok());
        assert_eq!(config.aliases.get("ll"), Some(&test_alias()));
    }

    #[test]
    fn add_alias_to_existing_config() {
        let mut config = Config::new();
        config.aliases.insert("ll".into(), test_alias());

        let mut new_alias = test_alias();
        new_alias.command = "git status".into();

        let result = add_alias(&mut config, "ll", &new_alias);
        assert!(result.is_err());
        assert_eq!(config.aliases.get("ll"), Some(&test_alias()));
        assert_ne!(config.aliases.get("ll"), Some(&new_alias));
    }

    #[test]
    fn add_disabled_alias() {
        let mut config = Config::new();
        let mut new_alias = test_alias();
        new_alias.enabled = false;

        let result = add_alias(&mut config, "ll", &new_alias);
        assert!(result.is_ok());
        assert_eq!(config.aliases.get("ll"), Some(&new_alias));
        assert_ne!(config.aliases.get("ll"), Some(&test_alias()));
    }

    #[test]
    fn add_existing_alias() {
        let mut config = Config::new();
        config.aliases.insert("ll".into(), test_alias());
        let result = add_alias(&mut config, "ll", &test_alias());
        assert!(result.is_err());
    }

    #[test]
    fn add_alias_to_nonexistent_group() {
        let mut config = Config::new();
        let mut new_alias = test_alias();
        new_alias.group = Some("nonexistent_group".into());

        let result = add_alias(&mut config, "ll", &new_alias);
        assert!(result.is_err());
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn add_alias_to_existing_group() {
        let mut config = Config::new();
        config.groups.insert("file_ops".into(), true);
        let mut new_alias = test_alias();
        new_alias.group = Some("file_ops".into());

        let result = add_alias(&mut config, "ll", &new_alias);
        assert!(result.is_ok());
        assert_eq!(config.aliases.get("ll"), Some(&new_alias));
        assert!(config.groups.contains_key("file_ops"));
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

    #[test]
    fn add_enabled_group() {
        let mut config = Config::new();
        let result = add_group(&mut config, "dev_tools", true);
        assert!(result.is_ok());
        assert_eq!(config.groups.get("dev_tools"), Some(&true));
    }

    #[test]
    fn add_global_alias() {
        let mut config = Config::new();
        let mut new_alias = test_alias();
        new_alias.global = true;

        let result = add_alias(&mut config, "ll", &new_alias);
        assert!(result.is_ok());
        assert_eq!(config.aliases.get("ll"), Some(&new_alias));
    }

    #[test]
    fn add_string_global_alias() {
        let alias = Alias::new("ls -la".into(), None, true, true);
        let result = add_alias_str("ll", &alias);
        assert_eq!(result, "alias -g -- 'll'='ls -la'");
    }

    #[test]
    fn add_string_non_global_alias() {
        let alias = Alias::new("ls -la".into(), None, true, false);
        let result = add_alias_str("ll", &alias);
        assert_eq!(result, "alias -- 'll'='ls -la'");
    }
}
