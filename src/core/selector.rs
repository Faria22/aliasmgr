use globset::Glob;
use log::error;

use super::Failure;
use crate::catalog::types::AliasCatalog;

pub fn select_aliases(
    catalog: &AliasCatalog,
    pattern: Option<&str>,
    tags: &[String],
) -> Result<Vec<String>, Failure> {
    let matcher = pattern
        .map(|pattern| {
            Glob::new(pattern)
                .map(|glob| glob.compile_matcher())
                .map_err(|error| {
                    error!("Invalid glob pattern '{pattern}': {error}");
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
        .filter(|(_, alias)| tags.iter().all(|tag| alias.tags.contains(tag)))
        .map(|(name, _)| name.clone())
        .collect())
}

pub fn aliases_with_tag(catalog: &AliasCatalog, tag: &str) -> Result<Vec<String>, Failure> {
    let names = select_aliases(catalog, None, &[tag.to_owned()])?;
    if names.is_empty()
        && !catalog
            .aliases
            .values()
            .any(|alias| alias.tags.contains(tag))
    {
        return Err(Failure::TagDoesNotExist);
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    #[test]
    fn tag_filters_use_and_semantics() {
        let mut catalog = AliasCatalog::new();
        let mut both = Alias::new("cmd".into(), true, false);
        both.tags.extend(["dev".into(), "rust".into()]);
        catalog.aliases.insert("both".into(), both);
        let mut one = Alias::new("cmd".into(), true, false);
        one.tags.insert("dev".into());
        catalog.aliases.insert("one".into(), one);

        assert_eq!(
            select_aliases(&catalog, None, &["dev".into(), "rust".into()]).unwrap(),
            ["both"]
        );
    }
}
