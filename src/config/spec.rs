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

/// Convert an Alias to its corresponding AliasSpecTypes representation.
///
/// # Arguments
/// * `alias` - A reference to the Alias to be converted.
///
/// # Returns
/// * An AliasSpecTypes representation of the given Alias.
fn convert_alias_to_spec(alias: &Alias) -> AliasSpecTypes {
    if !alias.detailed {
        AliasSpecTypes::Simple(alias.command.clone())
    } else {
        AliasSpecTypes::Detailed(AliasSpec {
            command: alias.command.clone(),
            enabled: alias.enabled,
        })
    }
}

/// Convert a group of aliases to its corresponding AliasSpecTypes representation.
///
/// # Arguments
/// * `group_name` - The name of the group.
/// * `enabled` - A boolean indicating if the group is enabled.
/// * `aliases` - A reference to the HashMap of all aliases.
///
/// # Returns
/// * An AliasSpecTypes representation of the group.
fn convert_group_to_spec(
    group_name: &str,
    enabled: bool,
    aliases: &IndexMap<String, Alias>,
) -> AliasSpecTypes {
    let mut alias_specs = IndexMap::new();
    for (name, alias) in aliases
        .iter()
        .filter(|(_, a)| a.group.as_deref() == Some(group_name))
    {
        alias_specs.insert(name.clone(), convert_alias_to_spec(alias));
    }

    AliasSpecTypes::Group(GroupSpec {
        enabled,
        aliases: alias_specs,
    })
}

/// Convert a Config to its corresponding ConfigSpec representation.
///
/// # Arguments
/// * `config` - A reference to the Config to be converted.
///
/// # Returns
/// * A ConfigSpec representation of the given Config.
pub fn convert_config_to_spec(config: &Config) -> ConfigSpec {
    let mut entries = IndexMap::new();

    // First, add all aliases that are not part of any group.
    for (name, alias) in &config.aliases {
        if let Some(group_name) = &alias.group
            && config.groups.contains_key(group_name)
        {
            continue;
        }
        entries.insert(name.clone(), convert_alias_to_spec(alias));
    }

    // Then, add all groups.
    for (group_name, group) in &config.groups {
        entries.insert(
            group_name.clone(),
            convert_group_to_spec(group_name, *group, &config.aliases),
        );
    }

    ConfigSpec { entries }
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
