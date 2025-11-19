//! Configuration types for command aliases.
//! ! This module defines the structures used to represent command aliases and their configurations.

use std::collections::HashMap;

/// Representation of an alias in the configuration.
#[derive(PartialEq, Eq, Debug)]
pub struct Alias {
    pub command: String,
    pub enabled: bool,
    pub group: Option<String>,
    pub detailed: bool,
}

/// Constructor for Alias with validation.
impl Alias {
    pub fn new(command: String, enabled: bool, group: Option<String>, detailed: bool) -> Self {
        if !enabled && !detailed {
            panic!("disabled aliases must use detailed representation");
        }

        Alias {
            command,
            enabled,
            group,
            detailed,
        }
    }
}

/// Overall configuration containing aliases and groups.
#[derive(PartialEq, Eq, Debug)]
pub struct Config {
    pub aliases: HashMap<String, Alias>,
    pub groups: HashMap<String, bool>,
}

/// Constructor for Config.
impl Config {
    pub fn new() -> Self {
        Config {
            aliases: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "disabled aliases must use detailed representation")]
    fn disabled_alias_must_be_detailed() {
        Alias::new("cmd".into(), false, None, false);
    }

    #[test]
    fn enabled_alias_may_be_simple() {
        let alias = Alias::new("cmd".into(), true, None, false);
        assert_eq!(
            alias,
            Alias {
                command: "cmd".into(),
                enabled: true,
                group: None,
                detailed: false,
            }
        );
    }

    #[test]
    fn enabled_alias_may_be_detailed() {
        let alias = Alias::new("cmd".into(), true, None, true);
        assert_eq!(
            alias,
            Alias {
                command: "cmd".into(),
                enabled: true,
                group: None,
                detailed: true,
            }
        );
    }
}
