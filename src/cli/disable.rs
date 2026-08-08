use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    subcommand_help_heading = "Explicit resources",
    subcommand_value_name = "RESOURCE"
)]
pub struct DisableCommand {
    /// Explicit resource type to disable
    #[command(subcommand)]
    pub target: Option<DisableTarget>,

    /// Name of the alias or group to disable
    #[arg(required = true)]
    pub name: Option<String>,
}

#[derive(Subcommand)]
pub enum DisableTarget {
    /// Disable an alias
    #[command(visible_alias = "a")]
    Alias(DisableArgs),

    /// Disable a group
    #[command(visible_alias = "g")]
    Group(DisableArgs),
}

#[derive(Args)]
pub struct DisableArgs {
    // name
    #[arg()]
    pub name: String,
}
