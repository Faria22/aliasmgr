use clap::{ArgGroup, Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_help_heading = "Group actions",
    subcommand_value_name = "ACTION",
    group(
        ArgGroup::new("remove_target")
            .args(["name", "group", "all"])
            .required(true)
            .multiple(false)
    )
)]
pub struct RemoveCommand {
    /// Name of the alias to remove
    #[arg()]
    pub name: Option<String>,
    /// Remove all aliases and their groups
    #[arg(short, long)]
    pub all: bool,
    /// Optional action: remove a group entirely
    #[command(subcommand)]
    pub subcommand: Option<GroupRemove>,
}

#[derive(Subcommand)]
pub enum GroupRemove {
    /// Remove a group and all its aliases
    #[command(visible_alias = "g")]
    Group {
        /// Name of the group to remove
        #[arg()]
        name: String,
    },
}
