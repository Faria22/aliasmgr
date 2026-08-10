use super::{Failure, Outcome};
use crate::catalog::types::AliasCatalog;

pub fn rename_alias(catalog: &mut AliasCatalog, old: &str, new: &str) -> Result<Outcome, Failure> {
    if !catalog.aliases.contains_key(old) {
        return Err(Failure::AliasDoesNotExist);
    }
    if catalog.aliases.contains_key(new) {
        return Err(Failure::AliasAlreadyExists);
    }
    let alias = catalog
        .aliases
        .remove(old)
        .expect("alias existence checked");
    catalog.aliases.insert(new.into(), alias);
    Ok(Outcome::CatalogChanged)
}

pub fn rename_tag(
    catalog: &mut AliasCatalog,
    old: &str,
    new: &str,
) -> Result<(Outcome, usize), Failure> {
    if old == new {
        let matched = catalog
            .aliases
            .values()
            .filter(|alias| alias.tags.contains(old))
            .count();
        return if matched == 0 {
            Err(Failure::TagDoesNotExist)
        } else {
            Ok((Outcome::NoChanges, matched))
        };
    }
    let mut changed = 0;
    for alias in catalog.aliases.values_mut() {
        if alias.tags.remove(old) {
            alias.tags.insert(new.into());
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
