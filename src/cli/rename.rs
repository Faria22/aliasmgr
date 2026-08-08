use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    subcommand_help_heading = "Explicit resources",
    subcommand_value_name = "RESOURCE"
)]
pub struct RenameCommand {
    /// Explicit resource type to rename
    #[command(subcommand)]
    pub target: Option<RenameTarget>,

    /// Current alias or group name
    #[arg(required = true)]
    pub old_name: Option<String>,

    /// New alias or group name
    #[arg(required = true)]
    pub new_name: Option<String>,
}

#[derive(Subcommand)]
pub enum RenameTarget {
    /// Rename an existing alias
    #[command(visible_alias = "a")]
    Alias(RenameArgs),

    /// Rename an existing group
    #[command(visible_alias = "g")]
    Group(RenameArgs),
}

#[derive(Args)]
pub struct RenameArgs {
    /// Current name
    #[arg()]
    pub old_name: String,

    /// New name
    #[arg()]
    pub new_name: String,
}
