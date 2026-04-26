use super::{Failure, Outcome};
use crate::catalog::types::AliasCatalog;

use log::error;

pub fn sort_aliases_in_group(
    catalog: &mut AliasCatalog,
    group: Option<&str>,
) -> Result<Outcome, Failure> {
    if let Some(group_name) = group
        && !catalog.groups.contains_key(group_name)
    {
        error!("Group '{}' does not exist.", group_name);
        return Err(Failure::GroupDoesNotExist);
    }

    catalog.aliases.sort_by(|key_a, val_a, key_b, val_b| {
        // Checks if both aliases belong to the specified group
        if val_a.group == val_b.group && val_a.group.as_deref() == group {
            key_a.cmp(key_b)
        } else {
            std::cmp::Ordering::Equal
        }
    });
    Ok(Outcome::CatalogChanged)
}

pub fn sort_groups(catalog: &mut AliasCatalog) -> Result<Outcome, Failure> {
    catalog.groups.sort_keys();
    Ok(Outcome::CatalogChanged)
}

pub fn sort_all_aliases(catalog: &mut AliasCatalog) -> Result<Outcome, Failure> {
    catalog.aliases.sort_keys();
    Ok(Outcome::CatalogChanged)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::app::shell::ShellType;
    use crate::catalog::types::Alias;
    use crate::core::list::get_aliases_from_single_group;

    #[test]
    fn test_sort_aliases_all_aliases() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("group".to_string(), true);
        catalog.aliases.insert(
            "alias2".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        catalog.aliases.insert(
            "alias1".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        catalog.aliases.insert(
            "alias4".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        catalog.aliases.insert(
            "alias3".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        let result = sort_all_aliases(&mut catalog).unwrap();
        assert_eq!(result, Outcome::CatalogChanged);
        assert_eq!(
            get_aliases_from_single_group(&catalog, None, &ShellType::Bash).unwrap(),
            vec!["alias3".to_string(), "alias4".to_string()]
        );
        assert_eq!(
            get_aliases_from_single_group(&catalog, Some("group"), &ShellType::Bash).unwrap(),
            vec!["alias1".to_string(), "alias2".to_string()]
        );
    }

    #[test]
    fn test_sort_groups() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("beta".to_string(), true);
        catalog.groups.insert("alpha".to_string(), true);
        let result = sort_groups(&mut catalog).unwrap();
        assert_eq!(result, Outcome::CatalogChanged);
        let keys: Vec<&String> = catalog.groups.keys().collect();
        assert_eq!(keys, vec![&"alpha".to_string(), &"beta".to_string()]);
    }

    #[test]
    fn test_sort_aliases_in_group() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("group".to_string(), true);
        catalog.aliases.insert(
            "alias3".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        catalog.aliases.insert(
            "alias1".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        catalog.aliases.insert(
            "alias4".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        catalog.aliases.insert(
            "alias2".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );

        let result = sort_aliases_in_group(&mut catalog, Some("group")).unwrap();
        assert_eq!(result, Outcome::CatalogChanged);
        assert_eq!(
            get_aliases_from_single_group(&catalog, Some("group"), &ShellType::Bash).unwrap(),
            vec!["alias1".to_string(), "alias3".to_string()],
        );
        assert_eq!(
            get_aliases_from_single_group(&catalog, None, &ShellType::Bash).unwrap(),
            vec!["alias4".to_string(), "alias2".to_string()],
        );
    }

    #[test]
    fn test_sort_aliases_in_ungrouped() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("group".to_string(), true);
        catalog.aliases.insert(
            "alias3".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        catalog.aliases.insert(
            "alias1".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        catalog.aliases.insert(
            "alias4".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        catalog.aliases.insert(
            "alias2".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );

        let result = sort_aliases_in_group(&mut catalog, None).unwrap();
        assert_eq!(result, Outcome::CatalogChanged);
        assert_eq!(
            get_aliases_from_single_group(&catalog, Some("group"), &ShellType::Bash).unwrap(),
            vec!["alias3".to_string(), "alias1".to_string()],
        );
        assert_eq!(
            get_aliases_from_single_group(&catalog, None, &ShellType::Bash).unwrap(),
            vec!["alias2".to_string(), "alias4".to_string()],
        );
    }

    #[test]
    fn test_sort_aliases_in_non_existent_group() {
        let mut catalog = AliasCatalog::new();
        let result = sort_aliases_in_group(&mut catalog, Some("non_existent_group"));
        assert!(matches!(result, Err(Failure::GroupDoesNotExist)));
    }
}
