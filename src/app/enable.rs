use crate::catalog::types::AliasCatalog;
use crate::cli::enable::{EnableCommand, EnableTarget};
use crate::core::enable::{enable_alias, enable_aliases, enable_all};
use crate::core::selector::{aliases_with_tag, select_aliases};
use crate::core::{Failure, Outcome};

use super::CommandOutcome;
pub fn handle_enable(
    catalog: &mut AliasCatalog,
    cmd: EnableCommand,
) -> Result<CommandOutcome, Failure> {
    match cmd.target {
        Some(EnableTarget::Alias(args)) if args.is_filter() => {
            let names = select_aliases(catalog, args.pattern.as_deref(), &args.tag)?;
            let matched = names.len();
            let (outcome, changed) = enable_aliases(catalog, &names);
            Ok(CommandOutcome::with_message(
                outcome,
                if matched == 0 {
                    "No aliases matched the selector.".into()
                } else {
                    format!("Enabled {changed} of {matched} matching aliases.")
                },
            ))
        }
        Some(EnableTarget::Alias(args)) => {
            enable_alias(catalog, args.name.as_deref().expect("exact name required"))
                .map(CommandOutcome::from)
        }
        Some(EnableTarget::Tag(args)) => {
            let names = aliases_with_tag(catalog, &args.name)?;
            let matched = names.len();
            let (outcome, changed) = enable_aliases(catalog, &names);
            Ok(CommandOutcome::with_message(
                outcome,
                format!(
                    "Enabled {changed} of {matched} aliases tagged '{}'.",
                    args.name
                ),
            ))
        }
        Some(EnableTarget::All) => {
            let outcome = enable_all(catalog);
            let message = if outcome == Outcome::CatalogChanged {
                "All aliases are now enabled."
            } else {
                "All aliases are already enabled."
            };
            Ok(CommandOutcome::with_message(outcome, message))
        }
        None => enable_alias(catalog, cmd.name.as_deref().expect("name required"))
            .map(CommandOutcome::from),
    }
}
