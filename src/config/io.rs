use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::{fs, io};

use crate::cli::Cli;

use super::spec::{ConfigSpec, convert_config_to_spec, convert_spec_to_config};
use super::types::Config;

pub fn config_path(cli: &Cli) -> PathBuf {
    if let Some(config_path) = &cli.config {
        return config_path.to_path_buf();
    }

    cross_xdg::BaseDirs::new()
        .expect("could not determine XDG base directories")
        .config_home()
        .join("aliasmgr")
        .join("aliases.toml")
}

pub fn load_config(cli: &Cli) -> Result<Config, Box<dyn Error>> {
    let path = config_path(cli);

    if !path.exists() {
        return Ok(Config {
            aliases: HashMap::new(),
            groups: HashMap::new(),
        });
    }

    let content = fs::read_to_string(path)?;
    let cfg: ConfigSpec = toml::from_str(&content)?;
    Ok(convert_spec_to_config(cfg))
}

pub fn save_config(cli: &Cli, config: &Config) -> io::Result<()> {
    let path = config_path(&cli);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let spec = convert_config_to_spec(config);
    let content = toml::to_string_pretty(&spec).expect("failed to serialize config");
    fs::write(path, content)?;
    Ok(())
}
