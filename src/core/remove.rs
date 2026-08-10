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
