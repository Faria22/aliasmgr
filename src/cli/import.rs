use std::path::PathBuf;

use clap::Args;

use super::validate_tag;

#[derive(Args)]
pub struct ImportCommand {
    /// Bash or Zsh files containing alias declarations
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
    /// Report what would happen without changing the catalog
    #[arg(short, long)]
    pub dry_run: bool,
    /// Keep catalog entries when imported aliases have the same names
    #[arg(short, long, conflicts_with = "replace_existing")]
    pub skip_existing: bool,
    /// Replace catalog entries when imported aliases have the same names
    #[arg(short, long, conflicts_with = "skip_existing")]
    pub replace_existing: bool,
    /// Add a tag to every imported alias; repeat to add multiple tags
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
}
