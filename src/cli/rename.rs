use super::validate_tag;
use clap::{Args, Subcommand};

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub struct RenameCommand {
    #[command(subcommand)]
    pub target: Option<RenameTarget>,
    #[arg(required = true)]
    pub old_name: Option<String>,
    #[arg(required = true)]
    pub new_name: Option<String>,
}

#[derive(Subcommand)]
pub enum RenameTarget {
    #[command(visible_alias = "a")]
    Alias(RenameArgs),
    #[command(visible_alias = "t")]
    Tag(TagRenameArgs),
}

#[derive(Args)]
pub struct RenameArgs {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Args)]
pub struct TagRenameArgs {
    #[arg(value_parser = validate_tag)]
    pub old_name: String,
    #[arg(value_parser = validate_tag)]
    pub new_name: String,
}
