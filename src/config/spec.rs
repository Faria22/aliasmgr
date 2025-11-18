use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{Alias, Config};

fn default_enabled() -> bool {
    true
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct AliasSpec {
    pub command: String,

    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupSpec {
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    #[serde(flatten)]
    pub aliases: HashMap<String, AliasSpecTypes>,
}

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

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigSpec {
    #[serde(flatten)]
    pub entries: HashMap<String, AliasSpecTypes>,
}

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

fn convert_group_to_spec(
    group_name: &str,
    enabled: bool,
    aliases: &HashMap<String, Alias>,
) -> AliasSpecTypes {
    let mut alias_specs = HashMap::new();
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

pub(crate) fn convert_config_to_spec(config: &Config) -> ConfigSpec {
    let mut entries = HashMap::new();

    for (name, alias) in &config.aliases {
        if let Some(group_name) = &alias.group {
            if config.groups.contains_key(group_name) {
                continue;
            }
        }
        entries.insert(name.clone(), convert_alias_to_spec(alias));
    }

    for (group_name, group) in &config.groups {
        entries.insert(
            group_name.clone(),
            convert_group_to_spec(group_name, *group, &config.aliases),
        );
    }

    ConfigSpec { entries }
}

fn convert_spec_to_alias(spec: AliasSpecTypes, group: Option<String>) -> Alias {
    match spec {
        AliasSpecTypes::Simple(command) => Alias {
            command,
            enabled: true,
            group,
            detailed: false,
        },
        AliasSpecTypes::Detailed(alias_spec) => Alias {
            command: alias_spec.command,
            enabled: alias_spec.enabled,
            group,
            detailed: true,
        },
        AliasSpecTypes::Group(_) => panic!("nested groups are not supported"),
    }
}

pub(crate) fn convert_spec_to_config(spec: ConfigSpec) -> Config {
    let mut aliases = HashMap::new();
    let mut groups = HashMap::new();

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
