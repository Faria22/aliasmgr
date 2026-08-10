use crate::catalog::types::AliasCatalog;
use crate::cli::disable::{DisableCommand, DisableTarget};
use crate::cli::interaction::InteractionMode;
use crate::core::disable::{disable_alias, disable_aliases, disable_all};
use crate::core::selector::{aliases_with_tag, select_aliases};
use crate::core::{Failure, Outcome};

use super::CommandOutcome;
use super::shell::ShellType;

pub fn handle_disable(
    catalog: &mut AliasCatalog,
    cmd: DisableCommand,
    _shell: &ShellType,
    _interaction_mode: InteractionMode,
) -> Result<CommandOutcome, Failure> {
    match cmd.target {
        Some(DisableTarget::Alias(args)) if args.is_filter() => {
            let names = select_aliases(catalog, args.pattern.as_deref(), &args.tag)?;
            let matched = names.len();
            let (outcome, changed) = disable_aliases(catalog, &names);
            Ok(CommandOutcome::with_message(
                outcome,
                if matched == 0 {
                    "No aliases matched the selector.".into()
                } else {
                    format!("Disabled {changed} of {matched} matching aliases.")
                },
            ))
        }
        Some(DisableTarget::Alias(args)) => {
            disable_alias(catalog, args.name.as_deref().expect("exact name required"))
                .map(CommandOutcome::from)
        }
        Some(DisableTarget::Tag(args)) => {
            let names = aliases_with_tag(catalog, &args.name)?;
            let matched = names.len();
            let (outcome, changed) = disable_aliases(catalog, &names);
            Ok(CommandOutcome::with_message(
                outcome,
                format!(
                    "Disabled {changed} of {matched} aliases tagged '{}'.",
                    args.name
                ),
            ))
        }
        Some(DisableTarget::All) => {
            let outcome = disable_all(catalog);
            let message = if outcome == Outcome::CatalogChanged {
                "All aliases are now disabled."
            } else {
                "All aliases are already disabled."
            };
            Ok(CommandOutcome::with_message(outcome, message))
        }
        None => disable_alias(catalog, cmd.name.as_deref().expect("name required"))
            .map(CommandOutcome::from),
    }
}
