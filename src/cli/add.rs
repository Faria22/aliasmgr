use clap::Args;

use super::validate_tag;

#[derive(Args)]
pub struct AddCommand {
    pub name: String,
    pub command: String,
    /// Create a Zsh global alias
    #[arg(short, long)]
    pub global: bool,
    /// Add a tag; repeat to add multiple tags
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
    /// Describe what the alias does
    #[arg(short, long)]
    pub description: Option<String>,
    /// Create the alias in a disabled state
    #[arg(long)]
    pub disabled: bool,
}
