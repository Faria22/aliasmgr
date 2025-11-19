//! Functions for listing alias groups and their aliases.
//! This module provides functionality to retrieve all alias groups
//! and the aliases associated with them from a given configuration.
//! It supports both grouped and ungrouped aliases.
//!
//! # Functions
//! - `get_all_groups`: Returns a mapping of all groups to their aliases.
//! - `get_single_group`: Returns aliases for a specific group.

use crate::config::types::Config;
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

    // Populate the groups with alias names
    for (alias_name, alias) in &config.aliases {
        let key = alias
            .group
            .clone()
            .map(GroupId::Named)
            .unwrap_or(GroupId::Ungrouped);
        groups.entry(key).or_default().push(alias_name.clone());
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
pub fn get_single_group(config: &Config, identifier: GroupId) -> Vec<String> {
    if let GroupId::Named(name) = &identifier {
        if !config.groups.contains_key(name) {
            return Vec::new();
        }

        return config
            .aliases
            .iter()
            .filter_map(|(alias_name, alias)| {
                (alias.group.as_ref()? == name).then(|| alias_name.clone())
            })
            .collect();
    }

    return config
        .aliases
        .iter()
        .filter_map(|(alias_name, alias)| alias.group.is_none().then(|| alias_name.clone()))
        .collect();
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
            "alias2".into(),
            Alias {
                command: "cmd2".into(),
                group: Some("group2".into()),
                detailed: false,
                enabled: true,
            },
        );
        aliases.insert(
            "alias3".into(),
            Alias {
                command: "cmd3".into(),
                group: None,
                detailed: false,
                enabled: true,
            },
        );

        Config { groups, aliases }
    }

    #[test]
    fn test_get_single_group() {
        let config = create_test_config();

        let group = get_single_group(&config, GroupId::Named("group1".into()));
        assert_eq!(group.len(), 1);
        assert_eq!(group[0], "alias1".to_string());
        assert!(!group.contains(&"alias2".to_string()));
        assert!(!group.contains(&"alias3".to_string()));
    }

    #[test]
    fn test_get_ungrouped_aliases() {
        let config = create_test_config();

        let ungrouped = get_single_group(&config, GroupId::Ungrouped);
        assert_eq!(ungrouped.len(), 1);
        assert_eq!(ungrouped[0], "alias3".to_string());
        assert!(!ungrouped.contains(&"alias1".to_string()));
        assert!(!ungrouped.contains(&"alias2".to_string()));
    }

    #[test]
    fn test_get_single_group_empty() {
        let config = Config::new();
        let group = get_single_group(&config, GroupId::Named("nonexistent".into()));
        assert_eq!(group.len(), 0);
    }

    #[test]
    fn test_get_all_groups() {
        let config = create_test_config();
        let groups = get_all_groups(&config);

        assert!(groups.contains_key(&GroupId::Named("group1".into())));
        assert!(groups.contains_key(&GroupId::Named("group2".into())));
        assert!(groups.contains_key(&GroupId::Ungrouped));

        assert_eq!(
            groups.get(&GroupId::Named("group1".into())).unwrap(),
            &vec!["alias1".to_string()]
        );
        assert_eq!(
            groups.get(&GroupId::Named("group2".into())).unwrap(),
            &vec!["alias2".to_string()]
        );
        assert_eq!(
            groups.get(&GroupId::Ungrouped).unwrap(),
            &vec!["alias3".to_string()]
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
            groups.get(&GroupId::Named("group1".into())).unwrap(),
            &Vec::<String>::new()
        );
        assert_eq!(
            groups.get(&GroupId::Named("group2".into())).unwrap(),
            &Vec::<String>::new()
        );
        assert_eq!(
            groups.get(&GroupId::Ungrouped).unwrap(),
            &Vec::<String>::new()
        );
    }

    #[test]
    fn test_get_all_groups_empty() {
        let config = Config::new();

        let groups = get_all_groups(&config);
        assert_eq!(groups.len(), 1); // Only ungrouped should be present
        assert!(groups.contains_key(&GroupId::Ungrouped));
        assert_eq!(
            groups.get(&GroupId::Ungrouped).unwrap(),
            &Vec::<String>::new()
        );
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
        assert_eq!(
            groups.get(&GroupId::Ungrouped).unwrap(),
            &vec!["alias1".to_string()]
        );
    }
}
