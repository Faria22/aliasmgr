use log::{debug, warn};
use std::error::Error;
use std::path::PathBuf;
use std::{fs, io};

use super::spec::{ConfigSpec, convert_config_to_spec, convert_spec_to_config};
use super::types::Config;

pub fn config_path(path: Option<&PathBuf>) -> PathBuf {
    if let Some(p) = path {
        return p.clone();
    }

    cross_xdg::BaseDirs::new()
        .expect("could not determine XDG base directories")
        .config_home()
        .join("aliasmgr")
        .join("aliases.toml")
}

pub fn load_config(path: Option<&PathBuf>) -> Result<Config, Box<dyn Error>> {
    let path = config_path(path);

    if !path.exists() {
        warn!("Config file {:?} does not exist, using empty config", path);
        return Ok(Config::new());
    }

    debug!("Loading config from {:?}", path);

    let content = fs::read_to_string(path)?;
    let cfg: ConfigSpec = toml::from_str(&content)?;
    Ok(convert_spec_to_config(cfg))
}

pub fn save_config(path: Option<&PathBuf>, config: &Config) -> io::Result<()> {
    let path = config_path(path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    debug!("Saving config to {:?}", path);

    let spec = convert_config_to_spec(config);
    let content = toml::to_string_pretty(&spec).expect("failed to serialize config");
    fs::write(path, content)?;
    Ok(())
}
