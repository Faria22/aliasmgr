use super::{Failure, Outcome};
use crate::catalog::types::AliasCatalog;

pub fn remove_alias(catalog: &mut AliasCatalog, name: &str) -> Result<Outcome, Failure> {
    catalog
        .aliases
        .remove(name)
        .map(|_| Outcome::CatalogChanged)
        .ok_or(Failure::AliasDoesNotExist)
}

pub fn remove_aliases(catalog: &mut AliasCatalog, names: &[String]) -> Outcome {
    for name in names {
        catalog.aliases.remove(name);
    }
    if names.is_empty() {
        Outcome::NoChanges
    } else {
        Outcome::CatalogChanged
    }
}

pub fn remove_all(catalog: &mut AliasCatalog) -> Outcome {
    if catalog.aliases.is_empty() {
        Outcome::NoChanges
    } else {
        catalog.aliases.clear();
        Outcome::CatalogChanged
    }
}

pub fn remove_tag(catalog: &mut AliasCatalog, tag: &str) -> Result<(Outcome, usize), Failure> {
    let mut changed = 0;
    for alias in catalog.aliases.values_mut() {
        if alias.tags.remove(tag) {
            alias.refresh_representation();
            changed += 1;
        }
    }
    if changed == 0 {
        Err(Failure::TagDoesNotExist)
    } else {
        Ok((Outcome::CatalogChanged, changed))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    #[test]
    fn empty_bulk_and_all_removals_are_noops() {
        let mut catalog = AliasCatalog::new();
        assert_eq!(remove_aliases(&mut catalog, &[]), Outcome::NoChanges);
        assert_eq!(remove_all(&mut catalog), Outcome::NoChanges);
    }

    #[test]
    fn nonempty_bulk_and_all_removals_change_the_catalog() {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("one".into(), Alias::new("cmd".into(), true, false));
        assert_eq!(
            remove_aliases(&mut catalog, &["one".into()]),
            Outcome::CatalogChanged
        );
        catalog
            .aliases
            .insert("two".into(), Alias::new("cmd".into(), true, false));
        assert_eq!(remove_all(&mut catalog), Outcome::CatalogChanged);
        assert!(catalog.aliases.is_empty());
    }
}
