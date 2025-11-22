use clap::{Args, Subcommand};

#[derive(Args)]
pub struct RemoveCommand {
    /// What to remove
    #[command(subcommand)]
    pub target: RemoveTarget,
}

#[derive(Subcommand)]
pub enum RemoveTarget {
    /// Remove an alias
    #[command(visible_alias = "a")]
    Alias(RemoveAliasArgs),

    /// Remove a group and all its aliases
    #[command(visible_alias = "g")]
    Group(GroupRemoveArgs),
}

#[derive(Args)]
pub struct RemoveAliasArgs {
    /// Name of the alias to remove
    #[arg()]
    pub name: String,
}

#[derive(Args)]
pub struct GroupRemoveArgs {
    /// Name of the group to remove. If not provided, all the aliases without a group will be removed.
    #[arg()]
    pub name: Option<String>,

    /// Removes the group, but moves all its aliases to `ungrouped`
    #[arg(short, long, default_value_t = false)]
    pub reassign: bool,
}
