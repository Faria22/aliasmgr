use dialoguer::Confirm;

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prompt_overwrite_existing_alias(alias: &str) -> bool {
    Confirm::new()
        .with_prompt(format!(
            "Alias {} already exists. Do you want to overwrite it?",
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
pub fn prompt_use_non_existing_config_file(path: &str) -> bool {
    Confirm::new()
        .with_prompt(format!(
            "Configuration file '{}' does not exist. Do you want to use this path anyway?",
            path
        ))
        .default(true)
        .interact()
        .unwrap()
}
