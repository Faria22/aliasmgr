//! Module for editing aliases in the configuration.
//! Provides functionality to edit existing aliases.
//! Handles errors when trying to edit non-existent aliases.
//!
//! # Functions
//! - `edit_alias`: Edits an alias in the configuration.

use crate::config::types::Config;
use crate::core::{Failure, Outcome};
use log::info;

/// Edits an alias in the given configuration.
///
/// # Arguments
/// - `config`: Mutable reference to the configuration.
/// - `name`: Name of the alias to edit.
/// - `new_command`: New command for the alias.
///
/// # Returns
/// - `Ok(())` if the alias was edited successfully.
/// - `Err(EditError)` if an error occurred.
pub fn edit_alias(config: &mut Config, name: &str, new_command: &str) -> Result<Outcome, Failure> {
    if !config.aliases.contains_key(name) {
        info!("Alias '{}' does not exist.", name);
        return Err(Failure::AliasDoesNotExist);
    }

    config.aliases.get_mut(name).unwrap().command = new_command.into();

    info!("Alias '{}' command updated to '{}'.", name, new_command);
    Ok(Outcome::Command(format!(
        "alias {}='{}'",
        name, new_command
    )))
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
            Alias::new("old_command".into(), true, None, false),
        );

        let result = edit_alias(&mut config, "test", "new_command");

        assert!(result.is_ok());
        assert_eq!(config.aliases.get("test").unwrap().command, "new_command");
    }

    #[test]
    fn test_edit_alias_nonexistent() {
        let mut config = Config::new();
        let result = edit_alias(&mut config, "nonexistent", "new_command");
        assert!(matches!(result, Err(Failure::AliasDoesNotExist)));
    }
}
