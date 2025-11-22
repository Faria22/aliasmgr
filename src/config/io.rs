//! This module provides functions to load and save the configuration for the alias manager.
//! ! It supports loading from and saving to a specified path or the default XDG configuration path.
//! ! The configuration is serialized and deserialized using the TOML format.
//! ! It also handles the creation of necessary directories if they do not exist.

use log::{debug, info, warn};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use indexmap::IndexMap;
use toml_edit::{DocumentMut, InlineTable, Item, Table};

use super::spec::{ConfigSpec, convert_spec_to_config};
use super::types::{Alias, Config};

/// Determine the configuration file path.
/// If a custom path is provided, it is used; otherwise, the default XDG config path is used.
///
/// # Arguments
/// `path` - An optional custom path to the configuration file.
///
/// # Returns
/// A `PathBuf` representing the configuration file path.
pub fn config_path(path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = path {
        info!("Using custom config path: {:?}", p);
        return p.clone();
    }

    cross_xdg::BaseDirs::new()
        .expect("could not determine XDG base directories")
        .config_home()
        .join("aliasmgr")
        .join("aliases.toml")
}

/// Load the configuration from the specified path or the default XDG config path.
/// If the file does not exist, an empty configuration is returned.
///
/// # Arguments
/// `path` - An optional custom path to the configuration file.
///
/// # Returns
/// A `Result` containing the loaded `Config` or an error.
pub fn load_config(path: Option<&PathBuf>) -> Result<Config> {
    let path = config_path(path);
    info!("Loading config from {:?}", path);

    if !path.exists() {
        info!("Config file {:?} does not exist, using empty config", path);
        return Ok(Config::new());
    }

    let content = fs::read_to_string(path)?;
    let cfg: ConfigSpec = toml::from_str(&content)?;
    Ok(convert_spec_to_config(cfg))
}

fn ensure_group_table<'a>(doc: &'a mut DocumentMut, name: &str) -> &'a mut Table {
    if !doc.contains_key(name) {
        doc[name] = Item::Table(Table::new());
    }

    let table = doc[name]
        .as_table_mut()
        .expect("group entry should be a table");
    table.set_implicit(false); // render as [group], not dotted
    table
}

fn build_alias_item(alias: &Alias) -> Item {
    if !alias.detailed && alias.enabled {
        Item::Value(alias.command.clone().into())
    } else {
        let mut inline = InlineTable::new();
        inline.insert("command", alias.command.clone().into());
        inline.insert("enabled", alias.enabled.into());
        inline.set_dotted(false);
        Item::Value(inline.into())
    }
}

fn insert_groups(doc: &mut DocumentMut, groups: &IndexMap<String, bool>) {
    for (group_name, enabled) in groups {
        let table = ensure_group_table(doc, group_name);
        if !*enabled {
            table["enabled"] = Item::Value((*enabled).into());
        }
    }
}

fn insert_aliases(
    doc: &mut DocumentMut,
    aliases: &IndexMap<String, Alias>,
    groups: &IndexMap<String, bool>,
) {
    for (alias_name, alias) in aliases {
        if let Some(group) = &alias.group {
            if !groups.contains_key(group) {
                warn!(
                    "Alias '{}' references unknown group '{}'",
                    alias_name, group
                );
            }
            let table = ensure_group_table(doc, group);
            table[alias_name] = build_alias_item(alias);
        } else {
            doc[alias_name] = build_alias_item(alias);
        }
    }
}

fn build_toml_document(config: &Config) -> DocumentMut {
    let mut doc = DocumentMut::new();
    insert_groups(&mut doc, &config.groups);
    insert_aliases(&mut doc, &config.aliases, &config.groups);
    doc
}

/// Save the configuration to the specified path or the default XDG config path.
/// If the file does not exist, it will be created along with any necessary parent directories.
///
/// # Arguments
/// `path` - An optional custom path to the configuration file.
/// `config` - A reference to the `Config` to be saved.
///
/// # Returns
/// A `Result` indicating success or failure.
pub fn save_config(config: &Config, custom_path: Option<&PathBuf>) -> Result<()> {
    let path = config_path(custom_path);

    if !path.exists() {
        warn!("Config file {:?} does not exist, creating it", path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        debug!("Overwriting existing config at {:?}", path);
    } else {
        debug!("Saving content into new config at {:?}", path);
    }

    let doc = build_toml_document(config);
    let content = doc.to_string();
    fs::write(path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tests::{expected_config, sample_toml};
    use assert_fs::TempDir;

    #[test]
    fn build_alias_item_simple_enabled_is_string() {
        let alias = Alias::new("cmd".into(), true, None, false);
        let item = build_alias_item(&alias);
        assert!(item.as_value().unwrap().as_str().is_some());
        assert_eq!(item.to_string(), "\"cmd\"");
    }

    #[test]
    fn build_alias_item_detailed_is_inline_table() {
        let alias = Alias::new("cmd".into(), true, None, true);
        let item = build_alias_item(&alias);
        let inline = item
            .as_value()
            .and_then(|v| v.as_inline_table())
            .expect("expected inline table");
        assert_eq!(inline.get("command").unwrap().as_str(), Some("cmd"));
        assert_eq!(inline.get("enabled").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn ensure_group_table_is_explicit() {
        let mut doc = DocumentMut::new();
        let table = ensure_group_table(&mut doc, "group");
        assert!(!table.is_implicit());
        assert!(doc["group"].as_table().is_some());
    }

    #[test]
    fn build_document_inlines_group_aliases() {
        let mut config = Config::new();
        config.groups.insert("group".into(), true);
        config.aliases.insert(
            "alias".into(),
            Alias::new("foo".into(), false, Some("group".into()), true),
        );

        let doc = build_toml_document(&config);
        let rendered = doc.to_string();

        assert!(rendered.contains("[group]"));
        assert!(rendered.contains("alias = { command = \"foo\", enabled = false }"));
        assert!(!rendered.contains("[group.alias]"));
    }

    #[test]
    fn build_document_writes_top_level_simple_alias() {
        let mut config = Config::new();
        config
            .aliases
            .insert("ls".into(), Alias::new("ls -la".into(), true, None, false));

        let doc = build_toml_document(&config);
        let rendered = doc.to_string();
        assert!(rendered.contains("ls = \"ls -la\""));
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
        assert_eq!(saved_content, sample_toml().replace("        ", ""));
    }

    #[test]
    fn test_config_path_custom() {
        let custom_path = PathBuf::from("/custom/path/aliases.toml");
        let path = config_path(Some(&custom_path));
        assert_eq!(path, custom_path);
    }

    #[test]
    fn test_config_path_default() {
        let path = config_path(None);
        assert!(path.ends_with(".config/aliasmgr/aliases.toml"));
    }

    #[test]
    fn test_load_config_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("nonexistent.toml");
        let cfg = load_config(Some(&temp_conf)).unwrap();
        assert_eq!(cfg, Config::new());
    }

    #[test]
    fn test_build_alias_item_disabled_detailed() {
        let alias = Alias::new("cmd".into(), false, None, true);
        let item = build_alias_item(&alias);
        let inline = item
            .as_value()
            .and_then(|v| v.as_inline_table())
            .expect("expected inline table");
        assert_eq!(inline.get("command").unwrap().as_str(), Some("cmd"));
        assert_eq!(inline.get("enabled").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_insert_alias_to_unknown_group() {
        let mut doc = DocumentMut::new();
        let mut config = Config::new();
        config.aliases.insert(
            "alias".into(),
            Alias::new("foo".into(), true, Some("unknown_group".into()), false),
        );

        insert_aliases(&mut doc, &config.aliases, &config.groups);
        let rendered = doc.to_string();
        assert!(rendered.contains("alias = \"foo\""));
    }

    #[test]
    fn test_save_config_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested/dir/aliases.toml");
        let config = expected_config();
        save_config(&config, Some(&nested_path)).unwrap();
        assert!(nested_path.exists());
    }

    #[test]
    fn test_save_config_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("new_aliases.toml");
        let config = expected_config();
        save_config(&config, Some(&temp_conf)).unwrap();
        assert!(temp_conf.exists());
    }

    #[test]
    fn test_save_config_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");
        fs::write(&temp_conf, "old_content").unwrap();
        let config = expected_config();
        save_config(&config, Some(&temp_conf)).unwrap();
        let saved_content = fs::read_to_string(&temp_conf).unwrap();
        assert_ne!(saved_content, "old_content");
    }

    #[test]
    fn test_build_alias_item_disabled_simple() {
        let alias = Alias::new("cmd".into(), false, None, true);
        let item = build_alias_item(&alias);
        let inline = item
            .as_value()
            .and_then(|v| v.as_inline_table())
            .expect("expected inline table");
        assert_eq!(inline.get("command").unwrap().as_str(), Some("cmd"));
        assert_eq!(inline.get("enabled").unwrap().as_bool(), Some(false));
    }
}
