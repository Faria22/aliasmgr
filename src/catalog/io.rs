//! Load and save the alias catalog.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use log::{debug, info, warn};
use toml_edit::{DocumentMut, InlineTable, Item};

use super::spec::{AliasCatalogSpec, convert_spec_to_catalog};
use super::types::{Alias, AliasCatalog};

pub fn catalog_path(path: Option<&PathBuf>) -> PathBuf {
    if let Some(path) = path {
        info!("Using custom catalog path: {:?}", path);
        return path.clone();
    }
    cross_xdg::BaseDirs::new()
        .expect("could not determine XDG base directories")
        .config_home()
        .join("aliasmgr")
        .join("aliases.toml")
}

pub fn load_catalog(path: &PathBuf) -> Result<AliasCatalog> {
    info!("Loading catalog from {:?}", path);
    if !path.exists() {
        return Ok(AliasCatalog::new());
    }

    let content = fs::read_to_string(path)?;
    let document = content.parse::<DocumentMut>()?;
    if let Some((name, _)) = document.iter().find(|(_, item)| item.is_table()) {
        bail!(
            "legacy alias groups are unsupported (found group '{name}'); migrate the catalog before using this version"
        );
    }

    let spec: AliasCatalogSpec = toml::from_str(&content)?;
    Ok(convert_spec_to_catalog(spec))
}

fn build_alias_item(alias: &Alias) -> Item {
    if !alias.detailed
        && alias.enabled
        && !alias.global
        && alias.description.is_none()
        && alias.tags.is_empty()
    {
        return Item::Value(alias.command.clone().into());
    }

    let mut inline = InlineTable::new();
    inline.insert("command", alias.command.clone().into());
    inline.insert("enabled", alias.enabled.into());
    inline.insert("global", alias.global.into());
    if let Some(description) = &alias.description {
        inline.insert("description", description.clone().into());
    }
    if !alias.tags.is_empty() {
        let mut tags = toml_edit::Array::new();
        for tag in &alias.tags {
            tags.push(tag.as_str());
        }
        inline.insert("tags", toml_edit::Value::Array(tags));
    }
    inline.set_dotted(false);
    Item::Value(inline.into())
}

fn build_toml_document(catalog: &AliasCatalog) -> DocumentMut {
    let mut document = DocumentMut::new();
    for (name, alias) in &catalog.aliases {
        document[name] = build_alias_item(alias);
    }
    document
}

pub fn save_catalog(catalog: &AliasCatalog, path: &PathBuf) -> Result<()> {
    let content = build_toml_document(catalog).to_string();
    if !path.exists() {
        warn!("alias catalog file {:?} does not exist, creating it", path);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    debug!("Saving catalog to {:?}", path);
    fs::write(path, content).with_context(|| format!("could not save catalog '{}'", path.display()))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    #[test]
    fn metadata_round_trip_is_sorted_and_detailed() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        let mut catalog = AliasCatalog::new();
        let mut alias = Alias::new("cargo test".into(), true, false);
        alias.description = Some("Run tests".into());
        alias
            .tags
            .extend(["rust".into(), "dev".into(), "rust".into()]);
        alias.refresh_representation();
        catalog.aliases.insert("test".into(), alias);

        save_catalog(&catalog, &path).unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("tags = [\"dev\", \"rust\"]"));
        assert_eq!(load_catalog(&path).unwrap(), catalog);
    }

    #[test]
    fn mixed_simple_and_detailed_aliases_load() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        fs::write(
            &path,
            "ll = \"ls -la\"\ntest = { command = \"cargo test\", tags = [\"dev\"] }\n",
        )
        .unwrap();
        let catalog = load_catalog(&path).unwrap();
        assert!(!catalog.aliases["ll"].detailed);
        assert!(catalog.aliases["test"].tags.contains("dev"));
    }

    #[test]
    fn legacy_groups_fail_clearly() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        fs::write(&path, "[dev]\nbuild = \"cargo build\"\n").unwrap();
        let error = load_catalog(&path).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("legacy alias groups are unsupported")
        );
    }

    #[test]
    fn missing_catalog_is_empty() {
        let directory = TempDir::new().unwrap();
        assert_eq!(
            load_catalog(&directory.path().join("missing.toml")).unwrap(),
            AliasCatalog::new()
        );
    }

    #[test]
    fn custom_catalog_path_is_used_verbatim() {
        let path = PathBuf::from("custom-aliases.toml");
        assert_eq!(catalog_path(Some(&path)), path);
    }

    #[test]
    fn default_catalog_path_uses_aliasmgr_config_directory() {
        assert!(catalog_path(None).ends_with("aliasmgr/aliases.toml"));
    }
}
