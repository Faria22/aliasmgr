use crate::catalog::types::AliasCatalog;
use crate::cli::interaction::{
    InteractionMode, prompt_confirm_remove_aliases, prompt_confirm_remove_all,
};
use crate::cli::remove::{RemoveCommand, RemoveTarget};
use crate::core::remove::{remove_alias, remove_aliases, remove_all, remove_tag};
use crate::core::selector::select_aliases;
use crate::core::{Failure, Outcome};

use super::CommandOutcome;
use super::shell::ShellType;

pub fn handle_remove(
    catalog: &mut AliasCatalog,
    cmd: RemoveCommand,
    _shell: &ShellType,
    interaction_mode: InteractionMode,
) -> Result<CommandOutcome, Failure> {
    match cmd.target {
        Some(RemoveTarget::Alias(args)) if args.is_filter() => {
            let names = select_aliases(catalog, args.pattern.as_deref(), &args.tag)?;
            let matched = names.len();
            let confirmed = matched > 0 && prompt_confirm_remove_aliases(interaction_mode, matched);
            let outcome = if confirmed {
                remove_aliases(catalog, &names)
            } else {
                Outcome::NoChanges
            };
            Ok(CommandOutcome::with_message(
                outcome,
                if confirmed {
                    format!("Removed {matched} of {matched} matching aliases.")
                } else if matched == 0 {
                    "No aliases matched the selector.".into()
                } else {
                    format!("Removed 0 of {matched} matching aliases.")
                },
            ))
        }
        Some(RemoveTarget::Alias(args)) => {
            remove_alias(catalog, args.name.as_deref().expect("exact name required"))
                .map(CommandOutcome::from)
        }
        Some(RemoveTarget::Tag(args)) => {
            let (outcome, changed) = remove_tag(catalog, &args.name)?;
            Ok(CommandOutcome::with_message(
                outcome,
                format!("Removed tag '{}' from {changed} aliases.", args.name),
            ))
        }
        Some(RemoveTarget::All) => {
            if prompt_confirm_remove_all(interaction_mode) {
                Ok(CommandOutcome::with_message(
                    remove_all(catalog),
                    "Removed all aliases.",
                ))
            } else {
                Ok(CommandOutcome::from(Outcome::NoChanges))
            }
        }
        None => remove_alias(catalog, cmd.name.as_deref().expect("name required"))
            .map(CommandOutcome::from),
    }
}
