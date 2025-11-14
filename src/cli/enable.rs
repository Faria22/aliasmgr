use clap::{ArgGroup, Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_help_heading = "Additional actions",
    subcommand_value_name = "ACTION",
    group(
        ArgGroup::new("enable_target")
            .args(["name", "all"])
            .required(true)
            .multiple(false)
    )
)]
pub struct EnableCommand {
    /// Name of the alias to enable
    #[arg()]
    pub name: Option<String>,
    /// Enable all alias
    #[arg(short, long)]
    pub all: bool,
    /// Enable a specific group of aliases
    #[command(subcommand)]
    pub group: Option<EnableGroupCommand>,
}

#[derive(Subcommand)]
pub enum EnableGroupCommand {
    /// Enable all aliases in a group
    #[command(visible_alias = "g")]
    Group {
        /// Name of the group to enable
        #[arg()]
        name: String,
    },
}
