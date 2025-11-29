use super::list::get_aliases_from_single_group;
use super::{Failure, Outcome};

use crate::config::types::Config;

use crate::app::shell::ShellType;

use log::error;

pub fn disable_alias(config: &mut Config, name: &str) -> Result<Outcome, Failure> {
    if !config.aliases.contains_key(name) {
        error!("Alias {} does not exist.", name);
        return Err(Failure::AliasDoesNotExist);
    }

    let alias = config.aliases.get_mut(name).unwrap();

    if !alias.enabled {
        return Ok(Outcome::NoChanges);
    }

    alias.enabled = false;

    // Checks if the group the alias is in is disabled
    // If it is, then the alias will not be removed from the shell
    if let Some(group) = &alias.group
        && !config.groups[group]
    {
        return Ok(Outcome::ConfigChanged);
    }

    Ok(Outcome::Command(format!("unalias '{}'", name)))
}

pub fn disable_group(
    config: &mut Config,
    name: &str,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    if !config.groups.contains_key(name) {
        error!("Group {} does not exist.", name);
        return Err(Failure::GroupDoesNotExist);
    }

    // If the group is already disabled, do nothing
    if !config.groups[name] {
        return Ok(Outcome::NoChanges);
    }

    *config.groups.get_mut(name).unwrap() = false;

    // Get all aliases in the group that are enabled and remove them from the shell
    let mut aliases_in_group = get_aliases_from_single_group(config, Some(name), shell)?;
    aliases_in_group.retain(|alias_name| config.aliases[alias_name].enabled);

    if aliases_in_group.is_empty() {
        return Ok(Outcome::ConfigChanged);
    }

    let mut command = String::new();
    for alias_name in aliases_in_group {
        command.push_str(&format!("unalias '{}'\n", alias_name));
        command.push('\n');
    }

    Ok(Outcome::Command(command))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use crate::config::types::Alias;
    use assert_matches::assert_matches;

    fn sample_config() -> Config {
        let mut config = Config::new();
        config.groups.insert("enabled_group".into(), true);
        config.groups.insert("disabled_group".into(), false);
        config.groups.insert("empty_group".into(), true);

        config.aliases.insert(
            "alias1".into(),
            Alias::new("cmd".into(), Some("enabled_group".into()), true, false),
        );
        config.aliases.insert(
            "alias2".into(),
            Alias::new("cmd".into(), Some("disabled_group".into()), true, false),
        );

        config
    }

    #[test]
    fn disable_existing_alias() {
        let mut config = sample_config();
        let result = disable_alias(&mut config, "alias1");
        assert!(result.is_ok());
        assert!(!config.aliases["alias1"].enabled);
        assert_matches!(result.unwrap(), Outcome::Command(_));
    }

    #[test]
    fn disable_disabled_alias() {
        let mut config = sample_config();
        let _ = disable_alias(&mut config, "alias1");
        assert!(!config.aliases["alias1"].enabled);

        let result = disable_alias(&mut config, "alias1");
        assert!(result.is_ok());
        assert!(!config.aliases["alias1"].enabled);
        assert_matches!(result.unwrap(), Outcome::NoChanges);
    }

    #[test]
    fn disable_nonexistent_alias() {
        let mut config = sample_config();
        let result = disable_alias(&mut config, "nonexisting");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Failure::AliasDoesNotExist);
    }

    #[test]
    fn disable_alias_in_disabled_group() {
        let mut config = sample_config();
        let result = disable_alias(&mut config, "alias2");
        assert!(result.is_ok());
        assert!(!config.aliases["alias2"].enabled);
        assert_eq!(result.unwrap(), Outcome::ConfigChanged);
    }

    #[test]
    fn disable_disabled_group() {
        let mut config = sample_config();
        let result = disable_group(&mut config, "disabled_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(!config.groups["disabled_group"]);
        assert_eq!(result.unwrap(), Outcome::NoChanges);
    }

    #[test]
    fn disable_nonexistent_group() {
        let mut config = sample_config();
        let result = disable_group(&mut config, "nonexisting", &ShellType::Bash);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Failure::GroupDoesNotExist);
    }

    #[test]
    fn disable_empty_group() {
        let mut config = sample_config();
        let result = disable_group(&mut config, "empty_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(!config.groups["empty_group"]);
        assert_eq!(result.unwrap(), Outcome::ConfigChanged);
    }

    #[test]
    fn disable_group_with_enabled_aliases() {
        let mut config = sample_config();
        let result = disable_group(&mut config, "enabled_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(!config.groups["enabled_group"]);
        assert_matches!(result.unwrap(), Outcome::Command(_));
    }

    #[test]
    fn disable_group_with_disabled_aliases() {
        let mut config = sample_config();
        let _ = disable_alias(&mut config, "alias1");
        assert!(!config.aliases["alias1"].enabled);

        let result = disable_group(&mut config, "enabled_group", &ShellType::Bash);
        assert!(!config.groups["enabled_group"]);
        assert_eq!(result.unwrap(), Outcome::ConfigChanged);
    }
}
