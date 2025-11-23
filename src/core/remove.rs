use super::{Failure, Outcome};
use crate::config::types::Config;
use log::error;

pub fn remove_alias(config: &mut Config, name: &str) -> Result<Outcome, Failure> {
    match config.aliases.shift_remove(name) {
        Some(_) => Ok(Outcome::Command(format!("unalias '{}'", name))),
        None => {
            error!("Alias '{}' does not exist", name);
            Err(Failure::AliasDoesNotExist)
        }
    }
}

pub fn remove_all_aliases(config: &mut Config) -> Result<Outcome, Failure> {
    config.aliases.clear();
    Ok(Outcome::Command("unalias -a".to_string()))
}

pub fn remove_all_groups(config: &mut Config) -> Result<Outcome, Failure> {
    config.groups.clear();
    Ok(Outcome::ConfigChanged)
}

pub fn remove_all(config: &mut Config) -> Result<Outcome, Failure> {
    remove_all_groups(config)?;
    remove_all_aliases(config)
}

pub fn remove_aliases(config: &mut Config, names: &[String]) -> Result<Outcome, Failure> {
    let mut command_outcome = String::new();
    for name in names {
        let result = remove_alias(config, name)?;
        // Collect remove command outcomes
        if let Outcome::Command(cmd) = result {
            command_outcome.push_str(&format!("{}\n", cmd));
        }
    }
    Ok(Outcome::Command(command_outcome.trim().to_string()))
}

pub fn remove_group(config: &mut Config, name: &str) -> Result<Outcome, Failure> {
    match config.groups.shift_remove(name) {
        Some(_) => Ok(Outcome::ConfigChanged),
        None => {
            error!("Group '{}' does not exist", name);
            Err(Failure::GroupDoesNotExist)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Alias;

    fn sample_config() -> Config {
        let mut config = Config::new();
        config.aliases.insert(
            "foo".to_string(),
            Alias::new("bar".to_string(), None, true, false),
        );
        config.aliases.insert(
            "baz".to_string(),
            Alias::new("qux".to_string(), None, true, false),
        );
        config.groups.insert("dev".to_string(), true);

        config
    }

    #[test]
    fn test_remove_alias_success() {
        let mut config = sample_config();
        let result = remove_alias(&mut config, "foo");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Outcome::Command("unalias 'foo'".to_string())
        );
        assert!(config.aliases.contains_key("baz"));
        assert!(!config.aliases.contains_key("foo"));
    }

    #[test]
    fn test_remove_alias_failure() {
        let mut config = sample_config();
        let result = remove_alias(&mut config, "nonexistent");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Failure::AliasDoesNotExist));
    }

    #[test]
    fn test_remove_group_success() {
        let mut config = sample_config();
        let result = remove_group(&mut config, "dev");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Outcome::ConfigChanged);
        assert!(!config.groups.contains_key("dev"));
    }

    #[test]
    fn test_remove_group_failure() {
        let mut config = sample_config();
        let result = remove_group(&mut config, "nonexistent");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_remove_all() {
        let mut config = sample_config();
        let result = remove_all(&mut config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Outcome::Command("unalias -a".to_string()));
        assert!(config.aliases.is_empty());
        assert!(config.groups.is_empty());
    }

    #[test]
    fn test_remove_aliases() {
        let mut config = sample_config();
        let names = vec!["foo".to_string(), "baz".to_string()];
        let result = remove_aliases(&mut config, &names);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Outcome::Command("unalias 'foo'\nunalias 'baz'".to_string())
        );
        assert!(config.aliases.is_empty());
        assert!(config.groups.contains_key("dev"));
    }
}
