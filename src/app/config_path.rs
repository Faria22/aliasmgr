use crate::cli::interaction::prompt_use_non_existing_config_file;
use std::env;
use std::path::PathBuf;

use anyhow::{Result, bail};

pub const CONFIG_FILE_ENV_VAR: &str = "ALIASMGR_CONFIG_PATH";

pub fn determine_config_path() -> Result<Option<PathBuf>> {
    if let Ok(path) = env::var(CONFIG_FILE_ENV_VAR) {
        let path = PathBuf::from(path);
        handle_config_file(&path, prompt_use_non_existing_config_file)
    } else {
        Ok(None)
    }
}

fn handle_config_file(path: &PathBuf, create: impl Fn(&str) -> bool) -> Result<Option<PathBuf>> {
    if path.exists() {
        return Ok(Some(path.clone()));
    }

    if create(path.to_str().unwrap()) {
        return Ok(Some(path.clone()));
    }

    bail!(
        "Configuration file '{}' does not exist and user chose not to use it.",
        path.to_str().unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env::with_var;
    use tempfile;

    #[test]
    fn test_determine_config_path_env_var_set_existing_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        with_var(
            CONFIG_FILE_ENV_VAR,
            Some(temp_file.path().to_str().unwrap()),
            || {
                let result = determine_config_path().unwrap();
                assert_eq!(result, Some(temp_file.path().to_path_buf()));
            },
        );
    }

    #[test]
    fn test_determine_config_path_set_non_existing_file_user_accepts() {
        with_var(
            CONFIG_FILE_ENV_VAR,
            Some("/non/existing/config/file"),
            || {
                let result =
                    handle_config_file(&PathBuf::from("/non/existing/config/file"), |_| true);
                assert!(result.is_ok());
                assert_eq!(
                    result.unwrap(),
                    Some(PathBuf::from("/non/existing/config/file"))
                );
            },
        );
    }

    #[test]
    fn test_determine_config_path_set_non_existing_file_user_declines() {
        with_var(
            CONFIG_FILE_ENV_VAR,
            Some("/non/existing/config/file"),
            || {
                let result =
                    handle_config_file(&PathBuf::from("/non/existing/config/file"), |_| false);
                assert!(result.is_err());
            },
        );
    }

    #[test]
    fn test_determine_config_path_env_var_not_set() {
        with_var(CONFIG_FILE_ENV_VAR, None as Option<&str>, || {
            let result = determine_config_path().unwrap();
            assert_eq!(result, None);
        });
    }
}
