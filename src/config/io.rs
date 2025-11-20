//! This module provides functions to load and save the configuration for the alias manager.
//! ! It supports loading from and saving to a specified path or the default XDG configuration path.
//! ! The configuration is serialized and deserialized using the TOML format.
//! ! It also handles the creation of necessary directories if they do not exist.

use log::{debug, info, warn};
use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use super::spec::{ConfigSpec, convert_config_to_spec, convert_spec_to_config};
use super::types::Config;

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

    let spec = convert_config_to_spec(config);
    let content = toml::to_string_pretty(&spec).expect("failed to serialize config");
    fs::write(path, content)?;

    Ok(())
}
