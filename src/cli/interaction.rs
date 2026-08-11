use dialoguer::Confirm;

const INPUT_REQUIRED_EXIT_CODE: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionMode {
    Interactive,
    Yes,
    No,
    NoInput,
}

fn non_interactive_answer(mode: InteractionMode, prompt: &str) -> Option<bool> {
    match mode {
        InteractionMode::Yes => Some(true),
        InteractionMode::No => Some(false),
        InteractionMode::NoInput => {
            eprintln!("ERROR: Input required to {prompt}; --no-input was supplied.");
            std::process::exit(INPUT_REQUIRED_EXIT_CODE);
        }
        InteractionMode::Interactive => None,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn confirm(mode: InteractionMode, action: &str, message: String, default: bool) -> bool {
    non_interactive_answer(mode, action).unwrap_or_else(|| {
        Confirm::new()
            .with_prompt(message)
            .default(default)
            .interact()
            .unwrap_or_else(|error| {
                eprintln!(
                    "ERROR: Could not prompt to {action}: {error}. Use --yes to accept, --no to decline, or --no-input to fail without prompting."
                );
                std::process::exit(INPUT_REQUIRED_EXIT_CODE);
            })
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_overwrite_existing_alias(mode: InteractionMode, alias: &str) -> bool {
    confirm(
        mode,
        "overwrite an existing alias",
        format!("Alias \"{alias}\" already exists. Do you want to overwrite it?"),
        true,
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_replace_imported_alias(mode: InteractionMode, alias: &str) -> bool {
    confirm(
        mode,
        "replace an existing alias during import",
        format!("Alias \"{alias}\" already exists. Replace it with the imported alias?"),
        false,
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_use_non_existing_catalog_file(mode: InteractionMode, path: &str) -> bool {
    confirm(
        mode,
        &format!("use missing catalog path '{path}'"),
        format!("Catalog file '{path}' does not exist. Do you want to use this path anyway?"),
        true,
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_confirm_remove_all(mode: InteractionMode) -> bool {
    confirm(
        mode,
        "remove all aliases",
        "Are you sure you want to remove all aliases?".into(),
        false,
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_confirm_remove_aliases(mode: InteractionMode, count: usize) -> bool {
    prompt_confirm_selected_aliases(mode, count, "matching the selector")
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_confirm_remove_tagged_aliases(
    mode: InteractionMode,
    count: usize,
    tag: &str,
) -> bool {
    prompt_confirm_selected_aliases(mode, count, &format!("tagged '{tag}'"))
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn prompt_confirm_selected_aliases(mode: InteractionMode, count: usize, selection: &str) -> bool {
    confirm(
        mode,
        &format!("remove {count} aliases {selection}"),
        format!(
            "Remove {count} alias{} {selection}?",
            if count == 1 { "" } else { "es" },
        ),
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_mode_does_not_supply_an_answer() {
        assert_eq!(
            non_interactive_answer(InteractionMode::Interactive, "continue"),
            None
        );
    }

    #[test]
    fn explicit_modes_supply_their_answer() {
        assert_eq!(
            non_interactive_answer(InteractionMode::Yes, "continue"),
            Some(true)
        );
        assert_eq!(
            non_interactive_answer(InteractionMode::No, "continue"),
            Some(false)
        );
    }
}
