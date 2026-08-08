use crate::catalog::types::AliasCatalog;
use crate::cli::interaction::prompt_alias_or_group;
use crate::cli::rename::{RenameCommand, RenameTarget};
use crate::core::rename::{rename_alias, rename_group};
use crate::core::{Failure, Outcome};

use super::resource::{ResourceType, resolve_resource_type};

fn handle_rename_shorthand(
    catalog: &mut AliasCatalog,
    old_name: &str,
    new_name: &str,
    choose_alias: impl FnOnce(&str) -> bool,
) -> Result<Outcome, Failure> {
    match resolve_resource_type(catalog, old_name, choose_alias) {
        ResourceType::Alias => rename_alias(catalog, old_name, new_name),
        ResourceType::Group => rename_group(catalog, old_name, new_name),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn handle_rename(catalog: &mut AliasCatalog, cmd: RenameCommand) -> Result<Outcome, Failure> {
    match cmd.target {
        Some(RenameTarget::Alias(args)) => rename_alias(catalog, &args.old_name, &args.new_name),
        Some(RenameTarget::Group(args)) => rename_group(catalog, &args.old_name, &args.new_name),
        None => handle_rename_shorthand(
            catalog,
            cmd.old_name
                .as_deref()
                .expect("clap requires an old name when no subcommand is used"),
            cmd.new_name
                .as_deref()
                .expect("clap requires a new name when no subcommand is used"),
            |name| prompt_alias_or_group(name, "renamed"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "ll".to_string(),
            Alias::new("ls -l".to_string(), None, true, false),
        );
        catalog.groups.insert("tools".to_string(), true);
        catalog
    }

    #[test]
    fn renames_shorthand_alias() {
        let mut catalog = sample_catalog();
        let result = handle_rename(
            &mut catalog,
            RenameCommand {
                target: None,
                old_name: Some("ll".to_string()),
                new_name: Some("list".to_string()),
            },
        );

        assert!(result.is_ok());
        assert!(!catalog.aliases.contains_key("ll"));
        assert!(catalog.aliases.contains_key("list"));
        assert!(catalog.groups.contains_key("tools"));
    }

    #[test]
    fn renames_shorthand_group() {
        let mut catalog = sample_catalog();
        let result = handle_rename(
            &mut catalog,
            RenameCommand {
                target: None,
                old_name: Some("tools".to_string()),
                new_name: Some("commands".to_string()),
            },
        );

        assert!(result.is_ok());
        assert!(catalog.aliases.contains_key("ll"));
        assert!(!catalog.groups.contains_key("tools"));
        assert!(catalog.groups.contains_key("commands"));
    }

    #[test]
    fn renames_explicit_alias_and_group() {
        let mut catalog = sample_catalog();
        let alias_result = handle_rename(
            &mut catalog,
            RenameCommand {
                target: Some(RenameTarget::Alias(crate::cli::rename::RenameArgs {
                    old_name: "ll".to_string(),
                    new_name: "list".to_string(),
                })),
                old_name: None,
                new_name: None,
            },
        );
        let group_result = handle_rename(
            &mut catalog,
            RenameCommand {
                target: Some(RenameTarget::Group(crate::cli::rename::RenameArgs {
                    old_name: "tools".to_string(),
                    new_name: "commands".to_string(),
                })),
                old_name: None,
                new_name: None,
            },
        );

        assert!(alias_result.is_ok());
        assert!(group_result.is_ok());
        assert!(catalog.aliases.contains_key("list"));
        assert!(catalog.groups.contains_key("commands"));
    }
}
