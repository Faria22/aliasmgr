use crate::catalog::types::AliasCatalog;
use crate::core::disable::{disable_alias, disable_all, disable_group};
use crate::core::{Failure, Outcome};

use crate::cli::disable::{DisableCommand, DisableTarget};
use crate::cli::interaction::{InteractionMode, prompt_alias_or_group};

use super::CommandOutcome;
use super::resource::{ResourceType, resolve_resource_type};
use super::shell::ShellType;

fn handle_disable_shorthand(
    catalog: &mut AliasCatalog,
    name: &str,
    shell: &ShellType,
    choose_alias: impl FnOnce(&str) -> bool,
) -> Result<Outcome, Failure> {
    match resolve_resource_type(catalog, name, choose_alias) {
        ResourceType::Alias => disable_alias(catalog, name),
        ResourceType::Group => disable_group(catalog, name, shell),
    }
}

pub fn handle_disable(
    catalog: &mut AliasCatalog,
    cmd: DisableCommand,
    shell: &ShellType,
    interaction_mode: InteractionMode,
) -> Result<CommandOutcome, Failure> {
    match cmd.target {
        Some(DisableTarget::Alias(args)) => {
            disable_alias(catalog, &args.name).map(CommandOutcome::from)
        }
        Some(DisableTarget::Group(args)) => {
            disable_group(catalog, &args.name, shell).map(CommandOutcome::from)
        }
        Some(DisableTarget::All) => disable_all(catalog).map(|outcome| {
            let message = match outcome {
                Outcome::CatalogChanged => "All aliases and groups are now disabled.",
                Outcome::NoChanges => "All aliases and groups are already disabled.",
            };
            CommandOutcome::with_message(outcome, message)
        }),
        None => handle_disable_shorthand(
            catalog,
            cmd.name
                .as_deref()
                .expect("clap requires a name when no subcommand is used"),
            shell,
            |name| prompt_alias_or_group(interaction_mode, name, "disabled"),
        )
        .map(CommandOutcome::from),
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
            Alias::new("ls -l".to_string(), None, true, false),
        );
        catalog.groups.insert("tools".to_string(), true);
        catalog
    }

    #[test]
    fn disables_shorthand_alias() {
        let mut catalog = sample_catalog();
        let result = handle_disable(
            &mut catalog,
            DisableCommand {
                target: None,
                name: Some("ll".to_string()),
            },
            &ShellType::Bash,
            InteractionMode::Interactive,
        );

        assert!(result.is_ok());
        assert!(!catalog.aliases["ll"].enabled);
        assert!(catalog.groups["tools"]);
    }

    #[test]
    fn disables_shorthand_group() {
        let mut catalog = sample_catalog();
        let result = handle_disable(
            &mut catalog,
            DisableCommand {
                target: None,
                name: Some("tools".to_string()),
            },
            &ShellType::Bash,
            InteractionMode::Interactive,
        );

        assert!(result.is_ok());
        assert!(catalog.aliases["ll"].enabled);
        assert!(!catalog.groups["tools"]);
    }

    #[test]
    fn disables_explicit_alias_and_group() {
        let mut catalog = sample_catalog();
        let alias_result = handle_disable(
            &mut catalog,
            DisableCommand {
                target: Some(DisableTarget::Alias(crate::cli::disable::DisableArgs {
                    name: "ll".to_string(),
                })),
                name: None,
            },
            &ShellType::Bash,
            InteractionMode::Interactive,
        );
        let group_result = handle_disable(
            &mut catalog,
            DisableCommand {
                target: Some(DisableTarget::Group(crate::cli::disable::DisableArgs {
                    name: "tools".to_string(),
                })),
                name: None,
            },
            &ShellType::Bash,
            InteractionMode::Interactive,
        );

        assert!(alias_result.is_ok());
        assert!(group_result.is_ok());
        assert!(!catalog.aliases["ll"].enabled);
        assert!(!catalog.groups["tools"]);
    }

    #[test]
    fn disables_all_aliases_and_groups() {
        let mut catalog = sample_catalog();

        let result = handle_disable(
            &mut catalog,
            DisableCommand {
                target: Some(DisableTarget::All),
                name: None,
            },
            &ShellType::Bash,
            InteractionMode::Interactive,
        );

        assert_eq!(
            result,
            Ok(CommandOutcome::with_message(
                Outcome::CatalogChanged,
                "All aliases and groups are now disabled."
            ))
        );
        assert!(!catalog.aliases["ll"].enabled);
        assert!(!catalog.groups["tools"]);
    }
}
