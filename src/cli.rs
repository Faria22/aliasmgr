use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    version,
    about,
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new alias or alias group
    #[command(visible_alias = "a")]
    Add(AddCommand),

    /// Remove an existing alias or alias group
    #[command(visible_alias = "rm")]
    Remove(RemoveCommand),

    /// List all active aliases
    #[command(visible_alias = "ls")]
    List {
        /// Lists all aliases group
        #[arg(short, long)]
        group: Option<String>,
        /// Lists all aliases
        #[arg(short, long)]
        all: bool,
    },

    /// Enable an alias
    #[command(visible_alias = "en")]
    Enable {
        /// Name of the alias
        name: Option<String>,
        #[arg(short, long)]
        /// Activate  group
        group: Option<String>,
        /// Enable all aliases
        #[arg(short, long)]
        all: bool,
    },

    /// Disable an alias
    #[command(visible_alias = "dis")]
    Disable {
        /// Name of the alias
        name: Option<String>,
        /// Deactivate group
        #[arg(short, long)]
        group: Option<String>,
        /// Disable all aliases
        #[arg(short, long)]
        all: bool,
    },

    /// Rename an existing alias
    #[command(visible_alias = "rn")]
    Rename {
        /// Old name of the alias
        old: String,
        /// New name of the alias
        new: String,
        /// Rename alias group
        #[arg(short, long)]
        group: Option<String>,
        /// Rename group
        #[command(subcommand)]
        group_rename: Option<GroupRename>,
    },

    /// Edit an existing alias
    #[command(visible_alias = "ed")]
    Edit {
        /// Name of the alias
        name: String,
        /// New command for the alias
        new_command: String,
    },

    /// Synchronize aliases with configuration file
    Sync,
}

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_help_heading = "Additional actions",
    subcommand_value_name = "ACTION"
)]
pub struct AddCommand {
    /// Name of the alias (default behavior when NAME is provided)
    #[arg()]
    pub name: Option<String>,
    /// Command the alias represents
    #[arg()]
    pub command: Option<String>,
    /// Add alias to a group
    #[arg(short, long)]
    pub group: Option<String>,
    /// Optional action: creating a group (default action adds an alias)
    #[command(subcommand)]
    pub subcommand: Option<AddGroupCommands>,
}

#[derive(Subcommand)]
pub enum AddGroupCommands {
    /// Create a new group
    #[command(visible_alias = "g")]
    Group {
        /// Name of the group
        name: String,
    },
}

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_help_heading = "Additional actions",
    subcommand_value_name = "ACTION"
)]
pub struct RemoveCommand {
    /// Name of the alias
    #[arg()]
    pub name: Option<String>,
    /// Remove all aliases from a group (the group will still exist)
    #[arg()]
    pub group: Option<String>,
    /// Remove all aliases (together with their groups)
    #[arg()]
    pub all: bool,
    /// Optional action: creating a group (default action adds an alias)
    #[command(subcommand)]
    pub subcommand: Option<RemoveGroupCommands>,
}

#[derive(Subcommand)]
pub enum RemoveGroupCommands {
    /// Remove a group
    #[command(visible_alias = "g")]
    Group {
        /// Name of the group
        name: String,
    },
}

#[derive(Subcommand)]
pub enum GroupRename {
    /// Rename a group
    Group {
        /// Old name of the group
        old: String,
        /// New name of the group
        new: String,
    },
}

#[derive(Subcommand)]
pub enum GroupRemove {
    /// Remove a group
    Group {
        /// Name of the group
        name: String,
    },
}
