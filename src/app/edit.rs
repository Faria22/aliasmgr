use log::warn;

use crate::catalog::types::AliasCatalog;
use crate::cli::edit::EditCommand;
use crate::core::conflict::conflict_warnings;
use crate::core::edit::edit_alias;
use crate::core::{Failure, Outcome};

use super::edit_tui::{self, EditorMode, EditorResult};
use super::shell::ShellType;

pub fn handle_edit(
    catalog: &mut AliasCatalog,
    cmd: EditCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    if cmd.interactive {
        let mode = if cmd.all {
            EditorMode::All
        } else {
            EditorMode::Single
        };
        let result = edit_tui::run(catalog, cmd.name.as_deref(), mode, shell.clone())
            .map_err(Failure::InteractiveEditor)?;
        return Ok(apply_interactive_result(catalog, result, shell));
    }
    let name = cmd
        .name
        .as_deref()
        .expect("validated non-interactive edit name");
    let mut alias = catalog
        .aliases
        .get(name)
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
    if cmd.global {
        if !alias.global && *shell != ShellType::Zsh {
            return Err(Failure::UnsupportedGlobalAlias);
        }
        alias.global = true;
    }
    if cmd.no_global {
        alias.global = false;
    }
    let outcome = edit_alias(catalog, name, &alias)?;
    if outcome == Outcome::CatalogChanged {
        for warning in conflict_warnings([name], shell)
            .get(name)
            .into_iter()
            .flatten()
        {
            warn!("{warning}");
        }
    }
    Ok(outcome)
}

fn apply_interactive_result(
    catalog: &mut AliasCatalog,
    result: EditorResult,
    shell: &ShellType,
) -> Outcome {
    match result {
        EditorResult::Continue => {
            unreachable!("the interactive event loop cannot return Continue")
        }
        EditorResult::ExitNoChanges => Outcome::NoChanges,
        EditorResult::Save(updated) => {
            let changed_names = updated
                .aliases
                .iter()
                .filter_map(|(name, alias)| {
                    catalog
                        .aliases
                        .get(name)
                        .is_none_or(|existing| {
                            existing.command != alias.command
                                || existing.enabled != alias.enabled
                                || existing.global != alias.global
                                || existing.description != alias.description
                                || existing.tags != alias.tags
                        })
                        .then_some(name.as_str())
                })
                .collect::<Vec<_>>();
            for warning in conflict_warnings(changed_names, shell).values().flatten() {
                warn!("{warning}");
            }
            *catalog = updated;
            Outcome::CatalogChanged
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    #[test]
    fn interactive_results_apply_only_saved_catalogs() {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("old".into(), Alias::new("old".into(), true, false));
        assert_eq!(
            apply_interactive_result(&mut catalog, EditorResult::ExitNoChanges, &ShellType::Bash),
            Outcome::NoChanges
        );
        assert!(catalog.aliases.contains_key("old"));

        let mut updated = AliasCatalog::new();
        updated
            .aliases
            .insert("new".into(), Alias::new("new".into(), true, false));
        assert_eq!(
            apply_interactive_result(&mut catalog, EditorResult::Save(updated), &ShellType::Bash),
            Outcome::CatalogChanged
        );
        assert!(catalog.aliases.contains_key("new"));
        assert!(!catalog.aliases.contains_key("old"));
    }
}
