//! Load and save the alias catalog.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use log::{debug, info, warn};
use toml_edit::{DocumentMut, InlineTable, Item};

use super::spec::{AliasCatalogSpec, convert_spec_to_catalog};
use super::types::{Alias, AliasCatalog};

pub fn catalog_path(path: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(path) = path {
        info!("Using custom catalog path: {:?}", path);
        return Ok(path.clone());
    }
    Ok(crate::config::config_home()
        .context("could not resolve default catalog path; set ALIASMGR_CATALOG_PATH or use init --catalog")?
        .join("aliasmgr")
        .join("aliases.toml"))
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

/// Resolve links in both the catalog name and its parents, including dangling
/// links to a catalog that has not been created yet.
fn resolve_target(path: &Path) -> Result<PathBuf> {
    fn resolve(path: &Path, links: &mut usize) -> io::Result<PathBuf> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                *links += 1;
                if *links > 40 {
                    return Err(io::Error::other("too many catalog symlinks"));
                }
                let target = fs::read_link(path)?;
                let target = if target.is_absolute() {
                    target
                } else {
                    path.parent().unwrap_or(Path::new(".")).join(target)
                };
                resolve(&target, links)
            }
            Ok(_) => fs::canonicalize(path),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let name = path.file_name().ok_or(error)?;
                let parent = path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new("."));
                Ok(resolve(parent, links)?.join(name))
            }
            Err(error) => Err(error),
        }
    }
    resolve(path, &mut 0).with_context(|| format!("could not resolve catalog '{}'", path.display()))
}

/// Hold this guard from before loading until after saving a mutating command.
/// The sidecar must stay in place: unlinking it would let waiting processes
/// lock a different inode and break serialization after an atomic replacement.
pub struct CatalogTransaction {
    _lock: File,
    path: PathBuf,
}

impl CatalogTransaction {
    pub fn begin(path: &Path) -> Result<Self> {
        Self::begin_with_lock(path, File::lock)
    }

    fn begin_with_lock(path: &Path, acquire: impl FnOnce(&File) -> io::Result<()>) -> Result<Self> {
        let path = resolve_target(path)?;
        let parent = path.parent().context("catalog path has no parent")?;
        fs::create_dir_all(parent).with_context(|| {
            format!("could not create catalog directory '{}'", parent.display())
        })?;
        let mut lock_path = path.as_os_str().to_os_string();
        lock_path.push(".lock");
        let lock_path = PathBuf::from(lock_path);
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .with_context(|| format!("could not open catalog lock '{}'", lock_path.display()))?;
        debug!("Waiting for catalog lock '{}'", lock_path.display());
        acquire(&lock).with_context(|| format!("could not lock catalog '{}'", path.display()))?;
        debug!("Acquired catalog lock '{}'", lock_path.display());
        Ok(Self { _lock: lock, path })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

fn atomic_replace(path: &Path, write: impl FnOnce(&mut File) -> io::Result<()>) -> Result<()> {
    let parent = path.parent().context("catalog path has no parent")?;
    fs::create_dir_all(parent)?;
    let permissions = match fs::metadata(path) {
        Ok(metadata) => {
            // Preserve the old write-access check even though rename only needs
            // directory access. Never truncate the original file.
            OpenOptions::new().write(true).open(path)?;
            Some(metadata.permissions())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    write(temporary.as_file_mut())?;
    temporary.as_file_mut().flush()?;
    if let Some(permissions) = permissions {
        temporary.as_file().set_permissions(permissions)?;
    }
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub fn save_catalog(catalog: &mut AliasCatalog, path: &PathBuf) -> Result<()> {
    let content = build_toml_document(catalog).to_string();
    let target = resolve_target(path)?;
    if !target.exists() {
        warn!("alias catalog file {:?} does not exist, creating it", path);
    }
    debug!("Saving catalog to {:?}", target);
    atomic_replace(&target, |file| file.write_all(content.as_bytes()))
        .with_context(|| format!("could not save catalog '{}'", path.display()))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use assert_fs::TempDir;

    fn example_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("ll".into(), Alias::new("ls -la".into(), true, false));
        catalog
    }

    #[test]
    fn failed_partial_write_keeps_original_and_removes_temporary_file() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        fs::write(&path, "original = \"echo original\"\n").unwrap();
        let error = atomic_replace(&path, |file| {
            file.write_all(b"partial")?;
            Err(io::Error::other("simulated disk full"))
        })
        .unwrap_err();
        assert!(error.to_string().contains("simulated disk full"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "original = \"echo original\"\n"
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn saving_preserves_symlink_chain_and_permissions() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let directory = TempDir::new().unwrap();
        let target = directory.path().join("target.toml");
        fs::write(&target, "old = \"echo old\"\n").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
        let link = directory.path().join("aliases.toml");
        let intermediate = directory.path().join("intermediate.toml");
        symlink("target.toml", &intermediate).unwrap();
        symlink("intermediate.toml", &link).unwrap();
        save_catalog(&mut example_catalog(), &link).unwrap();
        assert_eq!(
            fs::read_link(&link).unwrap(),
            Path::new("intermediate.toml")
        );
        assert_eq!(
            fs::read_link(&intermediate).unwrap(),
            Path::new("target.toml")
        );
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o640
        );
        assert_eq!(load_catalog(&target).unwrap(), example_catalog());
    }

    #[cfg(unix)]
    #[test]
    fn saving_through_dangling_symlink_creates_target() {
        use std::os::unix::fs::symlink;
        let directory = TempDir::new().unwrap();
        let link = directory.path().join("aliases.toml");
        let target = directory.path().join("nested/target.toml");
        symlink("nested/target.toml", &link).unwrap();
        let transaction = CatalogTransaction::begin(&link).unwrap();
        assert_eq!(
            transaction.path(),
            &target
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap()
                .join("target.toml")
        );
        save_catalog(&mut example_catalog(), transaction.path()).unwrap();
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(load_catalog(&target).unwrap(), example_catalog());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loop_returns_error() {
        use std::os::unix::fs::symlink;
        let directory = TempDir::new().unwrap();
        let link = directory.path().join("loop");
        symlink("loop", &link).unwrap();
        assert!(
            format!("{:#}", resolve_target(&link).unwrap_err())
                .contains("too many catalog symlinks")
        );
    }

    #[test]
    fn transaction_uses_stable_sidecar_across_replacement() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        let transaction = CatalogTransaction::begin(&path).unwrap();
        save_catalog(&mut example_catalog(), transaction.path()).unwrap();
        let contender = OpenOptions::new()
            .read(true)
            .write(true)
            .open(directory.path().join("aliases.toml.lock"))
            .unwrap();
        assert!(matches!(
            contender.try_lock(),
            Err(std::fs::TryLockError::WouldBlock)
        ));
        drop(transaction);
        contender.try_lock().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn saving_through_absolute_symlink_preserves_link() {
        use std::os::unix::fs::symlink;
        let directory = TempDir::new().unwrap();
        let target = directory.path().join("target.toml");
        let link = directory.path().join("aliases.toml");
        symlink(&target, &link).unwrap();
        save_catalog(&mut example_catalog(), &link).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), target);
        assert_eq!(load_catalog(&target).unwrap(), example_catalog());
    }

    #[test]
    fn lock_open_failure_reports_sidecar_path() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        let lock = directory.path().join("aliases.toml.lock");
        fs::create_dir(&lock).unwrap();
        let error = CatalogTransaction::begin(&path)
            .err()
            .expect("directory cannot be opened as lock file");
        let message = format!("{error:#}");
        assert!(message.contains("could not open catalog lock"), "{message}");
        assert!(message.contains("aliases.toml.lock"), "{message}");
        assert!(!path.exists());
    }

    #[test]
    fn lock_acquisition_failure_reports_catalog_and_releases_handle() {
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("aliases.toml");
        fs::write(&path, "original = \"echo original\"\n").unwrap();
        let error = CatalogTransaction::begin_with_lock(&path, |file| {
            // Model a filesystem lock failure after opening the sidecar.
            file.lock()?;
            Err(io::Error::other("simulated filesystem lock failure"))
        })
        .err()
        .expect("lock failure must propagate");
        let message = format!("{error:#}");
        assert!(message.contains("could not lock catalog"), "{message}");
        assert!(message.contains("aliases.toml"), "{message}");
        assert!(
            message.contains("simulated filesystem lock failure"),
            "{message}"
        );
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(directory.path().join("aliases.toml.lock"))
            .unwrap();
        lock.try_lock().unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "original = \"echo original\"\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn inaccessible_catalog_reports_resolution_and_metadata_errors() {
        use std::os::unix::fs::PermissionsExt;
        let directory = TempDir::new().unwrap();
        let parent = directory.path().join("private");
        fs::create_dir(&parent).unwrap();
        let path = parent.join("aliases.toml");
        fs::write(&path, "original = \"echo original\"\n").unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o000)).unwrap();
        let resolution = resolve_target(&path);
        let replacement = atomic_replace(&path, |file| file.write_all(b"replacement"));
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let message = format!("{:#}", resolution.unwrap_err());
        assert!(message.contains("could not resolve catalog"), "{message}");
        assert!(
            message.to_lowercase().contains("permission denied"),
            "{message}"
        );
        assert_eq!(
            replacement
                .unwrap_err()
                .downcast_ref::<io::Error>()
                .unwrap()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "original = \"echo original\"\n"
        );
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn catalog_directory_creation_failure_reports_directory() {
        use std::os::unix::fs::PermissionsExt;
        let directory = TempDir::new().unwrap();
        let parent = directory.path().join("readonly");
        fs::create_dir(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o555)).unwrap();
        let path = parent.join("missing/aliases.toml");
        let result = CatalogTransaction::begin(&path);
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        let error = result
            .err()
            .expect("readonly directory must reject creation");
        let message = format!("{error:#}");
        assert!(
            message.contains("could not create catalog directory"),
            "{message}"
        );
        assert!(message.contains("missing"), "{message}");
        assert!(
            message.to_lowercase().contains("permission denied"),
            "{message}"
        );
        assert!(!path.exists());
    }

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
        assert_eq!(catalog_path(Some(&path)).unwrap(), path);
    }

    #[test]
    fn default_catalog_path_uses_aliasmgr_config_directory() {
        let directory = tempfile::tempdir().unwrap();
        temp_env::with_vars(
            [
                ("HOME", Some(directory.path().as_os_str())),
                ("XDG_CONFIG_HOME", None),
            ],
            || {
                assert_eq!(
                    catalog_path(None).unwrap(),
                    directory.path().join(".config/aliasmgr/aliases.toml")
                )
            },
        );
    }

    #[test]
    fn default_path_requires_home_or_xdg_config_home() {
        temp_env::with_vars([("HOME", None::<&str>), ("XDG_CONFIG_HOME", None)], || {
            let error = format!("{:#}", catalog_path(None).unwrap_err());
            assert!(error.contains("HOME"));
            assert!(error.contains("XDG_CONFIG_HOME"));
            let custom = PathBuf::from("custom.toml");
            assert_eq!(catalog_path(Some(&custom)).unwrap(), custom);
        });
    }

    #[test]
    fn xdg_config_home_works_without_home() {
        let directory = tempfile::tempdir().unwrap();
        temp_env::with_vars(
            [
                ("HOME", None),
                ("XDG_CONFIG_HOME", Some(directory.path().as_os_str())),
            ],
            || {
                assert_eq!(
                    catalog_path(None).unwrap(),
                    directory.path().join("aliasmgr/aliases.toml")
                )
            },
        );
    }
}
