//! Module for adding aliases and groups to the catalog.
//! Provides functions to add aliases and groups, handling cases where they
//! already exist or where groups do not exist.
//!
//! # Functions
//! - `add_alias`: Adds an alias to the catalog.
//! - `add_group`: Adds a group to the catalog.

use super::{Failure, Outcome};
use crate::catalog::types::{Alias, AliasCatalog};
use log::info;

/// Adds an alias to the catalog.
///
/// # Arguments
/// - `catalog`: The catalog to modify.
/// - `name`: The name of the alias.
/// - `command`: The command for the alias.
/// - `group`: An optional group for the alias.
/// - `enabled`: Whether the alias should be enabled.
///
/// # Returns
/// - `Outcome`: Result of the alias addition attempt.
/// - `Failure`: Error encountered during the process.
pub fn add_alias(
    catalog: &mut AliasCatalog,
    name: &str,
    alias: &Alias,
) -> Result<Outcome, Failure> {
    // Check if alias already exists
    if catalog.aliases.contains_key(name) {
        info!("Alias '{}' already exists.", name);
        return Err(Failure::AliasAlreadyExists);
    }

    if let Some(group_name) = &alias.group
        && !catalog.groups.contains_key(group_name)
    {
        info!("Group '{:?}' does not exist.", alias.group);
        return Err(Failure::GroupDoesNotExist);
    }

    catalog.aliases.insert(name.into(), alias.clone());

    info!("Alias '{}' added with command '{}'.", name, alias.command);
    Ok(Outcome::Command(add_alias_str(name, alias).to_string()))
}

pub fn add_alias_str(name: &str, alias: &Alias) -> String {
    format!(
        "alias{} -- '{}'='{}'",
        if alias.global { " -g" } else { "" },
        name,
        alias.command
    )
}

/// Adds a group to the catalog.
///
/// # Arguments
/// - `catalog`: The catalog to modify.
/// - `name`: The name of the group.
/// - `enabled`: Whether the group should be enabled.
///
/// # Returns
/// - `ReturnStatus`: Result of the group addition attempt.
/// - `ReturnError`: Error encountered during the process.
pub fn add_group(
    catalog: &mut AliasCatalog,
    name: &str,
    enabled: bool,
) -> Result<Outcome, Failure> {
    if catalog.groups.contains_key(name) {
        info!("Group '{}' already exists.", name);
        return Err(Failure::GroupAlreadyExists);
    }

    catalog.groups.insert(name.into(), enabled);
    info!("Group '{}' added with enabled status '{}'.", name, enabled);

    Ok(Outcome::CatalogChanged)
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
    fn add_alias_to_empty_catalog() {
        let mut catalog = AliasCatalog::new();
        let result = add_alias(&mut catalog, "ll", &test_alias());
        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get("ll"), Some(&test_alias()));
    }

    #[test]
    fn add_alias_to_existing_catalog() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert("ll".into(), test_alias());

        let mut new_alias = test_alias();
        new_alias.command = "git status".into();

        let result = add_alias(&mut catalog, "ll", &new_alias);
        assert!(result.is_err());
        assert_eq!(catalog.aliases.get("ll"), Some(&test_alias()));
        assert_ne!(catalog.aliases.get("ll"), Some(&new_alias));
    }

    #[test]
    fn add_disabled_alias() {
        let mut catalog = AliasCatalog::new();
        let mut new_alias = test_alias();
        new_alias.enabled = false;

        let result = add_alias(&mut catalog, "ll", &new_alias);
        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get("ll"), Some(&new_alias));
        assert_ne!(catalog.aliases.get("ll"), Some(&test_alias()));
    }

    #[test]
    fn add_existing_alias() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert("ll".into(), test_alias());
        let result = add_alias(&mut catalog, "ll", &test_alias());
        assert!(result.is_err());
    }

    #[test]
    fn add_alias_to_nonexistent_group() {
        let mut catalog = AliasCatalog::new();
        let mut new_alias = test_alias();
        new_alias.group = Some("nonexistent_group".into());

        let result = add_alias(&mut catalog, "ll", &new_alias);
        assert!(result.is_err());
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn add_alias_to_existing_group() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("file_ops".into(), true);
        let mut new_alias = test_alias();
        new_alias.group = Some("file_ops".into());

        let result = add_alias(&mut catalog, "ll", &new_alias);
        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get("ll"), Some(&new_alias));
        assert!(catalog.groups.contains_key("file_ops"));
    }

    #[test]
    fn add_group_to_new_catalog() {
        let mut catalog = AliasCatalog::new();
        let result = add_group(&mut catalog, "dev_tools", true);
        assert!(result.is_ok());
        assert!(catalog.groups.contains_key("dev_tools"));
    }

    #[test]
    fn add_group_to_existing_catalog() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("utils".into(), true);
        let result = add_group(&mut catalog, "dev_tools", true);
        assert!(result.is_ok());
        assert!(catalog.groups.contains_key("dev_tools"));
        assert!(catalog.groups.contains_key("utils"));
    }

    #[test]
    fn add_existing_group() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("dev_tools".into(), true);
        let result = add_group(&mut catalog, "dev_tools", true);
        assert!(result.is_err());
        assert_matches!(result, Err(Failure::GroupAlreadyExists));
    }

    #[test]
    fn add_disabled_group() {
        let mut catalog = AliasCatalog::new();
        let result = add_group(&mut catalog, "dev_tools", false);
        assert!(result.is_ok());
        assert_eq!(catalog.groups.get("dev_tools"), Some(&false));
    }

    #[test]
    fn add_enabled_group() {
        let mut catalog = AliasCatalog::new();
        let result = add_group(&mut catalog, "dev_tools", true);
        assert!(result.is_ok());
        assert_eq!(catalog.groups.get("dev_tools"), Some(&true));
    }

    #[test]
    fn add_global_alias() {
        let mut catalog = AliasCatalog::new();
        let mut new_alias = test_alias();
        new_alias.global = true;

        let result = add_alias(&mut catalog, "ll", &new_alias);
        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get("ll"), Some(&new_alias));
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
