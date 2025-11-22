use super::{Failure, Outcome};
use crate::config::types::Config;

pub fn move_alias(
    config: &mut Config,
    alias: &str,
    new_group: &Option<String>,
) -> Result<Outcome, Failure> {
    // Checks if alias exists before moving forward
    if !config.aliases.contains_key(alias) {
        return Err(Failure::AliasDoesNotExist);
    }

    // If moving to a specific group, check if the group exists first
    if let Some(group) = new_group
        && !config.groups.contains_key(group)
    {
        return Err(Failure::GroupDoesNotExist);
    }

    config.aliases.get_mut(alias).unwrap().group = new_group.clone();
    Ok(Outcome::ConfigChanged)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::types::Alias;

    #[test]
    fn move_alias_to_existing_group() {
        let mut config = Config::new();
        config
            .aliases
            .insert("ll".into(), Alias::new("ls -la".into(), true, None, false));
        config.groups.insert("utilities".into(), true);
        let result = move_alias(&mut config, "ll", &Some("utilities".into()));
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias::new(
                "ls -la".into(),
                true,
                Some("utilities".into()),
                false
            ))
        );
    }

    #[test]
    fn move_alias_to_non_existent_group() {
        let mut config = Config::new();
        config
            .aliases
            .insert("ll".into(), Alias::new("ls -la".into(), true, None, false));
        let result = move_alias(&mut config, "ll", &Some("nonexistent".into()));
        assert!(matches!(result, Err(Failure::GroupDoesNotExist)));
    }

    #[test]
    fn move_non_existent_alias() {
        let mut config = Config::new();
        let result = move_alias(&mut config, "nonexistent", &Some("utilities".into()));
        assert!(matches!(result, Err(Failure::AliasDoesNotExist)));
    }
}
