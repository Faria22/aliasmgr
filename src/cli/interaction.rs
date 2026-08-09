use dialoguer::{Confirm, Select};

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
    if let Some(answer) = non_interactive_answer(mode, "overwrite an existing alias") {
        return answer;
    }
    Confirm::new()
        .with_prompt(format!(
            "Alias \"{}\" already exists. Do you want to overwrite it?",
            alias
        ))
        .default(true)
        .interact()
        .unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_create_non_existent_group(mode: InteractionMode, group: &str) -> bool {
    if let Some(answer) = non_interactive_answer(mode, &format!("create missing group '{group}'")) {
        return answer;
    }
    Confirm::new()
        .with_prompt(format!(
            "Group '{}' does not exist. Do you want to create it?",
            group
        ))
        .default(true)
        .interact()
        .unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_use_non_existing_catalog_file(mode: InteractionMode, path: &str) -> bool {
    if let Some(answer) =
        non_interactive_answer(mode, &format!("use missing catalog path '{path}'"))
    {
        return answer;
    }
    Confirm::new()
        .with_prompt(format!(
            "Catalog file '{}' does not exist. Do you want to use this path anyway?",
            path
        ))
        .default(true)
        .interact()
        .unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_confirm_remove_all(mode: InteractionMode) -> bool {
    if let Some(answer) = non_interactive_answer(mode, "remove all aliases and groups") {
        return answer;
    }
    Confirm::new()
        .with_prompt("Are you sure you want to remove all aliases and groups?")
        .default(false)
        .interact()
        .unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_confirm_remove_aliases(mode: InteractionMode, alias_count: usize) -> bool {
    if let Some(answer) = non_interactive_answer(
        mode,
        &format!("remove {alias_count} aliases matching the selector"),
    ) {
        return answer;
    }
    remove_aliases_confirm(alias_count).interact().unwrap()
}

fn remove_aliases_confirm(alias_count: usize) -> Confirm<'static> {
    Confirm::new()
        .with_prompt(format!(
            "Remove {alias_count} alias{} matching the selector?",
            if alias_count == 1 { "" } else { "es" }
        ))
        .default(false)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_alias_or_group(mode: InteractionMode, name: &str, action: &str) -> bool {
    if let Some(answer) = non_interactive_answer(
        mode,
        &format!("choose whether alias or group '{name}' should be {action}"),
    ) {
        return answer;
    }
    alias_or_group_select(name, action).interact().unwrap() == 0
}

fn alias_or_group_select(name: &str, action: &str) -> Select<'static> {
    Select::new()
        .with_prompt(format!(
            "An alias and a group named '{}' both exist. Which should be {}?",
            name, action
        ))
        .items(["Alias", "Group"])
        .default(0)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_reassign_group_aliases(mode: InteractionMode, name: &str) -> bool {
    if let Some(answer) =
        non_interactive_answer(mode, &format!("reassign aliases from group '{name}'"))
    {
        return answer;
    }
    reassign_group_confirm(name).interact().unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_enable_reassigned_aliases(
    mode: InteractionMode,
    name: &str,
    alias_count: usize,
) -> bool {
    if let Some(answer) = non_interactive_answer(
        mode,
        &format!("enable aliases reassigned from disabled group '{name}'"),
    ) {
        return answer;
    }
    enable_reassigned_aliases_confirm(name, alias_count)
        .interact()
        .unwrap()
}

fn reassign_group_confirm(name: &str) -> Confirm<'static> {
    Confirm::new()
        .with_prompt(format!(
            "Move aliases from group '{}' to ungrouped instead of removing them?",
            name
        ))
        .default(false)
}

fn enable_reassigned_aliases_confirm(name: &str, alias_count: usize) -> Confirm<'static> {
    Confirm::new()
        .with_prompt(format!(
            "Group '{}' is disabled. Enable its {} individually enabled alias{} after reassignment?",
            name,
            alias_count,
            if alias_count == 1 { "" } else { "es" }
        ))
        .default(false)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn builds_alias_or_group_select() {
        drop(alias_or_group_select("tools", "enabled"));
    }

    #[test]
    fn builds_remove_aliases_confirm() {
        drop(remove_aliases_confirm(2));
    }

    #[test]
    fn builds_reassign_group_confirm() {
        drop(reassign_group_confirm("tools"));
    }

    #[test]
    fn builds_enable_reassigned_aliases_confirm() {
        drop(enable_reassigned_aliases_confirm("tools", 2));
    }
}
