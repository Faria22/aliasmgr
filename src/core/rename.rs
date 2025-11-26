use super::add::add_alias;
use super::remove::remove_alias;
use super::{Failure, Outcome};
use crate::config::types::Config;

use log::error;

pub fn rename_alias(
    config: &mut Config,
    old_alias: &str,
    new_alias: &str,
) -> Result<Outcome, Failure> {
    if !config.aliases.contains_key(old_alias) {
        error!("Alias {} does not exists.", old_alias);
        return Err(Failure::AliasDoesNotExist);
    }

    if config.aliases.contains_key(new_alias) {
        error!("Alias {} already exists.", new_alias);
        return Err(Failure::AliasAlreadyExists);
    }

    let mut command = String::new();
    let alias = config.aliases[old_alias].clone();

    if let Outcome::Command(cmd) = remove_alias(config, old_alias)? {
        command.push_str(&cmd);
        command.push('\n');
    } else {
        // This should never happen
        error!("Unexpected behavior when removing alias {}", old_alias);
        return Err(Failure::UnexpectedBehavior);
    }

    if let Outcome::Command(cmd) = add_alias(config, new_alias, &alias)? {
        command.push_str(&cmd);
    } else {
        // This should never happen
        error!("Unexpected behavior when adding alias {}", new_alias);
        return Err(Failure::UnexpectedBehavior);
    }

    Ok(Outcome::Command(command))
}

pub fn rename_group(
    config: &mut Config,
    old_group: &str,
    new_group: &str,
) -> Result<Outcome, Failure> {
    if !config.groups.contains_key(old_group) {
        error!("Group {} does not exists.", old_group);
        return Err(Failure::GroupDoesNotExist);
    }

    if config.groups.contains_key(new_group) {
        error!("Group {} already exists.", new_group);
        return Err(Failure::GroupAlreadyExists);
    }

    let enabled = config
        .groups
        .shift_remove(old_group)
        .expect("the group has been checked to exist already");

    config.groups.insert(new_group.into(), enabled);

    for alias in config.aliases.values_mut() {
        if alias.group == Some(old_group.into()) {
            alias.group = Some(new_group.into());
        }
    }

    Ok(Outcome::ConfigChanged)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use crate::config::types::Alias;
    use assert_matches::assert_matches;

    fn create_config() -> Config {
        let mut config = Config::new();
        config.groups.insert("group".into(), true);
        config.groups.insert("other_group".into(), true);
        config.aliases.insert(
            "foo".into(),
            Alias::new("bar".into(), Some("group".into()), true, false),
        );
        config
            .aliases
            .insert("ll".into(), Alias::new("ls -la".into(), None, true, false));

        config
    }

    #[test]
    fn test_rename_alias_success() {
        let mut config = create_config();
        let result = rename_alias(&mut config, "foo", "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rename_alias_nonexistent() {
        let mut config = create_config();
        let result = rename_alias(&mut config, "nonexistent", "boo");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::AliasDoesNotExist);
    }

    #[test]
    fn test_rename_alias_to_existent() {
        let mut config = create_config();
        let result = rename_alias(&mut config, "foo", "ll");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::AliasAlreadyExists);
    }

    #[test]
    fn test_rename_group_success() {
        let mut config = create_config();
        let result = rename_group(&mut config, "group", "nonexistent");
        assert!(result.is_ok());
        assert_matches!(result.unwrap(), Outcome::ConfigChanged);
        assert_eq!(config.aliases["foo"].group, Some("nonexistent".into()));
    }

    #[test]
    fn test_rename_group_nonexistent() {
        let mut config = create_config();
        let result = rename_group(&mut config, "nonexistent", "boo");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::GroupDoesNotExist);
    }

    #[test]
    fn test_rename_group_to_existent() {
        let mut config = create_config();
        let result = rename_group(&mut config, "group", "other_group");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::GroupAlreadyExists);
    }
}
