use crate::cli::interaction::{InteractionMode, prompt_use_non_existing_catalog_file};
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

pub const CATALOG_FILE_ENV_VAR: &str = "ALIASMGR_CATALOG_PATH";

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

    fn display_name(self) -> &'static str {
        match self {
            FileType::Catalog => "Catalog file",
        }
    }
}

pub fn determine_catalog_path(mode: InteractionMode) -> Result<Option<PathBuf>> {
    determine_configured_file_path(FileType::Catalog, |path| {
        prompt_use_non_existing_catalog_file(mode, path)
    })
}

fn determine_configured_file_path(
    file_type: FileType,
    prompt: impl Fn(&str) -> bool,
) -> Result<Option<PathBuf>> {
    if let Ok(path) = env::var(file_type.env_var()) {
        let path = PathBuf::from(path);
        handle_configured_file_path(&path, prompt, file_type)
    } else {
        Ok(None)
    }
}

fn handle_configured_file_path(
    path: &Path,
    create: impl Fn(&str) -> bool,
    file_type: FileType,
) -> Result<Option<PathBuf>> {
    if path.exists() {
        return Ok(Some(path.to_path_buf()));
    }

    if create(path.to_str().unwrap()) {
        return Ok(Some(path.to_path_buf()));
    }

    bail!(
        "{} '{}' does not exist and user chose not to use it.",
        file_type.display_name(),
        path.to_str().unwrap()
    );
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
                let result = determine_catalog_path(InteractionMode::Interactive).unwrap();
                assert_eq!(result, Some(temp_file.path().to_path_buf()));
            },
        );
    }

    #[test]
    fn test_determine_catalog_path_set_non_existing_file_user_accepts() {
        let result = handle_configured_file_path(
            &PathBuf::from("/non/existing/catalog/file"),
            |_| true,
            FileType::Catalog,
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Some(PathBuf::from("/non/existing/catalog/file"))
        );
    }

    #[test]
    fn test_determine_catalog_path_set_non_existing_file_user_declines() {
        let result = handle_configured_file_path(
            &PathBuf::from("/non/existing/catalog/file"),
            |_| false,
            FileType::Catalog,
        );
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Catalog file '/non/existing/catalog/file'")
        );
    }

    #[test]
    fn test_determine_catalog_path_env_var_not_set() {
        with_var(CATALOG_FILE_ENV_VAR, None as Option<&str>, || {
            let result = determine_catalog_path(InteractionMode::Interactive).unwrap();
            assert_eq!(result, None);
        });
    }
}
