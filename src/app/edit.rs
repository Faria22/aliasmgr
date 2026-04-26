use crate::catalog::types::{Alias, AliasCatalog};
use crate::cli::edit::EditCommand;
use crate::cli::interaction::prompt_create_non_existent_group;
use crate::core::edit::edit_alias;
use crate::core::{Failure, Outcome};

fn handle_nonexistent_group(
    catalog: &mut AliasCatalog,
    group_name: &str,
    create_group: impl Fn(&str) -> bool,
) -> Result<(), Failure> {
    if create_group(group_name) {
        catalog.groups.insert(group_name.to_string(), true);
        Ok(())
    } else {
        Err(Failure::GroupDoesNotExist)
    }
}

pub fn handle_edit(catalog: &mut AliasCatalog, cmd: EditCommand) -> Result<Outcome, Failure> {
    let mut new_alias = Alias::new("".into(), None, true, false); // Default initialization

    if let Some(old_alias) = catalog.aliases.get(&cmd.name) {
        new_alias = old_alias.clone();
        new_alias.command = cmd.new_command;

        if cmd.toggle_enable {
            new_alias.enabled = !old_alias.enabled;
        }

        if cmd.toggle_global {
            new_alias.global = !old_alias.global;
        }

        if let Some(group) = cmd.group {
            // Checks if named group exists before moving it
            if let Some(group_name) = &group
                && !catalog.groups.contains_key(group_name)
            {
                handle_nonexistent_group(catalog, group_name, prompt_create_non_existent_group)?;
            }
            new_alias.group = group;
        }
    };

    // If no old alias found, edit_alias will return the appropriate error, and we can just use the
    // default new_alias as a placeholder.
    edit_alias(catalog, &cmd.name, &new_alias)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    fn create_test_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "test".into(),
            Alias::new("original_command".into(), None, true, false),
        );
        catalog
    }

    #[test]
    fn test_handle_edit_success() {
        let mut catalog = create_test_catalog();
        let cmd = EditCommand {
            name: "test".into(),
            new_command: "edited_command".into(),
            toggle_enable: false,
            toggle_global: false,
            group: None,
        };
        let result = handle_edit(&mut catalog, cmd);
        assert!(result.is_ok());
        let edited_alias = catalog.aliases.get("test").unwrap();
        assert_eq!(edited_alias.command, "edited_command");
    }

    #[test]
    fn test_handle_edit_nonexistent() {
        let mut catalog = AliasCatalog::new();
        let cmd = EditCommand {
            name: "nonexistent".into(),
            new_command: "edited_command".into(),
            toggle_enable: false,
            toggle_global: false,
            group: None,
        };
        let result = handle_edit(&mut catalog, cmd);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Failure::AliasDoesNotExist));
    }

    #[test]
    fn test_handle_edit_toggle_enable() {
        let mut catalog = create_test_catalog();
        let cmd = EditCommand {
            name: "test".into(),
            new_command: "edit_command".into(),
            toggle_enable: true,
            toggle_global: false,
            group: None,
        };
        let result = handle_edit(&mut catalog, cmd);
        assert!(result.is_ok());
        let edited_alias = catalog.aliases.get("test").unwrap();
        assert_eq!(edited_alias.command, "edit_command");
        assert!(!edited_alias.enabled);
    }

    #[test]
    fn test_handle_edit_toggle_global() {
        let mut catalog = create_test_catalog();
        let cmd = EditCommand {
            name: "test".into(),
            new_command: "edit_command".into(),
            toggle_enable: false,
            toggle_global: true,
            group: None,
        };
        let result = handle_edit(&mut catalog, cmd);
        assert!(result.is_ok());
        let edited_alias = catalog.aliases.get("test").unwrap();
        assert_eq!(edited_alias.command, "edit_command");
        assert!(edited_alias.global);
    }

    #[test]
    fn test_handle_edit_set_existing_group() {
        let mut catalog = create_test_catalog();
        catalog.groups.insert("dev".into(), true);
        let cmd = EditCommand {
            name: "test".into(),
            new_command: "edit_command".into(),
            toggle_enable: false,
            toggle_global: false,
            group: Some(Some("dev".into())),
        };
        let result = handle_edit(&mut catalog, cmd);
        assert!(result.is_ok());
        let edited_alias = catalog.aliases.get("test").unwrap();
        assert_eq!(edited_alias.command, "edit_command");
        assert_eq!(edited_alias.group.as_deref(), Some("dev"));
    }

    #[test]
    fn test_handle_edit_set_nonexistent_group_create() {
        let mut catalog = create_test_catalog();
        let result = handle_nonexistent_group(&mut catalog, "new_group", |_| true);
        assert!(result.is_ok());
        assert!(catalog.groups.contains_key("new_group"));
    }

    #[test]
    fn test_handle_edit_set_nonexistent_group_decline() {
        let mut catalog = create_test_catalog();
        let result = handle_nonexistent_group(&mut catalog, "new_group", |_| false);
        assert!(result.is_err());
        assert!(!catalog.groups.contains_key("new_group"));
    }
}
