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
mod tests {
    use crate::config::io::{load_config, save_config};
    use crate::config::spec::{ConfigSpec, convert_spec_to_config};
    use crate::config::types::{Alias, Config};
    use assert_fs::TempDir;
    use indexmap::IndexMap;
    use std::fs;

    fn sample_toml() -> &'static str {
        r#"
        py = "python3"
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

    fn expected_config() -> Config {
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

    #[test]
    fn test_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");
        fs::write(&temp_conf, sample_toml()).unwrap();

        let cfg = load_config(Some(&temp_conf)).unwrap();
        assert_eq!(cfg, expected_config());
    }

    #[test]
    fn test_save_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");

        let config = expected_config();
        save_config(&config, Some(&temp_conf)).unwrap();

        let saved_content = fs::read_to_string(&temp_conf).unwrap();
        let parsed_spec: ConfigSpec = toml::from_str(&saved_content).unwrap();
        let parsed_config = convert_spec_to_config(parsed_spec);
        assert_eq!(parsed_config, config);
    }

    #[test]
    fn test_convert_spec_to_config() {
        let spec: ConfigSpec = toml::from_str(sample_toml()).unwrap();
        let config = convert_spec_to_config(spec);
        assert_eq!(config, expected_config());
    }
}
