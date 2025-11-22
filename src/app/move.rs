use crate::core::{Failure, Outcome};

use crate::core::add::add_group;
use crate::core::r#move::move_alias;

use crate::config::types::Config;

use crate::cli::interaction::prompt_create_non_existent_group;
use crate::cli::r#move::MoveCommand;

use log::{debug, error, info};

pub fn handle_move(config: &mut Config, cmd: MoveCommand) -> Result<Outcome, Failure> {
    match move_alias(config, &cmd.name, &cmd.new_group) {
        Ok(outcome) => Ok(outcome),
        Err(e) => match e {
            Failure::GroupDoesNotExist => handle_non_existing_group(
                config,
                &cmd.name,
                &cmd.new_group.unwrap(),
                prompt_create_non_existent_group,
            ),
            Failure::AliasDoesNotExist => {
                error!("Alias '{}' does not exist", &cmd.name);
                Err(e)
            }
            _ => unreachable!(),
        },
    }
}

fn handle_non_existing_group(
    config: &mut Config,
    alias: &str,
    group: &str,
    create_group: impl Fn(&str) -> bool,
) -> Result<Outcome, Failure> {
    if create_group(&group) {
        info!("Created new group '{}'", group);
        add_group(config, &group, true)?;
        move_alias(config, &alias, &Some(group.to_string()))
    } else {
        debug!(
            "User aborted moving alias '{}' to non-existent group '{}'",
            &alias, &group
        );
        Ok(Outcome::NoChanges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Alias;
    use assert_matches::assert_matches;

    fn create_config_with_alias(alias_name: &str, command: &str) -> Config {
        let mut config = Config::new();
        config.aliases.insert(
            alias_name.into(),
            Alias::new(command.into(), true, None, false),
        );
        config
    }

    #[test]
    fn test_handle_non_existing_group_create() {
        let mut config = create_config_with_alias("ll", "ls -la");

        let outcome = handle_non_existing_group(&mut config, "ll", "new_group", |_| true).unwrap();

        assert_matches!(outcome, Outcome::ConfigChanged);
        assert!(config.groups.contains_key("new_group"));
        assert_eq!(
            config.aliases.get("ll").unwrap().group,
            Some("new_group".into())
        );
    }

    #[test]
    fn test_handle_non_existing_group_abort() {
        let mut config = create_config_with_alias("ll", "ls -la");
        let outcome = handle_non_existing_group(&mut config, "ll", "new_group", |_| false).unwrap();
        assert_matches!(outcome, Outcome::NoChanges);
        assert!(!config.groups.contains_key("new_group"));
        assert_eq!(config.aliases.get("ll").unwrap().group, None);
    }

    #[test]
    fn test_handle_move_alias_to_existing_group() {
        let mut config = create_config_with_alias("ll", "ls -la");
        config.groups.insert("utilities".into(), true);
        let cmd = MoveCommand {
            name: "ll".into(),
            new_group: Some("utilities".into()),
        };
        let outcome = handle_move(&mut config, cmd).unwrap();
        assert_matches!(outcome, Outcome::ConfigChanged);
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
    fn test_move_non_existent_alias() {
        let mut config = Config::new();
        let cmd = MoveCommand {
            name: "nonexistent".into(),
            new_group: Some("utilities".into()),
        };
        let result = handle_move(&mut config, cmd);
        assert_matches!(result, Err(Failure::AliasDoesNotExist));
        assert!(!config.aliases.contains_key("nonexistent"));
        assert!(!config.groups.contains_key("utilities"));
    }
}
