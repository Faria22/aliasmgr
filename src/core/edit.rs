//! Module for editing aliases in the configuration.
//! Provides functionality to edit existing aliases.
//! Handles errors when trying to edit non-existent aliases.
//!
//! # Functions
//! - `edit_alias`: Edits an alias in the configuration.

use super::add::add_alias_str;
use super::{Failure, Outcome};
use crate::config::types::{Alias, Config};
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
pub fn edit_alias(config: &mut Config, name: &str, new_alias: &Alias) -> Result<Outcome, Failure> {
    match config.aliases.get_mut(name) {
        Some(alias) => {
            info!("Editing alias '{}'.", name);
            *alias = new_alias.clone();
            Ok(Outcome::Command(add_alias_str(name, new_alias).to_string()))
        }
        None => {
            info!("Alias '{}' does not exist.", name);
            Err(Failure::AliasDoesNotExist)
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::types::{Alias, Config};
    use assert_matches::assert_matches;

    fn test_alias() -> Alias {
        Alias::new("test_command".into(), None, true, false)
    }

    #[test]
    fn test_edit_alias_success() {
        let mut config = Config::new();
        config.aliases.insert(
            "test".into(),
            Alias::new("old_command".into(), None, true, false),
        );

        let new_alias = test_alias();

        let result = edit_alias(&mut config, "test", &new_alias);

        assert!(result.is_ok());
        assert_eq!(config.aliases.get("test").unwrap(), &new_alias);
    }

    #[test]
    fn test_edit_alias_nonexistent() {
        let mut config = Config::new();
        let new_alias = test_alias();
        let result = edit_alias(&mut config, "nonexistent", &new_alias);
        assert_matches!(result, Err(Failure::AliasDoesNotExist));
    }
}
