use super::add::add_alias;
use super::remove::remove_alias;
use super::{Failure, Outcome};
use crate::catalog::types::AliasCatalog;

use log::error;

pub fn rename_alias(
    catalog: &mut AliasCatalog,
    old_alias: &str,
    new_alias: &str,
) -> Result<Outcome, Failure> {
    if !catalog.aliases.contains_key(old_alias) {
        error!("Alias {} does not exists.", old_alias);
        return Err(Failure::AliasDoesNotExist);
    }

    if catalog.aliases.contains_key(new_alias) {
        error!("Alias {} already exists.", new_alias);
        return Err(Failure::AliasAlreadyExists);
    }

    let mut command = String::new();
    let alias = catalog.aliases[old_alias].clone();

    if let Outcome::Command(cmd) = remove_alias(catalog, old_alias)? {
        command.push_str(&cmd);
        command.push('\n');
    } else {
        unreachable!("Unexpected behavior when removing alias {}", old_alias);
    }

    if let Outcome::Command(cmd) = add_alias(catalog, new_alias, &alias)? {
        command.push_str(&cmd);
    } else {
        unreachable!("Unexpected behavior when adding alias {}", new_alias);
    }

    Ok(Outcome::Command(command))
}

pub fn rename_group(
    catalog: &mut AliasCatalog,
    old_group: &str,
    new_group: &str,
) -> Result<Outcome, Failure> {
    if !catalog.groups.contains_key(old_group) {
        error!("Group {} does not exists.", old_group);
        return Err(Failure::GroupDoesNotExist);
    }

    if catalog.groups.contains_key(new_group) {
        error!("Group {} already exists.", new_group);
        return Err(Failure::GroupAlreadyExists);
    }

    let enabled = catalog
        .groups
        .shift_remove(old_group)
        .expect("the group has been checked to exist already");

    catalog.groups.insert(new_group.into(), enabled);

    for alias in catalog.aliases.values_mut() {
        if alias.group == Some(old_group.into()) {
            alias.group = Some(new_group.into());
        }
    }

    Ok(Outcome::CatalogChanged)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use crate::catalog::types::Alias;
    use assert_matches::assert_matches;

    fn create_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("group".into(), true);
        catalog.groups.insert("other_group".into(), true);
        catalog.aliases.insert(
            "foo".into(),
            Alias::new("bar".into(), Some("group".into()), true, false),
        );
        catalog
            .aliases
            .insert("ll".into(), Alias::new("ls -la".into(), None, true, false));

        catalog
    }

    #[test]
    fn test_rename_alias_success() {
        let mut catalog = create_catalog();
        let result = rename_alias(&mut catalog, "foo", "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rename_alias_nonexistent() {
        let mut catalog = create_catalog();
        let result = rename_alias(&mut catalog, "nonexistent", "boo");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::AliasDoesNotExist);
    }

    #[test]
    fn test_rename_alias_to_existent() {
        let mut catalog = create_catalog();
        let result = rename_alias(&mut catalog, "foo", "ll");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::AliasAlreadyExists);
    }

    #[test]
    fn test_rename_group_success() {
        let mut catalog = create_catalog();
        let result = rename_group(&mut catalog, "group", "nonexistent");
        assert!(result.is_ok());
        assert_matches!(result.unwrap(), Outcome::CatalogChanged);
        assert_eq!(catalog.aliases["foo"].group, Some("nonexistent".into()));
    }

    #[test]
    fn test_rename_group_nonexistent() {
        let mut catalog = create_catalog();
        let result = rename_group(&mut catalog, "nonexistent", "boo");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::GroupDoesNotExist);
    }

    #[test]
    fn test_rename_group_to_existent() {
        let mut catalog = create_catalog();
        let result = rename_group(&mut catalog, "group", "other_group");
        assert!(result.is_err());
        assert_matches!(result.err().unwrap(), Failure::GroupAlreadyExists);
    }
}
