//! Load and save the alias catalog.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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

fn build_alias_item(alias: &mut Alias) -> Item {
    alias.refresh_representation();
    if !alias.detailed {
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

fn build_toml_document(catalog: &mut AliasCatalog) -> DocumentMut {
    let mut document = DocumentMut::new();
    for (name, alias) in &mut catalog.aliases {
        document[name] = build_alias_item(alias);
    }
    document
}

pub fn save_catalog(catalog: &mut AliasCatalog, path: &PathBuf) -> Result<()> {
    let content = build_toml_document(catalog).to_string();
    if !path.exists() {
        warn!("alias catalog file {:?} does not exist, creating it", path);
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    debug!("Saving catalog to {:?}", path);
    atomic_write(path, content.as_bytes())
        .with_context(|| format!("could not save catalog '{}'", path.display()))
}

fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let write_path =
        if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            fs::canonicalize(path)?
        } else {
            path.to_owned()
        };
    let parent = write_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let permissions = fs::metadata(&write_path)
        .ok()
        .map(|metadata| metadata.permissions());
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(content)?;
    temporary.as_file_mut().sync_all()?;
    if let Some(permissions) = permissions {
        temporary.as_file().set_permissions(permissions)?;
    }
    temporary
        .persist(&write_path)
        .map_err(|error| error.error)?;
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    Ok(())
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
        catalog.aliases.insert("test".into(), alias);

        save_catalog(&mut catalog, &path).unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("tags = [\"dev\", \"rust\"]"));
        assert_eq!(load_catalog(&path).unwrap(), catalog);
    }

    #[test]
    fn saving_refreshes_each_alias_representation() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        let mut catalog = AliasCatalog::new();

        let mut disabled = Alias::new("disabled".into(), true, false);
        disabled.enabled = false;
        catalog.aliases.insert("disabled".into(), disabled);

        let mut global = Alias::new("global".into(), true, false);
        global.global = true;
        catalog.aliases.insert("global".into(), global);

        let mut described = Alias::new("described".into(), true, false);
        described.description = Some("Description".into());
        catalog.aliases.insert("described".into(), described);

        let mut tagged = Alias::new("tagged".into(), true, false);
        tagged.tags.insert("tag".into());
        catalog.aliases.insert("tagged".into(), tagged);

        assert!(catalog.aliases.values().all(|alias| !alias.detailed));
        save_catalog(&mut catalog, &path).unwrap();

        assert!(catalog.aliases.values().all(|alias| alias.detailed));
        let saved = fs::read_to_string(&path).unwrap();
        assert_eq!(
            saved,
            concat!(
                "described = { command = \"described\", enabled = true, global = false, description = \"Description\" }\n",
                "disabled = { command = \"disabled\", enabled = false, global = false }\n",
                "global = { command = \"global\", enabled = true, global = true }\n",
                "tagged = { command = \"tagged\", enabled = true, global = false, tags = [\"tag\"] }\n",
            )
        );
        assert_eq!(load_catalog(&path).unwrap(), catalog);
    }

    #[test]
    fn saving_normalizes_unneeded_detailed_representation() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        fs::write(
            &path,
            "ll = { command = \"ls -la\", enabled = true, global = false }\n",
        )
        .unwrap();
        let mut catalog = load_catalog(&path).unwrap();
        assert!(catalog.aliases["ll"].detailed);

        save_catalog(&mut catalog, &path).unwrap();

        assert!(!catalog.aliases["ll"].detailed);
        assert_eq!(fs::read_to_string(&path).unwrap(), "ll = \"ls -la\"\n");
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
        let mut catalog = load_catalog(&path).unwrap();
        assert!(!catalog.aliases["ll"].detailed);
        assert!(catalog.aliases["test"].tags.contains("dev"));

        save_catalog(&mut catalog, &path).unwrap();
        assert_eq!(load_catalog(&path).unwrap(), catalog);
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

    #[cfg(unix)]
    #[test]
    fn catalog_replacement_is_atomic_and_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        fs::write(&path, "old = \"value\"\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("new".into(), Alias::new("value".into(), true, false));

        save_catalog(&mut catalog, &path).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new = \"value\"\n");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn atomic_save_preserves_a_catalog_symlink() {
        use std::os::unix::fs::symlink;

        let directory = TempDir::new().unwrap();
        let target = directory.path().join("managed.toml");
        let link = directory.path().join("aliases.toml");
        fs::write(&target, "old = \"value\"\n").unwrap();
        symlink(&target, &link).unwrap();
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("new".into(), Alias::new("value".into(), true, false));

        save_catalog(&mut catalog, &link).unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(&target).unwrap(), "new = \"value\"\n");
    }
}
