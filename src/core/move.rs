use super::{Failure, Outcome};
use crate::catalog::types::AliasCatalog;

pub fn move_alias(
    catalog: &mut AliasCatalog,
    alias: &str,
    new_group: &Option<String>,
) -> Result<Outcome, Failure> {
    // Checks if alias exists before moving forward
    if !catalog.aliases.contains_key(alias) {
        return Err(Failure::AliasDoesNotExist);
    }

    // If moving to a specific group, check if the group exists first
    if let Some(group) = new_group
        && !catalog.groups.contains_key(group)
    {
        return Err(Failure::GroupDoesNotExist);
    }

    catalog.aliases.get_mut(alias).unwrap().group = new_group.clone();
    Ok(Outcome::CatalogChanged)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use super::*;
    use crate::catalog::types::Alias;
    use assert_matches::assert_matches;

    fn sample_alias() -> Alias {
        Alias::new("ls -la".into(), None, true, false)
    }

    static SAMPLE_ALIAS_NAME: &str = "ll";

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert(SAMPLE_ALIAS_NAME.into(), sample_alias());
        catalog
    }

    #[test]
    fn move_alias_to_existing_group() {
        let mut catalog = sample_catalog();
        catalog.groups.insert("utilities".into(), true);
        let result = move_alias(&mut catalog, SAMPLE_ALIAS_NAME, &Some("utilities".into()));

        let mut new_alias = sample_alias();
        new_alias.group = Some("utilities".into());

        assert!(result.is_ok());
        assert_eq!(catalog.aliases.get("ll"), Some(&new_alias));
    }

    #[test]
    fn move_alias_to_non_existent_group() {
        let mut catalog = sample_catalog();
        let result = move_alias(&mut catalog, SAMPLE_ALIAS_NAME, &Some("nonexistent".into()));
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn move_non_existent_alias() {
        let mut catalog = AliasCatalog::new();
        let result = move_alias(&mut catalog, "nonexistent", &Some("utilities".into()));
        assert_matches!(result, Err(Failure::AliasDoesNotExist));
    }

    #[test]
    fn move_alias_to_none_group() {
        let mut catalog = AliasCatalog::new();
        let mut alias = sample_alias();
        alias.group = Some("utilities".into());
        catalog.aliases.insert(SAMPLE_ALIAS_NAME.into(), alias);

        let result = move_alias(&mut catalog, SAMPLE_ALIAS_NAME, &None);

        assert!(result.is_ok());
        assert_eq!(
            catalog.aliases.get(SAMPLE_ALIAS_NAME),
            Some(&sample_alias())
        );
    }
}
