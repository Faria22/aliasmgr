use super::{Failure, Outcome};
use crate::config::types::Config;

use log::error;

pub fn sort_aliases_in_group(config: &mut Config, group: Option<&str>) -> Result<Outcome, Failure> {
    if let Some(group_name) = group
        && !config.groups.contains_key(group_name)
    {
        error!("Group '{}' does not exist.", group_name);
        return Err(Failure::GroupDoesNotExist);
    }

    config.aliases.sort_by(|key_a, val_a, key_b, val_b| {
        // Checks if both aliases belong to the specified group
        if val_a.group == val_b.group && val_a.group.as_deref() == group {
            key_a.cmp(key_b)
        } else {
            std::cmp::Ordering::Equal
        }
    });
    Ok(Outcome::ConfigChanged)
}

pub fn sort_groups(config: &mut Config) -> Result<Outcome, Failure> {
    config.groups.sort_keys();
    Ok(Outcome::ConfigChanged)
}

pub fn sort_all_aliases(config: &mut Config) -> Result<Outcome, Failure> {
    config.aliases.sort_keys();
    Ok(Outcome::ConfigChanged)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::app::shell::ShellType;
    use crate::config::types::Alias;
    use crate::core::list::get_aliases_from_single_group;

    #[test]
    fn test_sort_aliases_all_aliases() {
        let mut config = Config::new();
        config.groups.insert("group".to_string(), true);
        config.aliases.insert(
            "alias2".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        config.aliases.insert(
            "alias1".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        config.aliases.insert(
            "alias4".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        config.aliases.insert(
            "alias3".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        let result = sort_all_aliases(&mut config).unwrap();
        assert_eq!(result, Outcome::ConfigChanged);
        assert_eq!(
            get_aliases_from_single_group(&config, None, &ShellType::Bash).unwrap(),
            vec!["alias3".to_string(), "alias4".to_string()]
        );
        assert_eq!(
            get_aliases_from_single_group(&config, Some("group"), &ShellType::Bash).unwrap(),
            vec!["alias1".to_string(), "alias2".to_string()]
        );
    }

    #[test]
    fn test_sort_groups() {
        let mut config = Config::new();
        config.groups.insert("beta".to_string(), true);
        config.groups.insert("alpha".to_string(), true);
        let result = sort_groups(&mut config).unwrap();
        assert_eq!(result, Outcome::ConfigChanged);
        let keys: Vec<&String> = config.groups.keys().collect();
        assert_eq!(keys, vec![&"alpha".to_string(), &"beta".to_string()]);
    }

    #[test]
    fn test_sort_aliases_in_group() {
        let mut config = Config::new();
        config.groups.insert("group".to_string(), true);
        config.aliases.insert(
            "alias3".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        config.aliases.insert(
            "alias1".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        config.aliases.insert(
            "alias4".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        config.aliases.insert(
            "alias2".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );

        let result = sort_aliases_in_group(&mut config, Some("group")).unwrap();
        assert_eq!(result, Outcome::ConfigChanged);
        assert_eq!(
            get_aliases_from_single_group(&config, Some("group"), &ShellType::Bash).unwrap(),
            vec!["alias1".to_string(), "alias3".to_string()],
        );
        assert_eq!(
            get_aliases_from_single_group(&config, None, &ShellType::Bash).unwrap(),
            vec!["alias4".to_string(), "alias2".to_string()],
        );
    }

    #[test]
    fn test_sort_aliases_in_ungrouped() {
        let mut config = Config::new();
        config.groups.insert("group".to_string(), true);
        config.aliases.insert(
            "alias3".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        config.aliases.insert(
            "alias1".to_string(),
            Alias::new("cmd".into(), Some("group".to_string()), true, false),
        );
        config.aliases.insert(
            "alias4".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );
        config.aliases.insert(
            "alias2".to_string(),
            Alias::new("cmd".into(), None, true, false),
        );

        let result = sort_aliases_in_group(&mut config, None).unwrap();
        assert_eq!(result, Outcome::ConfigChanged);
        assert_eq!(
            get_aliases_from_single_group(&config, Some("group"), &ShellType::Bash).unwrap(),
            vec!["alias3".to_string(), "alias1".to_string()],
        );
        assert_eq!(
            get_aliases_from_single_group(&config, None, &ShellType::Bash).unwrap(),
            vec!["alias2".to_string(), "alias4".to_string()],
        );
    }

    #[test]
    fn test_sort_aliases_in_non_existent_group() {
        let mut config = Config::new();
        let result = sort_aliases_in_group(&mut config, Some("non_existent_group"));
        assert!(matches!(result, Err(Failure::GroupDoesNotExist)));
    }
}
