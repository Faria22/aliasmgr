use super::{Failure, Outcome};
use crate::app::shell::ShellType;
use crate::catalog::types::AliasCatalog;
use log::error;

pub fn remove_alias(catalog: &mut AliasCatalog, name: &str) -> Result<Outcome, Failure> {
    match catalog.aliases.shift_remove(name) {
        Some(_) => Ok(Outcome::CatalogChanged),
        None => {
            error!("Alias '{}' does not exist", name);
            Err(Failure::AliasDoesNotExist)
        }
    }
}

pub fn remove_all(catalog: &mut AliasCatalog, _shell: &ShellType) -> Result<Outcome, Failure> {
    catalog.aliases.clear();
    catalog.groups.clear();
    Ok(Outcome::CatalogChanged)
}

pub fn remove_aliases(catalog: &mut AliasCatalog, names: &[String]) -> Result<Outcome, Failure> {
    for name in names {
        remove_alias(catalog, name)?;
    }
    Ok(Outcome::CatalogChanged)
}

pub fn remove_group(catalog: &mut AliasCatalog, name: &str) -> Result<Outcome, Failure> {
    match catalog.groups.shift_remove(name) {
        Some(_) => Ok(Outcome::CatalogChanged),
        None => {
            error!("Group '{}' does not exist", name);
            Err(Failure::GroupDoesNotExist)
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "foo".to_string(),
            Alias::new("bar".to_string(), None, true, false),
        );
        catalog.aliases.insert(
            "baz".to_string(),
            Alias::new("qux".to_string(), None, true, false),
        );
        catalog.groups.insert("dev".to_string(), true);

        catalog
    }

    #[test]
    fn test_remove_alias_success() {
        let mut catalog = sample_catalog();
        let result = remove_alias(&mut catalog, "foo");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(catalog.aliases.contains_key("baz"));
        assert!(!catalog.aliases.contains_key("foo"));
    }

    #[test]
    fn test_remove_alias_failure() {
        let mut catalog = sample_catalog();
        let result = remove_alias(&mut catalog, "nonexistent");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Failure::AliasDoesNotExist));
    }

    #[test]
    fn test_remove_group_success() {
        let mut catalog = sample_catalog();
        let result = remove_group(&mut catalog, "dev");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(!catalog.groups.contains_key("dev"));
    }

    #[test]
    fn test_remove_group_failure() {
        let mut catalog = sample_catalog();
        let result = remove_group(&mut catalog, "nonexistent");
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_remove_all() {
        let mut catalog = sample_catalog();
        let result = remove_all(&mut catalog, &ShellType::Bash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(catalog.aliases.is_empty());
        assert!(catalog.groups.is_empty());
    }

    #[test]
    fn test_remove_aliases() {
        let mut catalog = sample_catalog();
        let names = vec!["foo".to_string(), "baz".to_string()];
        let result = remove_aliases(&mut catalog, &names);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Outcome::CatalogChanged);
        assert!(catalog.aliases.is_empty());
        assert!(catalog.groups.contains_key("dev"));
    }
}
