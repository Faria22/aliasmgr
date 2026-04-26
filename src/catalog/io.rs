//! This module provides functions to load and save the catalog for the alias manager.
//! ! It supports loading from and saving to a specified path or the default XDG catalog path.
//! ! The catalog is serialized and deserialized using the TOML format.
//! ! It also handles the creation of necessary directories if they do not exist.

use log::{debug, info, warn};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use indexmap::IndexMap;
use toml_edit::{DocumentMut, InlineTable, Item, Table};

use super::spec::{AliasCatalogSpec, convert_spec_to_catalog};
use super::types::{Alias, AliasCatalog};

/// Determine the catalog file path.
/// If a custom path is provided, it is used; otherwise, the default XDG catalog path is used.
///
/// # Arguments
/// `path` - An optional custom path to the catalog file.
///
/// # Returns
/// A `PathBuf` representing the catalog file path.
pub fn catalog_path(path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = path {
        info!("Using custom catalog path: {:?}", p);
        return p.clone();
    }

    cross_xdg::BaseDirs::new()
        .expect("could not determine XDG base directories")
        .config_home()
        .join("aliasmgr")
        .join("aliases.toml")
}

/// Determine the last synced catalog file path.
/// If a custom path is provided, it is used; otherwise, the default XDG last synced catalog path is used.
///
/// # Arguments
/// `path` - An optional custom path to the last synced catalog file.
///
/// # Returns
/// A `PathBuf` representing the last synced catalog file path.
pub fn last_synced_catalog_path(path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = path {
        return p.clone();
    }

    cross_xdg::BaseDirs::new()
        .expect("could not determine XDG base directories")
        .state_home()
        .join("aliasmgr")
        .join("last_synced_catalog.toml")
}

/// Load the catalog from the specified path or the default XDG catalog path.
/// If the file does not exist, an empty catalog is returned.
///
/// # Arguments
/// `path` - An optional custom path to the catalog file.
///
/// # Returns
/// A `Result` containing the loaded `AliasCatalog` or an error.
pub fn load_catalog(path: Option<&PathBuf>) -> Result<AliasCatalog> {
    let path = catalog_path(path);
    info!("Loading catalog from {:?}", path);

    if !path.exists() {
        info!(
            "alias catalog file {:?} does not exist, using empty catalog",
            path
        );
        return Ok(AliasCatalog::new());
    }

    let content = fs::read_to_string(path)?;
    let cfg: AliasCatalogSpec = toml::from_str(&content)?;
    Ok(convert_spec_to_catalog(cfg))
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
    if !alias.detailed && alias.enabled && !alias.global {
        Item::Value(alias.command.clone().into())
    } else {
        let mut inline = InlineTable::new();
        inline.insert("command", alias.command.clone().into());
        inline.insert("enabled", alias.enabled.into());
        inline.insert("global", alias.global.into());
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
                    "Alias '{}' references unknown '{}' group",
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

fn build_toml_document(catalog: &AliasCatalog) -> DocumentMut {
    let mut doc = DocumentMut::new();
    insert_groups(&mut doc, &catalog.groups);
    insert_aliases(&mut doc, &catalog.aliases, &catalog.groups);
    doc
}

/// Save the catalog to the specified path.
/// If the file does not exist, it will be created along with any necessary parent directories.
///
/// # Arguments
/// `path` - The path to the catalog file.
/// `catalog` - A reference to the `AliasCatalog` to be saved.
///
/// # Returns
/// A `Result` indicating success or failure.
fn save_catalog(catalog: &AliasCatalog, path: &PathBuf) -> Result<()> {
    if !path.exists() {
        warn!("alias catalog file {:?} does not exist, creating it", path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        debug!("Overwriting existing catalog at {:?}", path);
    } else {
        debug!("Saving content into new catalog at {:?}", path);
    }

    let doc = build_toml_document(catalog);
    let content = doc.to_string();
    fs::write(path, content)?;

    Ok(())
}

pub fn save_catalogs(
    catalog: &AliasCatalog,
    custom_catalog_path: Option<&PathBuf>,
    custom_last_synced_path: Option<&PathBuf>,
) -> Result<()> {
    let catalog_path = catalog_path(custom_catalog_path);
    save_catalog(catalog, &catalog_path)?;

    let last_synced_path = last_synced_catalog_path(custom_last_synced_path);
    save_catalog(catalog, &last_synced_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::tests::{SAMPLE_TOML, expected_catalog};
    use assert_fs::TempDir;

    #[test]
    fn build_alias_item_simple_enabled_is_string() {
        let alias = Alias::new("cmd".into(), None, true, false);
        let item = build_alias_item(&alias);
        assert!(item.as_value().unwrap().as_str().is_some());
        assert_eq!(item.to_string(), "\"cmd\"");
    }

    #[test]
    fn build_alias_item_detailed_is_inline_table() {
        let mut alias = Alias::new("cmd".into(), None, true, false);
        alias.detailed = true;

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
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("group".into(), true);
        catalog.aliases.insert(
            "alias".into(),
            Alias::new("foo".into(), Some("group".into()), false, false),
        );

        let doc = build_toml_document(&catalog);
        let rendered = doc.to_string();

        assert!(rendered.contains("[group]"));
        assert!(
            rendered.contains("alias = { command = \"foo\", enabled = false, global = false }")
        );
        assert!(!rendered.contains("[group.alias]"));
    }

    #[test]
    fn build_document_writes_top_level_simple_alias() {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("ls".into(), Alias::new("ls -la".into(), None, true, false));

        let doc = build_toml_document(&catalog);
        let rendered = doc.to_string();
        assert!(rendered.contains("ls = \"ls -la\""));
    }

    #[test]
    fn test_load_catalog() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");
        fs::write(&temp_conf, SAMPLE_TOML).unwrap();

        let cfg = load_catalog(Some(&temp_conf)).unwrap();
        assert_eq!(cfg, expected_catalog());
    }

    #[test]
    fn test_save_catalog() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");

        let catalog = expected_catalog();
        save_catalog(&catalog, &temp_conf).unwrap();

        let saved_content = fs::read_to_string(&temp_conf).unwrap();
        assert_eq!(saved_content, SAMPLE_TOML);
    }

    #[test]
    fn test_save_catalogs_writes_catalog_and_last_synced_catalog() {
        let temp_dir = TempDir::new().unwrap();
        let catalog_path = temp_dir.path().join("aliases.toml");
        let last_synced_catalog_path = temp_dir.path().join("last_synced_catalog.toml");

        let catalog = expected_catalog();
        save_catalogs(
            &catalog,
            Some(&catalog_path),
            Some(&last_synced_catalog_path),
        )
        .unwrap();

        assert_eq!(fs::read_to_string(&catalog_path).unwrap(), SAMPLE_TOML);
        assert_eq!(
            fs::read_to_string(&last_synced_catalog_path).unwrap(),
            SAMPLE_TOML
        );
    }

    #[test]
    fn test_catalog_path_custom() {
        let custom_path = PathBuf::from("/custom/path/aliases.toml");
        let path = catalog_path(Some(&custom_path));
        assert_eq!(path, custom_path);
    }

    #[test]
    fn test_catalog_path_default() {
        let path = catalog_path(None);
        assert!(path.ends_with(".config/aliasmgr/aliases.toml"));
    }

    #[test]
    fn test_last_synced_catalog_path_custom() {
        let custom_path = PathBuf::from("/custom/path/last_synced_catalog.toml");
        let path = last_synced_catalog_path(Some(&custom_path));
        assert_eq!(path, custom_path);
    }

    #[test]
    fn test_last_synced_catalog_path_default() {
        let path = last_synced_catalog_path(None);
        assert!(path.ends_with(".local/state/aliasmgr/last_synced_catalog.toml"));
    }

    #[test]
    fn test_load_catalog_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("nonexistent.toml");
        let cfg = load_catalog(Some(&temp_conf)).unwrap();
        assert_eq!(cfg, AliasCatalog::new());
    }

    // #[test]
    // fn test_build_alias_item_disabled_detailed() {
    //     let alias = Alias::new("cmd".into(), None, false, true);
    //     let item = build_alias_item(&alias);
    //     let inline = item
    //         .as_value()
    //         .and_then(|v| v.as_inline_table())
    //         .expect("expected inline table");
    //     assert_eq!(inline.get("command").unwrap().as_str(), Some("cmd"));
    //     assert_eq!(inline.get("enabled").unwrap().as_bool(), Some(false));
    // }

    #[test]
    fn test_insert_alias_to_unknown_group() {
        let mut doc = DocumentMut::new();
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "alias".into(),
            Alias::new("foo".into(), Some("unknown_group".into()), true, false),
        );

        insert_aliases(&mut doc, &catalog.aliases, &catalog.groups);
        let rendered = doc.to_string();
        assert!(rendered.contains("alias = \"foo\""));
    }

    #[test]
    fn test_save_catalog_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested/dir/aliases.toml");
        let catalog = expected_catalog();
        save_catalog(&catalog, &nested_path).unwrap();
        assert!(nested_path.exists());
    }

    #[test]
    fn test_save_catalog_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("new_aliases.toml");
        let catalog = expected_catalog();
        save_catalog(&catalog, &temp_conf).unwrap();
        assert!(temp_conf.exists());
    }

    #[test]
    fn test_save_catalog_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");
        fs::write(&temp_conf, "old_content").unwrap();
        let catalog = expected_catalog();
        save_catalog(&catalog, &temp_conf).unwrap();
        let saved_content = fs::read_to_string(&temp_conf).unwrap();
        assert_ne!(saved_content, "old_content");
    }

    #[test]
    fn test_build_alias_item_disabled_simple() {
        let alias = Alias::new("cmd".into(), None, false, false);
        let item = build_alias_item(&alias);
        let inline = item
            .as_value()
            .and_then(|v| v.as_inline_table())
            .expect("expected inline table");
        assert_eq!(inline.get("command").unwrap().as_str(), Some("cmd"));
        assert_eq!(inline.get("enabled").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn test_build_alias_item_global_detailed() {
        let alias = Alias::new("cmd".into(), None, true, true);
        let item = build_alias_item(&alias);
        let inline = item
            .as_value()
            .and_then(|v| v.as_inline_table())
            .expect("expected inline table");
        assert_eq!(inline.get("command").unwrap().as_str(), Some("cmd"));
        assert_eq!(inline.get("enabled").unwrap().as_bool(), Some(true));
        assert_eq!(inline.get("global").unwrap().as_bool(), Some(true));
    }
}
