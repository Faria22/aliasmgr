use crate::catalog::types::AliasCatalog;
use crate::cli::interaction::InteractionMode;
use crate::cli::rename::{RenameCommand, RenameTarget};
use crate::core::Failure;
use crate::core::rename::{rename_alias, rename_tag};

use super::CommandOutcome;

pub fn handle_rename(
    catalog: &mut AliasCatalog,
    cmd: RenameCommand,
    _interaction_mode: InteractionMode,
) -> Result<CommandOutcome, Failure> {
    match cmd.target {
        Some(RenameTarget::Alias(args)) => {
            rename_alias(catalog, &args.old_name, &args.new_name).map(CommandOutcome::from)
        }
        Some(RenameTarget::Tag(args)) => {
            let (outcome, changed) = rename_tag(catalog, &args.old_name, &args.new_name)?;
            Ok(CommandOutcome::with_message(
                outcome,
                format!(
                    "Renamed tag '{}' to '{}' on {changed} aliases.",
                    args.old_name, args.new_name
                ),
            ))
        }
        None => rename_alias(
            catalog,
            cmd.old_name.as_deref().expect("old name required"),
            cmd.new_name.as_deref().expect("new name required"),
        )
        .map(CommandOutcome::from),
    }
}
