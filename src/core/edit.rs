//! Module for editing aliases in the catalog.
//! Provides functionality to edit existing aliases.
//! Handles errors when trying to edit non-existent aliases.
//!
//! # Functions
//! - `edit_alias`: Edits an alias in the catalog.

use super::add::add_alias_str;
use super::{Failure, Outcome};
use crate::catalog::types::{Alias, AliasCatalog};
use log::info;

/// Edits an alias in the given catalog.
///
/// # Arguments
/// - `catalog`: Mutable reference to the catalog.
/// - `name`: Name of the alias to edit.
/// - `new_command`: New command for the alias.
///
/// # Returns
/// - `Ok(())` if the alias was edited successfully.
/// - `Err(EditError)` if an error occurred.
pub fn edit_alias(
    catalog: &mut AliasCatalog,
    name: &str,
    new_alias: &Alias,
) -> Result<Outcome, Failure> {
    match catalog.aliases.get_mut(name) {
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
    use crate::catalog::types::{Alias, AliasCatalog};
    use assert_matches::assert_matches;

    fn test_alias() -> Alias {
        Alias::new("test_command".into(), None, true, false)
    }

    #[test]
    fn test_edit_alias_success() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "test".into(),
            Alias::new("old_command".into(), None, true, false),
        );

        let new_alias = test_alias();

        let result = edit_alias(&mut catalog, "test", &new_alias);

        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get("test").unwrap(), &new_alias);
    }

    #[test]
    fn test_edit_alias_nonexistent() {
        let mut catalog = AliasCatalog::new();
        let new_alias = test_alias();
        let result = edit_alias(&mut catalog, "nonexistent", &new_alias);
        assert_matches!(result, Err(Failure::AliasDoesNotExist));
    }
}
