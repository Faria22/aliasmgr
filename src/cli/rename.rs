use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_help_heading = "Additional actions",
    subcommand_value_name = "ACTION"
)]
pub struct RenameCommand {
    /// Old alias name
    #[arg()]
    pub old: String,
    /// New alias name
    #[arg()]
    pub new: String,
    /// Rename a group instead of an alias
    #[command(subcommand)]
    pub group_rename: Option<GroupRename>,
}

#[derive(Subcommand)]
pub enum GroupRename {
    /// Rename a group
    #[command(visible_alias = "g")]
    Group {
        /// Old group name
        #[arg()]
        old: String,
        /// New group name
        #[arg()]
        new: String,
    },
}
