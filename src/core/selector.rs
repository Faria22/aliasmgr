use globset::Glob;
use log::error;

use crate::catalog::types::AliasCatalog;

use super::Failure;

pub fn select_aliases(
    catalog: &AliasCatalog,
    pattern: Option<&str>,
    group: Option<Option<&str>>,
) -> Result<Vec<String>, Failure> {
    if let Some(Some(group)) = group
        && !catalog.groups.contains_key(group)
    {
        error!("Group '{}' does not exist", group);
        return Err(Failure::GroupDoesNotExist);
    }

    let matcher = pattern
        .map(|pattern| {
            Glob::new(pattern)
                .map(|glob| glob.compile_matcher())
                .map_err(|error| {
                    error!("Invalid glob pattern '{}': {}", pattern, error);
                    Failure::InvalidPattern
                })
        })
        .transpose()?;

    Ok(catalog
        .aliases
        .iter()
        .filter(|(name, _)| {
            matcher
                .as_ref()
                .is_none_or(|matcher| matcher.is_match(name))
        })
        .filter(|(_, alias)| group.is_none_or(|group| alias.group.as_deref() == group))
        .map(|(name, _)| name.clone())
        .collect())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("dev".into(), true);
        catalog.groups.insert("ops".into(), true);
        catalog.aliases.insert(
            "build".into(),
            Alias::new("cargo build".into(), Some("dev".into()), true, false),
        );
        catalog.aliases.insert(
            "bench".into(),
            Alias::new("cargo bench".into(), Some("dev".into()), true, false),
        );
        catalog.aliases.insert(
            "deploy".into(),
            Alias::new("deploy".into(), Some("ops".into()), true, false),
        );
        catalog.aliases.insert(
            "local".into(),
            Alias::new("echo local".into(), None, true, false),
        );
        catalog
    }

    #[test]
    fn combines_pattern_and_group_filters() {
        let selected = select_aliases(&sample_catalog(), Some("b*"), Some(Some("dev"))).unwrap();
        assert_eq!(selected, ["bench", "build"]);
    }

    #[test]
    fn selects_ungrouped_aliases() {
        let selected = select_aliases(&sample_catalog(), None, Some(None)).unwrap();
        assert_eq!(selected, ["local"]);
    }

    #[test]
    fn rejects_missing_groups_and_invalid_patterns() {
        assert_eq!(
            select_aliases(&sample_catalog(), None, Some(Some("missing"))),
            Err(Failure::GroupDoesNotExist)
        );
        assert_eq!(
            select_aliases(&sample_catalog(), Some("["), None),
            Err(Failure::InvalidPattern)
        );
    }
}
