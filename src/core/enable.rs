use super::add::add_alias_str;
use super::list::get_aliases_from_single_group;
use super::{Failure, Outcome};

use crate::config::types::Config;

use crate::app::shell::ShellType;

use log::error;

pub fn enable_alias(config: &mut Config, name: &str) -> Result<Outcome, Failure> {
    if !config.aliases.contains_key(name) {
        error!("Alias {} does not exist.", name);
        return Err(Failure::AliasDoesNotExist);
    }

    let alias = config.aliases.get_mut(name).unwrap();
    alias.enabled = true;

    // Checks if the group the alias is in is disabled
    // If it is, then the alias will not be added to the shell
    if let Some(group) = &alias.group
        && !config.groups[group]
    {
        return Ok(Outcome::ConfigChanged);
    }

    Ok(Outcome::Command(add_alias_str(name, &alias)))
}

pub fn enable_group(
    config: &mut Config,
    name: &str,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    if !config.groups.contains_key(name) {
        error!("Group {} does not exist.", name);
        return Err(Failure::GroupDoesNotExist);
    }

    *config.groups.get_mut(name).unwrap() = true;
    let mut aliases_in_group = get_aliases_from_single_group(config, Some(name), shell)?;

    aliases_in_group.retain(|alias_name| config.aliases[alias_name].enabled);

    if aliases_in_group.is_empty() {
        return Ok(Outcome::ConfigChanged);
    }

    let mut command = String::new();
    for alias_name in aliases_in_group {
        let alias = &config.aliases[&alias_name];
        command.push_str(&add_alias_str(&alias_name, alias));
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
        config.groups.insert("empty_group".into(), false);

        config.aliases.insert(
            "alias1".into(),
            Alias::new("cmd".into(), Some("enabled_group".into()), false, false),
        );
        config.aliases.insert(
            "alias2".into(),
            Alias::new("cmd".into(), Some("disabled_group".into()), false, false),
        );

        config
    }

    #[test]
    fn enable_existing_alias() {
        let mut config = sample_config();
        let result = enable_alias(&mut config, "alias1");
        assert!(result.is_ok());
        assert!(config.aliases["alias1"].enabled);
        assert_matches!(result.unwrap(), Outcome::Command(_));
    }

    #[test]
    fn enable_nonexistent_alias() {
        let mut config = sample_config();
        let result = enable_alias(&mut config, "nonexisting");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Failure::AliasDoesNotExist);
    }

    #[test]
    fn enable_alias_in_disabled_group() {
        let mut config = sample_config();
        let result = enable_alias(&mut config, "alias2");
        assert!(result.is_ok());
        assert!(config.aliases["alias2"].enabled);
        assert_eq!(result.unwrap(), Outcome::ConfigChanged);
    }

    #[test]
    fn enable_nonexistent_group() {
        let mut config = sample_config();
        let result = enable_group(&mut config, "nonexisting", &ShellType::Bash);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Failure::GroupDoesNotExist);
    }

    #[test]
    fn enable_empty_group() {
        let mut config = sample_config();
        let result = enable_group(&mut config, "empty_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(config.groups["empty_group"]);
        assert_eq!(result.unwrap(), Outcome::ConfigChanged);
    }

    #[test]
    fn enable_group_with_disabled_aliases() {
        let mut config = sample_config();
        let result = enable_group(&mut config, "disabled_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(config.groups["disabled_group"]);
        assert_eq!(result.unwrap(), Outcome::ConfigChanged);
    }

    #[test]
    fn enable_group_with_enabled_aliases() {
        let mut config = sample_config();
        let _ = enable_alias(&mut config, "alias2");
        assert!(config.aliases["alias2"].enabled);

        let result = enable_group(&mut config, "disabled_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(config.groups["disabled_group"]);
        assert_matches!(result.unwrap(), Outcome::Command(_));
    }
}
