use clap::Args;

use super::validate_tag;

#[derive(Args)]
pub struct AddCommand {
    pub name: String,
    pub command: String,
    /// Describe what the alias does
    #[arg(short = 'd', long)]
    pub description: Option<String>,
    /// Add a tag; repeat to add multiple tags
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
    /// Create the alias in a disabled state
    #[arg(long)]
    pub disabled: bool,
    /// Create a Zsh global alias
    #[arg(short = 'g', long)]
    pub global: bool,
}
