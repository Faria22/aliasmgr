use super::{Failure, Outcome};
use crate::catalog::types::{Alias, AliasCatalog};

pub fn edit_alias(
    catalog: &mut AliasCatalog,
    name: &str,
    alias: &Alias,
) -> Result<Outcome, Failure> {
    let Some(current) = catalog.aliases.get_mut(name) else {
        return Err(Failure::AliasDoesNotExist);
    };
    if current == alias {
        return Ok(Outcome::NoChanges);
    }
    *current = alias.clone();
    Ok(Outcome::CatalogChanged)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn missing_unchanged_and_changed_aliases_have_distinct_outcomes() {
        let mut catalog = AliasCatalog::new();
        let original = Alias::new("old".into(), true, false);
        assert_eq!(
            edit_alias(&mut catalog, "missing", &original),
            Err(Failure::AliasDoesNotExist)
        );
        catalog.aliases.insert("name".into(), original.clone());
        assert_eq!(
            edit_alias(&mut catalog, "name", &original),
            Ok(Outcome::NoChanges)
        );
        let changed = Alias::new("new".into(), true, false);
        assert_eq!(
            edit_alias(&mut catalog, "name", &changed),
            Ok(Outcome::CatalogChanged)
        );
    }
}
