use dialoguer::{Confirm, Select};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_overwrite_existing_alias(alias: &str) -> bool {
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
    Confirm::new()
        .with_prompt("Are you sure you want to remove all aliases and groups?")
        .default(false)
        .interact()
        .unwrap()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_alias_or_group(name: &str, action: &str) -> bool {
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
    reassign_group_confirm(name).interact().unwrap()
}

fn reassign_group_confirm(name: &str) -> Confirm<'static> {
    Confirm::new()
        .with_prompt(format!(
            "Move aliases from group '{}' to ungrouped instead of removing them?",
            name
        ))
        .default(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_alias_or_group_select() {
        drop(alias_or_group_select("tools", "enabled"));
    }

    #[test]
    fn builds_reassign_group_confirm() {
        drop(reassign_group_confirm("tools"));
    }
}
