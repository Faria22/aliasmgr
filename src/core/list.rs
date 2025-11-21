//! Functions for listing alias groups and their aliases.
//! This module provides functionality to retrieve all alias groups
//! and the aliases associated with them from a given configuration.
//! It supports both grouped and ungrouped aliases.
//!
//! # Functions
//! - `get_all_groups`: Returns a mapping of all groups to their aliases.
//! - `get_single_group`: Returns aliases for a specific group.

use crate::config::types::Config;
use crate::core::Failure;
use std::collections::HashMap;
use std::vec::Vec;

/// Identifier for a group, either named or ungrouped.
#[derive(Eq, Hash, PartialEq)]
pub enum GroupId {
    Ungrouped,
    Named(String),
}

/// Retrieves all alias groups and their associated aliases from the configuration.
/// # Arguments
/// * `config` - A reference to the configuration containing groups and aliases.
///
/// # Returns
/// A HashMap where keys are GroupId (either named or ungrouped) and values
/// are vectors of alias names belonging to those groups.
pub fn get_all_groups(config: &Config) -> HashMap<GroupId, Vec<String>> {
    let mut groups = HashMap::<GroupId, Vec<String>>::new();

    // Initialize the groups with empty vectors
    groups.insert(GroupId::Ungrouped, Vec::new());
    for group_name in config.groups.keys() {
        groups.insert(GroupId::Named(group_name.clone()), Vec::new());
    }

    // Populate the groups with alias names
    for (alias_name, alias) in &config.aliases {
        let key = alias
            .group
            .clone()
            .map(GroupId::Named)
            .unwrap_or(GroupId::Ungrouped);
        groups
            .get_mut(&key)
            .expect("group is in alias, but not in the group vector")
            .push(alias_name.clone());
    }

    groups
}

/// Retrieves aliases for a specific group from the configuration.
/// # Arguments
/// * `config` - A reference to the configuration containing groups and aliases.
/// * `name` - `GroupId` specifying the group to retrieve aliases for.
///
/// # Returns
/// A vector of alias names belonging to the specified group.
pub fn get_single_group(config: &Config, identifier: GroupId) -> Result<Vec<String>, Failure> {
    if let GroupId::Named(name) = &identifier {
        if !config.groups.contains_key(name) {
            return Err(Failure::GroupDoesNotExist);
        }

        return Ok(config
            .aliases
            .iter()
            .filter(|(_, alias)| alias.group.as_ref() == Some(name))
            .map(|(alias_name, _)| alias_name.clone())
            .collect());
    }

    Ok(config
        .aliases
        .iter()
        .filter(|(_, alias)| alias.group.is_none())
        .map(|(alias_name, _)| alias_name.clone())
        .collect())
}

/// Retrieves all disabled aliases grouped by their respective groups from the configuration.
///
/// # Arguments
/// * `config` - A reference to the configuration containing groups and aliases.
///
/// # Returns
/// A HashMap where keys are GroupId (either named or ungrouped) and values
/// are vectors of disabled alias names belonging to those groups.
pub fn get_disabled_aliases_grouped(config: &Config) -> HashMap<GroupId, Vec<String>> {
    let mut groups = get_all_groups(config);
    groups.retain(|group_name, group| {
        if let GroupId::Named(g) = group_name {
            // Add entire group if it's disabled
            return !config.groups.get(g).unwrap_or(&true);
        }

        // Retain only disabled aliases
        group.retain(|alias| !config.aliases[alias].enabled);
        true
    });

    groups
}

/// Retrieves all enabled aliases grouped by their respective groups from the configuration.
///
/// # Arguments
/// * `config` - A reference to the configuration containing groups and aliases.
///
/// # Returns
/// A HashMap where keys are GroupId (either named or ungrouped) and values
/// are vectors of enabled alias names belonging to those groups.
pub fn get_enabled_aliases_grouped(config: &Config) -> HashMap<GroupId, Vec<String>> {
    let mut groups = get_all_groups(config);
    groups.retain(|group_name, group| {
        // Skip entire group if it's disabled
        if let GroupId::Named(g) = group_name
            && !config.groups.get(g).unwrap_or(&true)
        {
            return false;
        }

        // Retain only enabled aliases
        group.retain(|alias| config.aliases[alias].enabled);
        true
    });

    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Alias;

    fn create_test_config() -> Config {
        let mut groups = HashMap::new();
        let mut aliases = HashMap::new();

        groups.insert("group1".into(), true);
        groups.insert("group2".into(), true);
        groups.insert("group3".into(), false);

        // Named group with enabled and disabled aliases
        aliases.insert(
            "alias1".into(),
            Alias {
                command: "cmd1".into(),
                group: Some("group1".into()),
                detailed: false,
                enabled: true,
            },
        );
        aliases.insert(
            "alias1_disabled".into(),
            Alias {
                command: "cmd1".into(),
                group: Some("group1".into()),
                detailed: true,
                enabled: false,
            },
        );

        // Another named group with only enabled aliases
        aliases.insert(
            "alias2".into(),
            Alias {
                command: "cmd2".into(),
                group: Some("group2".into()),
                detailed: false,
                enabled: true,
            },
        );
        //
        // Named group that is disabled with enabled and disabled aliases
        aliases.insert(
            "alias3".into(),
            Alias {
                command: "cmd3".into(),
                group: Some("group3".into()),
                detailed: false,
                enabled: true,
            },
        );
        aliases.insert(
            "alias3_disabled".into(),
            Alias {
                command: "cmd3".into(),
                group: Some("group3".into()),
                detailed: true,
                enabled: false,
            },
        );

        // Ungrouped aliases
        aliases.insert(
            "alias4".into(),
            Alias {
                command: "cmd4".into(),
                group: None,
                detailed: false,
                enabled: true,
            },
        );
        aliases.insert(
            "alias4_disabled".into(),
            Alias {
                command: "cmd4".into(),
                group: None,
                detailed: false,
                enabled: false,
            },
        );

        Config { groups, aliases }
    }

    #[test]
    fn test_get_single_group() {
        let config = create_test_config();

        let group = get_single_group(&config, GroupId::Named("group1".into()));

        assert!(group.is_ok());
        let group = group.unwrap();

        assert_eq!(group.len(), 2);

        assert!(group.contains(&"alias1".to_string()));
        assert!(group.contains(&"alias1_disabled".to_string()));

        assert!(!group.contains(&"alias2".to_string()));
        assert!(!group.contains(&"alias3".to_string()));
        assert!(!group.contains(&"alias3_disabled".to_string()));
        assert!(!group.contains(&"alias4".to_string()));
        assert!(!group.contains(&"alias4_disabled".to_string()));
    }

    #[test]
    fn test_get_ungrouped_aliases() {
        let config = create_test_config();

        let ungrouped = get_single_group(&config, GroupId::Ungrouped);

        assert!(ungrouped.is_ok());
        let ungrouped = ungrouped.unwrap();

        assert_eq!(ungrouped.len(), 2);

        assert!(ungrouped.contains(&"alias4".to_string()));
        assert!(ungrouped.contains(&"alias4_disabled".to_string()));

        assert!(!ungrouped.contains(&"alias1".to_string()));
        assert!(!ungrouped.contains(&"alias1_disabled".to_string()));
        assert!(!ungrouped.contains(&"alias2".to_string()));
        assert!(!ungrouped.contains(&"alias3".to_string()));
        assert!(!ungrouped.contains(&"alias3_disabled".to_string()));
    }

    #[test]
    fn test_get_nonexistent_group() {
        let config = Config::new();
        let group = get_single_group(&config, GroupId::Named("nonexistent".into()));
        assert!(group.is_err());
    }

    #[test]
    fn test_get_all_groups() {
        let config = create_test_config();
        let groups = get_all_groups(&config);

        assert!(groups.contains_key(&GroupId::Named("group1".into())));
        assert!(groups.contains_key(&GroupId::Named("group2".into())));
        assert!(groups.contains_key(&GroupId::Named("group3".into())));
        assert!(groups.contains_key(&GroupId::Ungrouped));

        assert!(
            groups
                .get(&GroupId::Named("group1".into()))
                .unwrap()
                .contains(&"alias1".to_string())
        );
        assert!(
            groups
                .get(&GroupId::Named("group1".into()))
                .unwrap()
                .contains(&"alias1_disabled".to_string())
        );

        assert!(
            groups
                .get(&GroupId::Named("group2".into()))
                .unwrap()
                .contains(&"alias2".to_string())
        );
        assert!(
            groups
                .get(&GroupId::Named("group3".into()))
                .unwrap()
                .contains(&"alias3".to_string())
        );
        assert!(
            groups
                .get(&GroupId::Ungrouped)
                .unwrap()
                .contains(&"alias4".to_string())
        );
    }

    #[test]
    fn test_get_all_groups_no_aliases() {
        let mut groups_map = HashMap::new();
        groups_map.insert("group1".into(), true);
        groups_map.insert("group2".into(), true);

        let config = Config {
            aliases: HashMap::new(),
            groups: groups_map,
        };

        let groups = get_all_groups(&config);
        assert_eq!(groups.len(), 3); // group1, group2, ungrouped
        assert_eq!(
            groups.get(&GroupId::Named("group1".into())).unwrap().len(),
            0
        );
        assert_eq!(
            groups.get(&GroupId::Named("group2".into())).unwrap().len(),
            0
        );
        assert_eq!(groups.get(&GroupId::Ungrouped).unwrap().len(), 0);
    }

    #[test]
    fn test_get_all_groups_empty() {
        let config = Config::new();

        let groups = get_all_groups(&config);
        assert_eq!(groups.len(), 1); // Only ungrouped should be present
        assert!(groups.contains_key(&GroupId::Ungrouped));
        assert_eq!(groups.get(&GroupId::Ungrouped).unwrap().len(), 0);
    }

    #[test]
    fn test_get_all_groups_no_groups() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "alias1".into(),
            Alias {
                command: "cmd1".into(),
                group: None,
                detailed: false,
                enabled: true,
            },
        );
        let config = Config {
            groups: HashMap::new(),
            aliases,
        };
        let groups = get_all_groups(&config);
        assert_eq!(groups.len(), 1); // Only ungrouped should be present
        assert!(
            groups
                .get(&GroupId::Ungrouped)
                .unwrap()
                .contains(&"alias1".to_string())
        );
    }

    #[test]
    fn test_get_disabled_aliases() {
        let config = create_test_config();
        let disabled = get_disabled_aliases_grouped(&config);
        assert!(disabled.get(&GroupId::Named("group1".into())).is_none());
        assert!(disabled.get(&GroupId::Named("group2".into())).is_none());
        assert!(
            disabled
                .get(&GroupId::Named("group3".into()))
                .unwrap()
                .contains(&"alias3".to_string())
        );
        assert!(
            disabled
                .get(&GroupId::Named("group3".into()))
                .unwrap()
                .contains(&"alias3_disabled".to_string())
        );

        assert!(
            disabled
                .get(&GroupId::Ungrouped)
                .unwrap()
                .contains(&"alias4_disabled".to_string())
        );
    }

    #[test]
    fn test_get_enabled_aliases() {
        let config = create_test_config();
        let enabled = get_enabled_aliases_grouped(&config);
        assert!(
            enabled
                .get(&GroupId::Named("group1".into()))
                .unwrap()
                .contains(&"alias1".to_string())
        );
        assert!(
            enabled
                .get(&GroupId::Named("group2".into()))
                .unwrap()
                .contains(&"alias2".to_string())
        );
        assert!(enabled.get(&GroupId::Named("group3".into())).is_none());
        assert!(
            enabled
                .get(&GroupId::Ungrouped)
                .unwrap()
                .contains(&"alias4".to_string())
        );
    }
}
