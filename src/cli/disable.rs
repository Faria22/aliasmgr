use clap::{ArgGroup, Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_help_heading = "Additional actions",
    subcommand_value_name = "ACTION",
    group(
        ArgGroup::new("disable_target")
            .args(["name", "all"])
            .required(true)
            .multiple(false)
    )
)]
pub struct DisableCommand {
    /// Name of the alias to disable
    #[arg()]
    pub name: Option<String>,
    /// Disable all aliases
    #[arg(short, long)]
    pub all: bool,
    /// Disable a group of aliases
    #[command(subcommand)]
    pub group: Option<DisableGroupCommand>,
}

#[derive(Subcommand)]
pub enum DisableGroupCommand {
    /// Disable all aliases in a group
    #[command(visible_alias = "g")]
    Group {
        /// Name of the group to disable
        #[arg()]
        name: String,
    },
}
