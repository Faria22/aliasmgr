//! Configuration types for command aliases.
//! ! This module defines the structures used to represent command aliases and their configurations.

use indexmap::IndexMap;

/// Representation of an alias in the configuration.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Alias {
    pub command: String,
    pub group: Option<String>,
    pub enabled: bool,
    // Keeps track of whether the alias uses detailed representation.
    pub detailed: bool,
    pub global: bool,
}

/// Constructor for Alias with validation.
impl Alias {
    pub fn new(command: String, group: Option<String>, enabled: bool, global: bool) -> Self {
        Alias {
            command,
            enabled,
            group,
            detailed: !enabled || global,
            global,
        }
    }
}

/// Overall configuration containing aliases and groups.
#[derive(PartialEq, Eq, Debug)]
pub struct Config {
    pub aliases: IndexMap<String, Alias>,
    pub groups: IndexMap<String, bool>,
}

/// Constructor for Config.
impl Config {
    pub fn new() -> Self {
        Config {
            aliases: IndexMap::new(),
            groups: IndexMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_non_global_alias_must_not_be_detailed() {
        let alias = Alias::new("cmd".into(), None, true, false);
        assert_eq!(
            alias,
            Alias {
                command: "cmd".into(),
                enabled: true,
                group: None,
                detailed: false,
                global: false,
            }
        );
    }

    #[test]
    fn disabled_alias_must_be_detailed() {
        let alias = Alias::new("cmd".into(), None, false, false);
        assert_eq!(
            alias,
            Alias {
                command: "cmd".into(),
                enabled: false,
                group: None,
                detailed: true,
                global: false,
            }
        );
    }

    #[test]
    fn global_alias_must_be_detailed() {
        let alias = Alias::new("cmd".into(), None, false, true);
        assert_eq!(
            alias,
            Alias {
                command: "cmd".into(),
                enabled: true,
                group: None,
                detailed: true,
                global: true,
            }
        );
    }

    #[test]
    fn disabled_global_alias_must_be_detailed() {
        let alias = Alias::new("cmd".into(), None, false, true);
        assert_eq!(
            alias,
            Alias {
                command: "cmd".into(),
                enabled: false,
                group: None,
                detailed: true,
                global: true,
            }
        )
    }
}
