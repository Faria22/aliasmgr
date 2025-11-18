use std::collections::HashMap;

#[derive(PartialEq, Eq)]
pub struct Alias {
    pub command: String,
    pub enabled: bool,
    pub group: Option<String>,
    pub detailed: bool,
}

#[derive(PartialEq, Eq)]
pub struct Config {
    pub aliases: HashMap<String, Alias>,
    pub groups: HashMap<String, bool>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            aliases: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}
