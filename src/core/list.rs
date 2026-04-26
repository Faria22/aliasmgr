//! Functions for listing alias groups and their aliases.
//! This module provides functionality to retrieve all alias groups
//! and the aliases associated with them from a given catalog.
//! It supports both grouped and ungrouped aliases.
//!
//! # Functions
//! - `get_all_groups`: Returns a mapping of all groups to their aliases.
//! - `get_single_group`: Returns aliases for a specific group.

use log::info;

use crate::app::shell::ShellType;

use crate::catalog::types::AliasCatalog;
use crate::core::Failure;
use indexmap::IndexMap;
use std::vec::Vec;

/// Retrieves all alias groups and their associated aliases from the catalog.
/// # Arguments
/// * `catalog` - A reference to the catalog containing groups and aliases.
///
/// # Returns
/// A HashMap where keys are GroupId (either named or ungrouped) and values
/// are vectors of alias names belonging to those groups.
pub fn get_all_aliases_grouped(
    catalog: &AliasCatalog,
    shell: &ShellType,
) -> IndexMap<Option<String>, Vec<String>> {
    let mut groups = IndexMap::<Option<String>, Vec<String>>::new();

    // Initialize the groups with empty vectors
    groups.insert(None, Vec::new());
    for group_name in catalog.groups.keys() {
        groups.insert(Some(group_name.clone()), Vec::new());
    }

    // Populate the groups with alias names
    for (alias_name, alias) in &catalog.aliases {
        if alias.global && *shell != ShellType::Zsh {
            continue;
        }
        groups
            .get_mut(&alias.group)
            .expect("group is in aliases, but not in the group vector")
            .push(alias_name.clone());
    }

    groups
}

/// Retrieves aliases for a specific group from the catalog.
/// # Arguments
/// * `catalog` - A reference to the catalog containing groups and aliases.
/// * `name` - `GroupId` specifying the group to retrieve aliases for.
///
/// # Returns
/// A vector of alias names belonging to the specified group.
pub fn get_aliases_from_single_group(
    catalog: &AliasCatalog,
    group: Option<&str>,
    shell: &ShellType,
) -> Result<Vec<String>, Failure> {
    if let Some(name) = group
        && !catalog.groups.contains_key(name)
    {
        info!("Group '{}' does not exist.", name);
        return Err(Failure::GroupDoesNotExist);
    }

    info!("Retrieving aliases.");
    Ok(catalog
        .aliases
        .iter()
        .filter(|(_, alias)| alias.group.as_deref() == group)
        .filter(|(_, alias)| !(alias.global && *shell != ShellType::Zsh))
        .map(|(alias_name, _)| alias_name.clone())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    fn create_test_catalog() -> AliasCatalog {
        let mut groups = IndexMap::new();
        let mut aliases = IndexMap::new();

        groups.insert("group1".into(), true);
        groups.insert("group2".into(), true);
        groups.insert("group3".into(), false);

        // Named group with enabled and disabled aliases, and a global alias
        aliases.insert(
            "alias1".into(),
            Alias::new("cmd1".into(), Some("group1".into()), true, false),
        );
        aliases.insert(
            "alias1_disabled".into(),
            Alias::new("cmd1".into(), Some("group1".into()), false, false),
        );
        aliases.insert(
            "global_alias".into(),
            Alias::new("global_cmd".into(), None, true, true),
        );

        // Another named group with only enabled aliases
        aliases.insert(
            "alias2".into(),
            Alias::new("cmd2".into(), Some("group2".into()), true, false),
        );
        //
        // Named group that is disabled with enabled and disabled aliases
        aliases.insert(
            "alias3".into(),
            Alias::new("cmd3".into(), Some("group3".into()), true, false),
        );
        aliases.insert(
            "alias3_disabled".into(),
            Alias::new("cmd3".into(), Some("group3".into()), true, false),
        );

        // Ungrouped aliases
        aliases.insert(
            "alias4".into(),
            Alias::new("cmd4".into(), None, true, false),
        );
        aliases.insert(
            "alias4_disabled".into(),
            Alias::new("cmd4".into(), None, false, false),
        );

        AliasCatalog { groups, aliases }
    }

    #[test]
    fn test_get_single_group() {
        let catalog = create_test_catalog();

        let group = get_aliases_from_single_group(&catalog, Some("group1"), &ShellType::Bash);

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
        let catalog = create_test_catalog();

        let ungrouped = get_aliases_from_single_group(&catalog, None, &ShellType::Bash);

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
        let catalog = AliasCatalog::new();
        let group = get_aliases_from_single_group(&catalog, Some("nonexistent"), &ShellType::Bash);
        assert!(group.is_err());
    }

    #[test]
    fn test_get_all_groups() {
        let catalog = create_test_catalog();
        let groups = get_all_aliases_grouped(&catalog, &ShellType::Bash);

        assert!(groups.contains_key(&Some("group1".into())));
        assert!(groups.contains_key(&Some("group2".into())));
        assert!(groups.contains_key(&Some("group3".into())));
        assert!(groups.contains_key(&None));

        assert!(
            groups
                .get(&Some("group1".into()))
                .unwrap()
                .contains(&"alias1".to_string())
        );
        assert!(
            groups
                .get(&Some("group1".into()))
                .unwrap()
                .contains(&"alias1_disabled".to_string())
        );

        assert!(
            groups
                .get(&Some("group2".into()))
                .unwrap()
                .contains(&"alias2".to_string())
        );
        assert!(
            groups
                .get(&Some("group3".into()))
                .unwrap()
                .contains(&"alias3".to_string())
        );
        assert!(groups.get(&None).unwrap().contains(&"alias4".to_string()));
    }

    #[test]
    fn test_get_all_groups_no_aliases() {
        let mut groups_map = IndexMap::new();
        groups_map.insert("group1".into(), true);
        groups_map.insert("group2".into(), true);

        let catalog = AliasCatalog {
            aliases: IndexMap::new(),
            groups: groups_map,
        };

        let groups = get_all_aliases_grouped(&catalog, &ShellType::Bash);
        assert_eq!(groups.len(), 3); // group1, group2, ungrouped
        assert_eq!(groups.get(&Some("group1".into())).unwrap().len(), 0);
        assert_eq!(groups.get(&Some("group2".into())).unwrap().len(), 0);
        assert_eq!(groups.get(&None).unwrap().len(), 0);
    }

    #[test]
    fn test_get_all_groups_empty() {
        let catalog = AliasCatalog::new();

        let groups = get_all_aliases_grouped(&catalog, &ShellType::Bash);
        assert_eq!(groups.len(), 1); // Only ungrouped should be present
        assert!(groups.contains_key(&None));
        assert_eq!(groups.get(&None).unwrap().len(), 0);
    }

    #[test]
    fn test_get_all_groups_no_groups() {
        let mut aliases = IndexMap::new();
        aliases.insert(
            "alias1".into(),
            Alias::new("cmd1".into(), None, true, false),
        );
        let catalog = AliasCatalog {
            groups: IndexMap::new(),
            aliases,
        };
        let groups = get_all_aliases_grouped(&catalog, &ShellType::Bash);
        assert_eq!(groups.len(), 1); // Only ungrouped should be present
        assert!(groups.get(&None).unwrap().contains(&"alias1".to_string()));
    }

    #[test]
    fn test_get_single_group_no_aliases() {
        let mut groups_map = IndexMap::new();
        groups_map.insert("group1".into(), true);

        let catalog = AliasCatalog {
            aliases: IndexMap::new(),
            groups: groups_map,
        };

        let group = get_aliases_from_single_group(&catalog, Some("group1"), &ShellType::Bash);
        assert!(group.is_ok());
        let group = group.unwrap();
        assert_eq!(group.len(), 0);
    }

    #[test]
    fn test_get_single_group_bash_skips_global() {
        let catalog = create_test_catalog();
        let ungrouped = get_aliases_from_single_group(&catalog, None, &ShellType::Bash);
        assert!(ungrouped.is_ok());
        let ungrouped = ungrouped.unwrap();
        assert!(!ungrouped.contains(&"global_alias".to_string()));
    }

    #[test]
    fn test_get_all_groups_bash_skips_global() {
        let catalog = create_test_catalog();
        let groups = get_all_aliases_grouped(&catalog, &ShellType::Bash);
        let ungrouped = groups.get(&None).unwrap();
        assert!(!ungrouped.contains(&"global_alias".to_string()));
    }

    #[test]
    fn test_get_single_group_zsh_includes_global() {
        let catalog = create_test_catalog();
        let ungrouped = get_aliases_from_single_group(&catalog, None, &ShellType::Zsh);
        assert!(ungrouped.is_ok());
        let ungrouped = ungrouped.unwrap();
        assert!(ungrouped.contains(&"global_alias".to_string()));
    }

    #[test]
    fn test_get_all_group_zsh_includes_global() {
        let catalog = create_test_catalog();
        let groups = get_all_aliases_grouped(&catalog, &ShellType::Zsh);
        let ungrouped = groups.get(&None).unwrap();
        assert!(ungrouped.contains(&"global_alias".to_string()));
    }
}
