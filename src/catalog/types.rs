//! Catalog types for command aliases.

use std::collections::{BTreeMap, BTreeSet};

/// Representation of an alias in the catalog.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Alias {
    pub command: String,
    pub enabled: bool,
    pub global: bool,
    pub description: Option<String>,
    pub tags: BTreeSet<String>,
    // Keeps track of whether the alias uses detailed representation.
    pub detailed: bool,
}

impl Alias {
    pub fn new(command: String, enabled: bool, global: bool) -> Self {
        Self {
            command,
            enabled,
            global,
            description: None,
            tags: BTreeSet::new(),
            detailed: !enabled || global,
        }
    }

    pub fn refresh_representation(&mut self) {
        self.detailed =
            !self.enabled || self.global || self.description.is_some() || !self.tags.is_empty();
    }
}

/// Overall catalog containing aliases in deterministic name order.
#[derive(PartialEq, Eq, Debug, Default)]
pub struct AliasCatalog {
    pub aliases: BTreeMap<String, Alias>,
}

impl AliasCatalog {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn metadata_requires_detailed_representation() {
        let mut alias = Alias::new("cmd".into(), true, false);
        assert!(!alias.detailed);

        alias.tags.insert("dev".into());
        alias.refresh_representation();
        assert!(alias.detailed);
    }
}
