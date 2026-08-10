//! Catalog module for managing command aliases and groups.
//! This module provides functionality to load, save, and manipulate
//! alias catalogs, including serialization and deserialization
//! using the TOML format.
//!
//! # Modules
//! - `io`: Functions for loading and saving catalog files.
//! - `spec`: Specification structures and conversion functions for alias catalog.
//! - `types`: Core data structures representing aliases and catalogs.

pub(crate) mod io;
pub(crate) mod spec;
pub(crate) mod types;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) mod tests {
    use crate::catalog::types::{Alias, AliasCatalog};
    use std::collections::BTreeMap;

    pub const SAMPLE_TOML: &str = {
        r#"js = { command = "node", enabled = false, global = false }
py = "python3"
x = { command = "xargs", enabled = true, global = true }

[foo]
enabled = false
bar = "echo 'Hello World'"
ll = { command = "ls -la", enabled = true, global = false }

[git]
ga = "git add"
gc = { command = "git commit", enabled = true, global = false }
"#
    };

    pub fn expected_catalog() -> AliasCatalog {
        let mut aliases = BTreeMap::new();
        let mut groups = BTreeMap::new();
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

        AliasCatalog { aliases, groups }
    }
}
