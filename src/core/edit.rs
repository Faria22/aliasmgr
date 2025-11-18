use crate::config::types::Config;
use log::info;

enum EditError {
    AliasDoesNotExist,
}

pub fn edit_alias(config: &mut Config, name: &str, new_command: &str) {
    if let Err(e) = edit_alias_in_config(config, name, new_command) {
        match e {
            EditError::AliasDoesNotExist => {
                eprintln!("Error: Alias '{}' does not exist.", name);
            }
        }
    }

    info!("Alias '{}' edited successfully.", name);
}

fn edit_alias_in_config(
    config: &mut Config,
    name: &str,
    new_command: &str,
) -> Result<(), EditError> {
    if !config.aliases.contains_key(name) {
        info!("Alias '{}' does not exist.", name);
        return Err(EditError::AliasDoesNotExist);
    }

    config.aliases.get_mut(name).unwrap().command = new_command.into();

    info!("Alias '{}' command updated to '{}'.", name, new_command);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Alias, Config};

    #[test]
    fn test_edit_alias_success() {
        let mut config = Config::new();
        config.aliases.insert(
            "test".into(),
            Alias {
                command: "old_command".into(),
                enabled: true,
                detailed: false,
                group: None,
            },
        );

        let result = edit_alias_in_config(&mut config, "test", "new_command");

        assert!(result.is_ok());
        assert_eq!(config.aliases.get("test").unwrap().command, "new_command");
    }

    #[test]
    fn test_edit_alias_nonexistent() {
        let mut config = Config::new();
        let result = edit_alias_in_config(&mut config, "nonexistent", "new_command");
        assert!(matches!(result, Err(EditError::AliasDoesNotExist)));
    }
}
