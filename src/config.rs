use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Alias {
    pub command: String,

    #[serde(default = "default_enabled")]
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    #[serde(default = "default_enabled")]
    pub enable: bool,

    #[serde(flatten)]
    pub aliases: HashMap<String, AliasSpec>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum AliasSpec {
    // foo = "..."
    Simple(String),

    // foo = { command = "...", enable = false }
    Detailed(Alias),

    // [foo]
    // bar = "..."
    Group(Group),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(flatten)]
    pub entries: HashMap<String, AliasSpec>,
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .expect("no config dir found")
        .join("aliasmgr")
        .join("aliases.toml")
}

pub fn load_config() -> io::Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Ok(Config {
            entries: HashMap::new(),
        });
    }

    let content = fs::read_to_string(path)?;
    let cfg: Config = toml::from_str(&content)?;
    Ok(cfg)
}
