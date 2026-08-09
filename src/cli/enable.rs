use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    subcommand_help_heading = "Explicit resources",
    subcommand_value_name = "RESOURCE"
)]
pub struct EnableCommand {
    /// Explicit resource type to enable
    #[command(subcommand)]
    pub target: Option<EnableTarget>,

    /// Name of the alias or group to enable
    #[arg(required = true)]
    pub name: Option<String>,
}

#[derive(Subcommand)]
pub enum EnableTarget {
    /// Enable an alias
    #[command(visible_alias = "a")]
    Alias(EnableArgs),

    /// Enable a group
    #[command(visible_alias = "g")]
    Group(EnableArgs),

    /// Enable all aliases and groups
    All,
}

#[derive(Args)]
pub struct EnableArgs {
    // name
    #[arg()]
    pub name: String,
}
