use super::{Failure, Outcome};
use crate::catalog::types::AliasCatalog;

fn set_aliases(catalog: &mut AliasCatalog, names: &[String], enabled: bool) -> (Outcome, usize) {
    let mut changed = 0;
    for name in names {
        let alias = catalog
            .aliases
            .get_mut(name)
            .expect("selected alias exists");
        if alias.enabled != enabled {
            alias.enabled = enabled;
            alias.refresh_representation();
            changed += 1;
        }
    }
    (
        if changed == 0 {
            Outcome::NoChanges
        } else {
            Outcome::CatalogChanged
        },
        changed,
    )
}

pub fn disable_alias(catalog: &mut AliasCatalog, name: &str) -> Result<Outcome, Failure> {
    if !catalog.aliases.contains_key(name) {
        return Err(Failure::AliasDoesNotExist);
    }
    Ok(set_aliases(catalog, &[name.into()], false).0)
}

pub fn disable_aliases(catalog: &mut AliasCatalog, names: &[String]) -> (Outcome, usize) {
    set_aliases(catalog, names, false)
}

pub fn disable_all(catalog: &mut AliasCatalog) -> Outcome {
    let names = catalog.aliases.keys().cloned().collect::<Vec<_>>();
    set_aliases(catalog, &names, false).0
}
