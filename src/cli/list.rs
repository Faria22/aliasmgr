use clap::{ArgGroup, Args, ValueEnum};
use serde::Deserialize;

use super::validate_tag;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ListColumn {
    Status,
    Name,
    Command,
    Global,
    Tags,
    Description,
}

impl ListColumn {
    pub const DEFAULTS: [Self; 6] = [
        Self::Status,
        Self::Name,
        Self::Command,
        Self::Global,
        Self::Tags,
        Self::Description,
    ];
}

#[derive(Args)]
#[command(group(ArgGroup::new("list_scope").args(["disabled", "all"]).multiple(false)))]
pub struct ListCommand {
    pub pattern: Option<String>,
    /// List aliases containing every supplied tag
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
    /// List only disabled aliases
    #[arg(short = 'd', long)]
    pub disabled: bool,
    /// List enabled and disabled aliases
    #[arg(long)]
    pub all: bool,
    /// List only Zsh global aliases
    #[arg(short = 'g', long)]
    pub global: bool,
    /// Select human-readable or JSON output
    #[arg(short = 'f', long, value_enum, default_value = "human")]
    pub format: OutputFormat,
    /// Override configured table columns, in display order
    #[arg(long, value_enum, value_delimiter = ',', num_args = 1..)]
    pub columns: Option<Vec<ListColumn>>,
}
