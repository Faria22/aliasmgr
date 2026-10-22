use crate::cli::interaction::{InteractionMode, prompt_use_non_existing_catalog_file};
use std::env;
use std::path::{Path, PathBuf};

pub const CATALOG_FILE_ENV_VAR: &str = "ALIASMGR_CATALOG_PATH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogPathDecision {
    Use(Option<PathBuf>),
    Decline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    Catalog,
}

impl FileType {
    fn env_var(self) -> &'static str {
        match self {
            FileType::Catalog => CATALOG_FILE_ENV_VAR,
        }
    }
}

pub fn determine_catalog_path(mode: InteractionMode, mutates_catalog: bool) -> CatalogPathDecision {
    determine_configured_file_path(FileType::Catalog, |path| {
        !mutates_catalog || prompt_use_non_existing_catalog_file(mode, path)
    })
}

fn determine_configured_file_path(
    file_type: FileType,
    prompt: impl Fn(&str) -> bool,
) -> CatalogPathDecision {
    if let Some(path) = env::var_os(file_type.env_var()) {
        let path = PathBuf::from(path);
        handle_configured_file_path(&path, prompt)
    } else {
        CatalogPathDecision::Use(None)
    }
}

fn handle_configured_file_path(path: &Path, create: impl Fn(&str) -> bool) -> CatalogPathDecision {
    if path.exists() {
        return CatalogPathDecision::Use(Some(path.to_path_buf()));
    }

    if create(&path.to_string_lossy()) {
        return CatalogPathDecision::Use(Some(path.to_path_buf()));
    }

    CatalogPathDecision::Decline
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use temp_env::with_var;

    #[test]
    fn test_determine_catalog_path_env_var_set_existing_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        with_var(
            CATALOG_FILE_ENV_VAR,
            Some(temp_file.path().to_str().unwrap()),
            || {
                let result = determine_catalog_path(InteractionMode::Interactive, true);
                assert_eq!(
                    result,
                    CatalogPathDecision::Use(Some(temp_file.path().to_path_buf()))
                );
            },
        );
    }

    #[test]
    fn test_determine_catalog_path_set_non_existing_file_user_accepts() {
        let result =
            handle_configured_file_path(&PathBuf::from("/non/existing/catalog/file"), |_| true);
        assert_eq!(
            result,
            CatalogPathDecision::Use(Some(PathBuf::from("/non/existing/catalog/file")))
        );
    }

    #[test]
    fn test_determine_catalog_path_set_non_existing_file_user_declines() {
        let result =
            handle_configured_file_path(&PathBuf::from("/non/existing/catalog/file"), |_| false);
        assert_eq!(result, CatalogPathDecision::Decline);
    }

    #[test]
    fn test_determine_catalog_path_env_var_not_set() {
        with_var(CATALOG_FILE_ENV_VAR, None as Option<&str>, || {
            let result = determine_catalog_path(InteractionMode::Interactive, true);
            assert_eq!(result, CatalogPathDecision::Use(None));
        });
    }

    #[test]
    fn missing_catalog_only_requires_confirmation_for_mutations() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("missing.toml");
        with_var(CATALOG_FILE_ENV_VAR, Some(path.as_os_str()), || {
            assert_eq!(
                determine_catalog_path(InteractionMode::NoInput, false),
                CatalogPathDecision::Use(Some(path.clone()))
            );
            assert_eq!(
                determine_catalog_path(InteractionMode::No, true),
                CatalogPathDecision::Decline
            );
            assert_eq!(
                determine_catalog_path(InteractionMode::Yes, true),
                CatalogPathDecision::Use(Some(path.clone()))
            );
            assert!(!path.exists());
        });
    }

    #[cfg(unix)]
    #[test]
    fn missing_non_utf8_path_is_displayed_lossily_and_preserved() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let directory = tempfile::tempdir().unwrap();
        let path = directory
            .path()
            .join(OsString::from_vec(b"aliases-\xff.toml".to_vec()));
        let result = handle_configured_file_path(&path, |display| {
            assert_eq!(display, path.to_string_lossy());
            assert!(display.contains('\u{fffd}'));
            true
        });
        assert_eq!(result, CatalogPathDecision::Use(Some(path.clone())));
        assert_eq!(
            handle_configured_file_path(&path, |_| false),
            CatalogPathDecision::Decline
        );
        with_var(CATALOG_FILE_ENV_VAR, Some(path.as_os_str()), || {
            assert_eq!(
                determine_catalog_path(InteractionMode::Yes, true),
                CatalogPathDecision::Use(Some(path.clone()))
            );
        });
        assert!(!path.exists());
    }
}
