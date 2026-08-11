use dialoguer::Confirm;

const INPUT_REQUIRED_EXIT_CODE: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionMode {
    Interactive,
    Force,
    NoInput,
}

fn non_interactive_answer(mode: InteractionMode, prompt: &str) -> Option<bool> {
    match mode {
        InteractionMode::Force => Some(true),
        InteractionMode::NoInput => {
            eprintln!("ERROR: Input required to {prompt}; --no-input was supplied.");
            std::process::exit(INPUT_REQUIRED_EXIT_CODE);
        }
        InteractionMode::Interactive => None,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_overwrite_existing_alias(mode: InteractionMode, alias: &str) -> bool {
    non_interactive_answer(mode, "overwrite an existing alias").unwrap_or_else(|| {
        Confirm::new()
            .with_prompt(format!(
                "Alias \"{alias}\" already exists. Do you want to overwrite it?"
            ))
            .default(true)
            .interact()
            .unwrap()
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_replace_imported_alias(mode: InteractionMode, alias: &str) -> bool {
    non_interactive_answer(mode, "replace an existing alias during import").unwrap_or_else(|| {
        Confirm::new()
            .with_prompt(format!(
                "Alias \"{alias}\" already exists. Replace it with the imported alias?"
            ))
            .default(false)
            .interact()
            .unwrap()
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_use_non_existing_catalog_file(mode: InteractionMode, path: &str) -> bool {
    non_interactive_answer(mode, &format!("use missing catalog path '{path}'")).unwrap_or_else(
        || {
            Confirm::new()
                .with_prompt(format!(
                    "Catalog file '{path}' does not exist. Do you want to use this path anyway?"
                ))
                .default(true)
                .interact()
                .unwrap()
        },
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_confirm_remove_all(mode: InteractionMode) -> bool {
    non_interactive_answer(mode, "remove all aliases").unwrap_or_else(|| {
        Confirm::new()
            .with_prompt("Are you sure you want to remove all aliases?")
            .default(false)
            .interact()
            .unwrap()
    })
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
    non_interactive_answer(mode, &format!("remove {count} aliases {selection}")).unwrap_or_else(
        || {
            Confirm::new()
                .with_prompt(format!(
                    "Remove {count} alias{} {selection}?",
                    if count == 1 { "" } else { "es" },
                ))
                .default(false)
                .interact()
                .unwrap()
        },
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
}
