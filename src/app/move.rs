use crate::core::{Failure, Outcome};

use crate::core::add::add_group;
use crate::core::r#move::move_alias;

use crate::catalog::types::AliasCatalog;

use crate::cli::interaction::prompt_create_non_existent_group;
use crate::cli::r#move::MoveCommand;

use log::{debug, error, info};

pub fn handle_move(catalog: &mut AliasCatalog, cmd: MoveCommand) -> Result<Outcome, Failure> {
    match move_alias(catalog, &cmd.name, &cmd.new_group) {
        Ok(outcome) => Ok(outcome),
        Err(e) => match e {
            Failure::GroupDoesNotExist => handle_non_existing_group(
                catalog,
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
    catalog: &mut AliasCatalog,
    alias: &str,
    group: &str,
    create_group: impl Fn(&str) -> bool,
) -> Result<Outcome, Failure> {
    if create_group(group) {
        info!("Created new group '{}'", group);
        add_group(catalog, group, true)?;
        move_alias(catalog, alias, &Some(group.to_string()))
    } else {
        debug!(
            "User aborted moving alias '{}' to non-existent group '{}'",
            &alias, group
        );
        Ok(Outcome::NoChanges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;
    use assert_matches::assert_matches;

    fn create_catalog_with_alias(alias_name: &str, command: &str) -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            alias_name.into(),
            Alias::new(command.into(), None, true, false),
        );
        catalog
    }

    #[test]
    fn test_handle_non_existing_group_create() {
        let mut catalog = create_catalog_with_alias("ll", "ls -la");

        let outcome = handle_non_existing_group(&mut catalog, "ll", "new_group", |_| true).unwrap();

        assert_matches!(outcome, Outcome::CatalogChanged);
        assert!(catalog.groups.contains_key("new_group"));
        assert_eq!(
            catalog.aliases.get("ll").unwrap().group,
            Some("new_group".into())
        );
    }

    #[test]
    fn test_handle_non_existing_group_abort() {
        let mut catalog = create_catalog_with_alias("ll", "ls -la");
        let outcome =
            handle_non_existing_group(&mut catalog, "ll", "new_group", |_| false).unwrap();
        assert_matches!(outcome, Outcome::NoChanges);
        assert!(!catalog.groups.contains_key("new_group"));
        assert_eq!(catalog.aliases.get("ll").unwrap().group, None);
    }

    #[test]
    fn test_handle_move_alias_to_existing_group() {
        let mut catalog = create_catalog_with_alias("ll", "ls -la");
        catalog.groups.insert("utilities".into(), true);
        let cmd = MoveCommand {
            name: "ll".into(),
            new_group: Some("utilities".into()),
        };
        let outcome = handle_move(&mut catalog, cmd).unwrap();
        assert_matches!(outcome, Outcome::CatalogChanged);
        assert_eq!(
            catalog.aliases.get("ll"),
            Some(&Alias::new(
                "ls -la".into(),
                Some("utilities".into()),
                true,
                false
            ))
        );
    }

    #[test]
    fn test_move_non_existent_alias() {
        let mut catalog = AliasCatalog::new();
        let cmd = MoveCommand {
            name: "nonexistent".into(),
            new_group: Some("utilities".into()),
        };
        let result = handle_move(&mut catalog, cmd);
        assert_matches!(result, Err(Failure::AliasDoesNotExist));
        assert!(!catalog.aliases.contains_key("nonexistent"));
        assert!(!catalog.groups.contains_key("utilities"));
    }
}
