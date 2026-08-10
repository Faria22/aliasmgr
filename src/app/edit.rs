use log::warn;

use crate::catalog::types::AliasCatalog;
use crate::cli::edit::EditCommand;
use crate::cli::interaction::InteractionMode;
use crate::core::conflict::conflict_warnings;
use crate::core::edit::edit_alias;
use crate::core::{Failure, Outcome};

use super::shell::ShellType;

pub fn handle_edit(
    catalog: &mut AliasCatalog,
    cmd: EditCommand,
    shell: &ShellType,
    _interaction_mode: InteractionMode,
) -> Result<Outcome, Failure> {
    let mut alias = catalog
        .aliases
        .get(&cmd.name)
        .cloned()
        .ok_or(Failure::AliasDoesNotExist)?;
    if let Some(command) = cmd.command {
        alias.command = command;
    }
    if let Some(description) = cmd.description {
        alias.description = Some(description);
    }
    if cmd.clear_description {
        alias.description = None;
    }
    alias.tags.extend(cmd.add_tag);
    for tag in cmd.remove_tag {
        alias.tags.remove(&tag);
    }
    if cmd.toggle_enabled {
        alias.enabled = !alias.enabled;
    }
    if cmd.toggle_global {
        if !alias.global && *shell != ShellType::Zsh {
            return Err(Failure::UnsupportedGlobalAlias);
        }
        alias.global = !alias.global;
    }
    alias.refresh_representation();
    let outcome = edit_alias(catalog, &cmd.name, &alias)?;
    if outcome == Outcome::CatalogChanged {
        for warning in conflict_warnings([cmd.name.as_str()], shell)
            .get(&cmd.name)
            .into_iter()
            .flatten()
        {
            warn!("{warning}");
        }
    }
    Ok(outcome)
}
