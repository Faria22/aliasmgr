use log::{error, warn};

use crate::catalog::types::{Alias, AliasCatalog};
use crate::cli::add::AddCommand;
use crate::cli::interaction::{InteractionMode, prompt_overwrite_existing_alias};
use crate::core::add::add_alias;
use crate::core::conflict::conflict_warnings;
use crate::core::edit::edit_alias;
use crate::core::validation::is_valid_alias_name;
use crate::core::{Failure, Outcome};

use super::shell::ShellType;

pub fn handle_add(
    catalog: &mut AliasCatalog,
    args: AddCommand,
    shell: &ShellType,
    interaction_mode: InteractionMode,
) -> Result<Outcome, Failure> {
    if args.global && *shell != ShellType::Zsh {
        error!("Global aliases are only supported in zsh.");
        return Err(Failure::UnsupportedGlobalAlias);
    }
    if !is_valid_alias_name(&args.name) {
        error!("Invalid alias name '{}'.", args.name);
        return Err(Failure::InvalidAliasName);
    }

    let mut alias = Alias::new(args.command, !args.disabled, args.global);
    alias.description = args.description;
    alias.tags.extend(args.tag);
    alias.refresh_representation();

    let outcome = if catalog.aliases.contains_key(&args.name) {
        if prompt_overwrite_existing_alias(interaction_mode, &args.name) {
            edit_alias(catalog, &args.name, &alias)?
        } else {
            Outcome::NoChanges
        }
    } else {
        add_alias(catalog, &args.name, &alias)?
    };

    if outcome == Outcome::CatalogChanged {
        for warning in conflict_warnings([args.name.as_str()], shell)
            .get(&args.name)
            .into_iter()
            .flatten()
        {
            warn!("{warning}");
        }
    }
    Ok(outcome)
}
