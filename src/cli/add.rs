use clap::Args;

use super::validate_tag;

#[derive(Args)]
pub struct AddCommand {
    pub name: String,
    pub command: String,
    #[arg(long)]
    pub description: Option<String>,
    /// Add a tag; repeat to add multiple tags
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
    #[arg(short, long)]
    pub disabled: bool,
    #[arg(long)]
    pub global: bool,
}
