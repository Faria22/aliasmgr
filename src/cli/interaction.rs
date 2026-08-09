use dialoguer::{Confirm, Select};
use std::sync::atomic::{AtomicU8, Ordering};

const INTERACTIVE: u8 = 0;
const FORCE: u8 = 1;
const NO_INPUT: u8 = 2;
const INPUT_REQUIRED_EXIT_CODE: i32 = 2;

static INTERACTION_MODE: AtomicU8 = AtomicU8::new(INTERACTIVE);

pub fn configure_interaction(force: bool, no_input: bool) {
    let mode = if force {
        FORCE
    } else if no_input {
        NO_INPUT
    } else {
        INTERACTIVE
    };
    INTERACTION_MODE.store(mode, Ordering::Relaxed);
}

fn non_interactive_answer(prompt: &str) -> Option<bool> {
    match INTERACTION_MODE.load(Ordering::Relaxed) {
        FORCE => Some(true),
        NO_INPUT => {
            eprintln!("ERROR: Input required to {prompt}; --no-input was supplied.");
            std::process::exit(INPUT_REQUIRED_EXIT_CODE);
        }
        _ => None,
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_overwrite_existing_alias(alias: &str) -> bool {
    if let Some(answer) = non_interactive_answer("overwrite an existing alias") {
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
pub fn prompt_create_non_existent_group(group: &str) -> bool {
    if let Some(answer) = non_interactive_answer(&format!("create missing group '{group}'")) {
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
pub fn prompt_use_non_existing_catalog_file(path: &str) -> bool {
    if let Some(answer) = non_interactive_answer(&format!("use missing catalog path '{path}'")) {
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
pub fn prompt_confirm_remove_all() -> bool {
    if let Some(answer) = non_interactive_answer("remove all aliases and groups") {
        return answer;
    }
    Confirm::new()
        .with_prompt("Are you sure you want to remove all aliases and groups?")
        .default(false)
        .interact()
        .unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_alias_or_group(name: &str, action: &str) -> bool {
    if let Some(answer) = non_interactive_answer(&format!(
        "choose whether alias or group '{name}' should be {action}"
    )) {
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
pub fn prompt_reassign_group_aliases(name: &str) -> bool {
    if let Some(answer) = non_interactive_answer(&format!("reassign aliases from group '{name}'")) {
        return answer;
    }
    reassign_group_confirm(name).interact().unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_enable_reassigned_aliases(name: &str, alias_count: usize) -> bool {
    if let Some(answer) = non_interactive_answer(&format!(
        "enable aliases reassigned from disabled group '{name}'"
    )) {
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
        configure_interaction(false, false);
        assert_eq!(non_interactive_answer("continue"), None);
    }

    #[test]
    fn builds_alias_or_group_select() {
        drop(alias_or_group_select("tools", "enabled"));
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
