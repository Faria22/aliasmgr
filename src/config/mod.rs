//! Configuration module for managing command aliases and groups.
//! This module provides functionality to load, save, and manipulate
//! alias configurations, including serialization and deserialization
//! using the TOML format.
//!
//! # Modules
//! - `io`: Functions for loading and saving configuration files.
//! - `spec`: Specification structures and conversion functions for alias configuration.
//! - `types`: Core data structures representing aliases and configurations.

pub(crate) mod io;
pub(crate) mod spec;
pub(crate) mod types;

#[cfg(test)]
pub(crate) mod tests {
    use crate::config::types::{Alias, Config};
    use indexmap::IndexMap;

    pub const SAMPLE_TOML: &str = {
        r#"py = "python3"
js = { command = "node", enabled = false, global = false }
x = { command = "xargs", enabled = true, global = true }

[git]
ga = "git add"
gc = { command = "git commit", enabled = true, global = false }

[foo]
enabled = false
bar = "echo 'Hello World'"
ll = { command = "ls -la", enabled = true, global = false }
"#
    };

    pub fn expected_config() -> Config {
        let mut aliases = IndexMap::new();
        let mut groups = IndexMap::new();
        aliases.insert("py".into(), Alias::new("python3".into(), None, true, false));
        aliases.insert("js".into(), Alias::new("node".into(), None, false, false));
        aliases.insert("x".into(), Alias::new("xargs".into(), None, true, true));

        aliases.insert(
            "ga".into(),
            Alias::new("git add".into(), Some("git".into()), true, false),
        );

        let mut alias = Alias::new("git commit".into(), Some("git".into()), true, false);
        alias.detailed = true;
        aliases.insert("gc".into(), alias);

        aliases.insert(
            "bar".into(),
            Alias::new("echo 'Hello World'".into(), Some("foo".into()), true, false),
        );

        let mut alias = Alias::new("ls -la".into(), Some("foo".into()), true, false);
        alias.detailed = true;
        aliases.insert("ll".into(), alias);

        groups.insert("git".into(), true);
        groups.insert("foo".into(), false);

        Config { aliases, groups }
    }
}
