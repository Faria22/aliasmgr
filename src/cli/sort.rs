use clap::{Args, Subcommand};

#[derive(Args)]
pub struct SortCommand {
    /// What to sort
    #[command(subcommand)]
    pub target: SortTarget,
}

#[derive(Subcommand)]
pub enum SortTarget {
    /// Sort aliases
    #[command(visible_alias = "a")]
    Aliases(SortAliasesArgs),

    /// Sort groups
    #[command(visible_alias = "g")]
    Groups,
}

#[derive(Args)]
pub struct SortAliasesArgs {
    /// Sort aliases in GROUP.
    /// If not specified, sorts all aliases.
    /// If specified, but left empty, sorts ungrouped aliases.
    #[arg(short, long, value_name = "GROUP")]
    pub group: Option<String>,
}
