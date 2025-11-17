use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Alias {
    pub command: String,
    pub enable: bool,
    pub group: Option<String>,
    pub detailed: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Group {
    pub enable: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Config {
    pub aliases: HashMap<String, Alias>,
    pub groups: HashMap<String, Group>,
}
