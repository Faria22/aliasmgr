use super::list::get_aliases_from_single_group;
use super::{Failure, Outcome};

use crate::catalog::types::AliasCatalog;

use crate::app::shell::ShellType;

use log::error;

pub fn disable_alias(catalog: &mut AliasCatalog, name: &str) -> Result<Outcome, Failure> {
    if !catalog.aliases.contains_key(name) {
        error!("Alias {} does not exist.", name);
        return Err(Failure::AliasDoesNotExist);
    }

    let alias = catalog.aliases.get_mut(name).unwrap();

    if !alias.enabled {
        return Ok(Outcome::NoChanges);
    }

    alias.enabled = false;

    // Checks if the group the alias is in is disabled
    // If it is, then the alias will not be removed from the shell
    if let Some(group) = &alias.group
        && !catalog.groups[group]
    {
        return Ok(Outcome::CatalogChanged);
    }

    Ok(Outcome::Command(format!("unalias '{}'", name)))
}

pub fn disable_group(
    catalog: &mut AliasCatalog,
    name: &str,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    if !catalog.groups.contains_key(name) {
        error!("Group {} does not exist.", name);
        return Err(Failure::GroupDoesNotExist);
    }

    // If the group is already disabled, do nothing
    if !catalog.groups[name] {
        return Ok(Outcome::NoChanges);
    }

    *catalog.groups.get_mut(name).unwrap() = false;

    // Get all aliases in the group that are enabled and remove them from the shell
    let mut aliases_in_group = get_aliases_from_single_group(catalog, Some(name), shell)?;
    aliases_in_group.retain(|alias_name| catalog.aliases[alias_name].enabled);

    if aliases_in_group.is_empty() {
        return Ok(Outcome::CatalogChanged);
    }

    let mut command = String::new();
    for alias_name in aliases_in_group {
        command.push_str(&format!("unalias '{}'\n", alias_name));
        command.push('\n');
    }

    Ok(Outcome::Command(command))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use crate::catalog::types::Alias;
    use assert_matches::assert_matches;

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("enabled_group".into(), true);
        catalog.groups.insert("disabled_group".into(), false);
        catalog.groups.insert("empty_group".into(), true);

        catalog.aliases.insert(
            "alias1".into(),
            Alias::new("cmd".into(), Some("enabled_group".into()), true, false),
        );
        catalog.aliases.insert(
            "alias2".into(),
            Alias::new("cmd".into(), Some("disabled_group".into()), true, false),
        );

        catalog
    }

    #[test]
    fn disable_existing_alias() {
        let mut catalog = sample_catalog();
        let result = disable_alias(&mut catalog, "alias1");
        assert!(result.is_ok());
        assert!(!catalog.aliases["alias1"].enabled);
        assert_matches!(result.unwrap(), Outcome::Command(_));
    }

    #[test]
    fn disable_disabled_alias() {
        let mut catalog = sample_catalog();
        let _ = disable_alias(&mut catalog, "alias1");
        assert!(!catalog.aliases["alias1"].enabled);

        let result = disable_alias(&mut catalog, "alias1");
        assert!(result.is_ok());
        assert!(!catalog.aliases["alias1"].enabled);
        assert_matches!(result.unwrap(), Outcome::NoChanges);
    }

    #[test]
    fn disable_nonexistent_alias() {
        let mut catalog = sample_catalog();
        let result = disable_alias(&mut catalog, "nonexisting");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Failure::AliasDoesNotExist);
    }

    #[test]
    fn disable_alias_in_disabled_group() {
        let mut catalog = sample_catalog();
        let result = disable_alias(&mut catalog, "alias2");
        assert!(result.is_ok());
        assert!(!catalog.aliases["alias2"].enabled);
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
    }

    #[test]
    fn disable_disabled_group() {
        let mut catalog = sample_catalog();
        let result = disable_group(&mut catalog, "disabled_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(!catalog.groups["disabled_group"]);
        assert_eq!(result.unwrap(), Outcome::NoChanges);
    }

    #[test]
    fn disable_nonexistent_group() {
        let mut catalog = sample_catalog();
        let result = disable_group(&mut catalog, "nonexisting", &ShellType::Bash);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), Failure::GroupDoesNotExist);
    }

    #[test]
    fn disable_empty_group() {
        let mut catalog = sample_catalog();
        let result = disable_group(&mut catalog, "empty_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(!catalog.groups["empty_group"]);
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
    }

    #[test]
    fn disable_group_with_enabled_aliases() {
        let mut catalog = sample_catalog();
        let result = disable_group(&mut catalog, "enabled_group", &ShellType::Bash);
        assert!(result.is_ok());
        assert!(!catalog.groups["enabled_group"]);
        assert_matches!(result.unwrap(), Outcome::Command(_));
    }

    #[test]
    fn disable_group_with_disabled_aliases() {
        let mut catalog = sample_catalog();
        let _ = disable_alias(&mut catalog, "alias1");
        assert!(!catalog.aliases["alias1"].enabled);

        let result = disable_group(&mut catalog, "enabled_group", &ShellType::Bash);
        assert!(!catalog.groups["enabled_group"]);
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
    }
}
