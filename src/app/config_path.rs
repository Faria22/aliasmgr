use crate::cli::interaction::use_non_existing_config_file;
use std::env;
use std::path::PathBuf;

use anyhow::{Result, bail};

pub fn determine_config_path() -> Result<Option<PathBuf>> {
    if let Ok(path) = env::var("ALIASMGR_CONFIG_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(Some(path));
        }

        if use_non_existing_config_file(path.to_str().unwrap()) {
            return Ok(Some(path));
        }

        bail!(
            "Configuration file '{}' does not exist and user chose not to use it.",
            path.to_str().unwrap()
        );
    } else {
        return Ok(None);
    }
}
