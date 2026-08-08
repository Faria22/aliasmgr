use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    subcommand_help_heading = "Explicit resources",
    subcommand_value_name = "RESOURCE"
)]
pub struct RemoveCommand {
    /// Explicit resource type to remove
    #[command(subcommand)]
    pub target: Option<RemoveTarget>,

    /// Name of the alias or group to remove
    #[arg(required = true)]
    pub name: Option<String>,
}

#[derive(Subcommand)]
pub enum RemoveTarget {
    /// Remove an alias
    #[command(visible_alias = "a")]
    Alias(RemoveAliasArgs),

    /// Remove a group and all its aliases
    #[command(visible_alias = "g")]
    Group(GroupRemoveArgs),

    /// Remove all aliases and groups
    All,
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
    #[arg(short, long, default_value_t = false, requires("name"))]
    pub reassign: bool,
}
