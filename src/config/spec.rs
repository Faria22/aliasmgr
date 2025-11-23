//! Specification structures and conversion functions for alias configuration.
//! This module defines the structures used for serializing and deserializing
//! alias configurations, as well as functions to convert between the internal
//! representation and the specification representation.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::types::{Alias, Config};

fn default_enabled() -> bool {
    true
}

/// Specification structures for serialization/deserialization of alias.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct AliasSpec {
    pub command: String,

    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Specification for a group of aliases.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupSpec {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(flatten)]
    pub aliases: IndexMap<String, AliasSpecTypes>,
}

/// Different types of alias specifications.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AliasSpecTypes {
    // foo = "bar"
    Simple(String),

    // foo = {command = "bar", enable = true}
    Detailed(AliasSpec),

    // [foo]
    // foo = "bar"
    // or
    // foo = {command = "bar", enable = true}
    Group(GroupSpec),
}

/// Overall configuration specification.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigSpec {
    #[serde(flatten)]
    pub entries: IndexMap<String, AliasSpecTypes>,
}

/// Convert an AliasSpecTypes to its corresponding Alias representation.
///
/// # Arguments
/// * `spec` - The AliasSpecTypes to be converted.
/// * `group` - An optional group name for the alias.
///
/// # Returns
/// * An Alias representation of the given AliasSpecTypes.
fn convert_spec_to_alias(spec: AliasSpecTypes, group: Option<String>) -> Alias {
    match spec {
        AliasSpecTypes::Simple(command) => Alias::new(command, true, group, false),
        AliasSpecTypes::Detailed(alias_spec) => {
            Alias::new(alias_spec.command, alias_spec.enabled, group, true)
        }
        AliasSpecTypes::Group(_) => panic!("nested groups are not supported"),
    }
}

/// Convert a ConfigSpec to its corresponding Config representation.
///
/// # Arguments
/// * `spec` - The ConfigSpec to be converted.
///
/// # Returns
/// * A Config representation of the given ConfigSpec.
pub fn convert_spec_to_config(spec: ConfigSpec) -> Config {
    let mut aliases = IndexMap::new();
    let mut groups = IndexMap::new();

    for (name, entry) in spec.entries {
        match entry {
            AliasSpecTypes::Group(group_spec) => {
                groups.insert(name.clone(), group_spec.enabled);

                for (alias_name, alias_entry) in group_spec.aliases {
                    let alias = convert_spec_to_alias(alias_entry, Some(name.clone()));
                    aliases.insert(alias_name, alias);
                }
            }
            alias => {
                let alias_cfg = convert_spec_to_alias(alias, None);
                aliases.insert(name, alias_cfg);
            }
        }
    }

    Config { aliases, groups }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tests::{SAMPLE_TOML, expected_config};

    #[test]
    fn test_convert_spec_to_config() {
        let spec: ConfigSpec = toml::from_str(SAMPLE_TOML).unwrap();
        let config = convert_spec_to_config(spec);
        assert_eq!(config, expected_config());
    }

    #[test]
    #[should_panic = "nested groups are not supported"]
    fn test_nested_group_handling() {
        let toml_data = r#"
        [group1]
        enabled = true
        alias1 = "command1"

        [group1.subgroup]
        enabled = false
        alias2 = "command2"
        "#;

        let spec: ConfigSpec = toml::from_str(toml_data).unwrap();
        convert_spec_to_config(spec);
    }
}
