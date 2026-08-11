use super::selector::AliasSelectorArgs;
use super::validate_tag;
use clap::{Args, Subcommand};

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub struct EnableCommand {
    #[command(subcommand)]
    pub target: Option<EnableTarget>,
    #[arg(required = true)]
    pub name: Option<String>,
}

#[derive(Subcommand)]
pub enum EnableTarget {
    #[command(visible_alias = "a")]
    Alias(AliasSelectorArgs),
    #[command(visible_alias = "t")]
    Tag(TagArgs),
    All,
}

#[derive(Args)]
pub struct TagArgs {
    #[arg(value_parser = validate_tag)]
    pub name: String,
}
