use super::selector::AliasSelectorArgs;
use super::validate_tag;
use clap::{Args, Subcommand};

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub struct RemoveCommand {
    #[command(subcommand)]
    pub target: Option<RemoveTarget>,
    #[arg(required = true)]
    pub name: Option<String>,
}

#[derive(Subcommand)]
pub enum RemoveTarget {
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
    /// Remove every alias carrying the tag instead of detaching it
    #[arg(long)]
    pub aliases: bool,
}
