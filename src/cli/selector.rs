use clap::{ArgGroup, Args};

use super::validate_tag;

#[derive(Args)]
#[command(group(ArgGroup::new("alias_selector").args(["name", "pattern", "tag"]).required(true).multiple(true)))]
pub struct AliasSelectorArgs {
    #[arg(conflicts_with_all = ["pattern", "tag"])]
    pub name: Option<String>,
    #[arg(long, value_name = "GLOB", value_parser = validate_glob)]
    pub pattern: Option<String>,
    /// Select aliases containing every supplied tag
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
}

impl AliasSelectorArgs {
    pub fn is_filter(&self) -> bool {
        self.pattern.is_some() || !self.tag.is_empty()
    }
}

fn validate_glob(pattern: &str) -> Result<String, String> {
    globset::Glob::new(pattern)
        .map(|_| pattern.to_owned())
        .map_err(|error| format!("invalid glob pattern: {error}"))
}
