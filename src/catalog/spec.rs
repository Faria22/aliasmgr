//! Serializable catalog specification.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::types::{Alias, AliasCatalog};

fn enabled_by_default() -> bool {
    true
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct AliasSpec {
    pub command: String,

    #[serde(default = "enabled_by_default")]
    pub enabled: bool,

    #[serde(default)]
    pub global: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub tags: BTreeSet<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AliasSpecTypes {
    Simple(String),
    Detailed(AliasSpec),
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct AliasCatalogSpec {
    #[serde(flatten)]
    pub aliases: BTreeMap<String, AliasSpecTypes>,
}

pub fn convert_spec_to_catalog(spec: AliasCatalogSpec) -> AliasCatalog {
    let aliases = spec
        .aliases
        .into_iter()
        .map(|(name, spec)| {
            let alias = match spec {
                AliasSpecTypes::Simple(command) => Alias::new(command, true, false),
                AliasSpecTypes::Detailed(spec) => Alias {
                    command: spec.command,
                    enabled: spec.enabled,
                    global: spec.global,
                    description: spec.description,
                    tags: spec.tags,
                    detailed: true,
                },
            };
            (name, alias)
        })
        .collect();
    AliasCatalog { aliases }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn converts_mixed_simple_and_detailed_aliases() {
        let spec: AliasCatalogSpec = toml::from_str(
            r#"
            ll = "ls -la"
            test = { command = "cargo test", description = "Run tests", tags = ["dev", "rust"] }
            "#,
        )
        .unwrap();
        let catalog = convert_spec_to_catalog(spec);

        assert!(!catalog.aliases["ll"].detailed);
        assert_eq!(
            catalog.aliases["test"].description.as_deref(),
            Some("Run tests")
        );
        assert_eq!(
            catalog.aliases["test"]
                .tags
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["dev", "rust"]
        );
    }
}
