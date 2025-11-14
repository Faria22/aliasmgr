use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_help_heading = "Group actions",
    subcommand_value_name = "ACTION"
)]
pub struct AddCommand {
    /// Name of the alias to create
    #[arg()]
    pub name: Option<String>,
    /// Command aliased
    #[arg()]
    pub command: Option<String>,
    /// Add alias to GROUP
    #[arg(short, long, value_name = "GROUP")]
    pub group: Option<String>,
    /// Optional action: create a brand new group
    #[command(subcommand)]
    pub subcommand: Option<AddGroupCommands>,
}

#[derive(Subcommand)]
pub enum AddGroupCommands {
    /// Create a new group
    #[command(visible_alias = "g")]
    Group {
        /// Name of the group to create
        #[arg()]
        name: String,
    },
}
