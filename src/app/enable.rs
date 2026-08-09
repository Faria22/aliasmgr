use crate::catalog::types::AliasCatalog;
use crate::core::enable::{enable_alias, enable_all, enable_group};
use crate::core::{Failure, Outcome};

use crate::cli::enable::{EnableCommand, EnableTarget};
use crate::cli::interaction::prompt_alias_or_group;

use super::resource::{ResourceType, resolve_resource_type};
use super::shell::ShellType;

fn handle_enable_shorthand(
    catalog: &mut AliasCatalog,
    name: &str,
    shell: &ShellType,
    choose_alias: impl FnOnce(&str) -> bool,
) -> Result<Outcome, Failure> {
    match resolve_resource_type(catalog, name, choose_alias) {
        ResourceType::Alias => enable_alias(catalog, name),
        ResourceType::Group => enable_group(catalog, name, shell),
    }
}

pub fn handle_enable(
    catalog: &mut AliasCatalog,
    cmd: EnableCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        Some(EnableTarget::Alias(args)) => enable_alias(catalog, &args.name),
        Some(EnableTarget::Group(args)) => enable_group(catalog, &args.name, shell),
        Some(EnableTarget::All) => enable_all(catalog),
        None => handle_enable_shorthand(
            catalog,
            cmd.name
                .as_deref()
                .expect("clap requires a name when no subcommand is used"),
            shell,
            |name| prompt_alias_or_group(name, "enabled"),
        ),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "ll".to_string(),
            Alias::new("ls -l".to_string(), None, false, false),
        );
        catalog.groups.insert("tools".to_string(), false);
        catalog
    }

    #[test]
    fn enables_shorthand_alias() {
        let mut catalog = sample_catalog();
        let result = handle_enable(
            &mut catalog,
            EnableCommand {
                target: None,
                name: Some("ll".to_string()),
            },
            &ShellType::Bash,
        );

        assert!(result.is_ok());
        assert!(catalog.aliases["ll"].enabled);
        assert!(!catalog.groups["tools"]);
    }

    #[test]
    fn enables_shorthand_group() {
        let mut catalog = sample_catalog();
        let result = handle_enable(
            &mut catalog,
            EnableCommand {
                target: None,
                name: Some("tools".to_string()),
            },
            &ShellType::Bash,
        );

        assert!(result.is_ok());
        assert!(!catalog.aliases["ll"].enabled);
        assert!(catalog.groups["tools"]);
    }

    #[test]
    fn enables_explicit_alias_and_group() {
        let mut catalog = sample_catalog();
        let alias_result = handle_enable(
            &mut catalog,
            EnableCommand {
                target: Some(EnableTarget::Alias(crate::cli::enable::EnableArgs {
                    name: "ll".to_string(),
                })),
                name: None,
            },
            &ShellType::Bash,
        );
        let group_result = handle_enable(
            &mut catalog,
            EnableCommand {
                target: Some(EnableTarget::Group(crate::cli::enable::EnableArgs {
                    name: "tools".to_string(),
                })),
                name: None,
            },
            &ShellType::Bash,
        );

        assert!(alias_result.is_ok());
        assert!(group_result.is_ok());
        assert!(catalog.aliases["ll"].enabled);
        assert!(catalog.groups["tools"]);
    }

    #[test]
    fn enables_all_aliases_and_groups() {
        let mut catalog = sample_catalog();

        let result = handle_enable(
            &mut catalog,
            EnableCommand {
                target: Some(EnableTarget::All),
                name: None,
            },
            &ShellType::Bash,
        );

        assert_eq!(result, Ok(Outcome::CatalogChanged));
        assert!(catalog.aliases["ll"].enabled);
        assert!(catalog.groups["tools"]);
    }
}
