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

    pub fn sample_toml() -> &'static str {
        r#"py = "python3"
        js = { command = "node", enabled = false }

        [git]
        ga = "git add"
        gc = { command = "git commit", enabled = true }

        [foo]
        enabled = false
        bar = "echo 'Hello World'"
        ll = { command = "ls -la", enabled = true }
        "#
    }

    pub fn expected_config() -> Config {
        let mut aliases = IndexMap::new();
        let mut groups = IndexMap::new();
        aliases.insert("py".into(), Alias::new("python3".into(), true, None, false));

        aliases.insert("js".into(), Alias::new("node".into(), false, None, true));

        aliases.insert(
            "ga".into(),
            Alias::new("git add".into(), true, Some("git".into()), false),
        );

        aliases.insert(
            "gc".into(),
            Alias::new("git commit".into(), true, Some("git".into()), true),
        );

        aliases.insert(
            "bar".into(),
            Alias::new("echo 'Hello World'".into(), true, Some("foo".into()), false),
        );

        aliases.insert(
            "ll".into(),
            Alias::new("ls -la".into(), true, Some("foo".into()), true),
        );

        groups.insert("git".into(), true);
        groups.insert("foo".into(), false);

        Config { aliases, groups }
    }
}
