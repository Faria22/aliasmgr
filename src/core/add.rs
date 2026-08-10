use log::info;

use super::{Failure, Outcome};
use crate::catalog::types::{Alias, AliasCatalog};

pub fn add_alias(
    catalog: &mut AliasCatalog,
    name: &str,
    alias: &Alias,
) -> Result<Outcome, Failure> {
    if catalog.aliases.contains_key(name) {
        info!("Alias '{name}' already exists.");
        return Err(Failure::AliasAlreadyExists);
    }
    catalog.aliases.insert(name.into(), alias.clone());
    Ok(Outcome::CatalogChanged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_alias_and_rejects_duplicates() {
        let mut catalog = AliasCatalog::new();
        let alias = Alias::new("ls -la".into(), true, false);
        assert_eq!(
            add_alias(&mut catalog, "ll", &alias),
            Ok(Outcome::CatalogChanged)
        );
        assert_eq!(
            add_alias(&mut catalog, "ll", &alias),
            Err(Failure::AliasAlreadyExists)
        );
    }
}
