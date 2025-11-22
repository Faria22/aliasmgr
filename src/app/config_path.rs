use crate::cli::interaction::prompt_use_non_existing_config_file;
use std::env;
use std::path::PathBuf;

use anyhow::{Result, bail};

pub const CONFIG_FILE_ENV_VAR: &str = "ALIASMGR_CONFIG_PATH";

pub fn determine_config_path() -> Result<Option<PathBuf>> {
    if let Ok(path) = env::var(CONFIG_FILE_ENV_VAR) {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(Some(path));
        }

        if prompt_use_non_existing_config_file(path.to_str().unwrap()) {
            return Ok(Some(path));
        }

        bail!(
            "Configuration file '{}' does not exist and user chose not to use it.",
            path.to_str().unwrap()
        );
    } else {
        Ok(None)
    }
}
