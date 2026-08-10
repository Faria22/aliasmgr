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
