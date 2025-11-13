use clap::{Parser, Subcommand};

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
    /// Add a new alias
    #[command(visible_alias = "a")]
    Add {
        // /// Name of the alias
        // name: String,
        // /// Command the alias represents
        // command: String,
        // /// Add alias to a  group
        // #[arg(short, long)]
        // group: Option<String>,
        /// Create a new alias group
        #[command(subcommand)]
        command: AddCommands,
    },
    /// Remove an existing alias
    #[command(visible_alias = "rm")]
    Remove {
        /// Name of the alias
        name: Option<String>,
        /// Remove all aliases from a group (the group will still exist)
        #[arg(short, long)]
        group: Option<String>,
        /// Remove all aliases (together with their groups)
        #[arg(short, long)]
        all: bool,
        /// Remove an alias group
        #[command(subcommand)]
        group_remove: Option<GroupRemove>,
    },
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
        name: String,
        old_command: String,
        new_command: String,
    },
    /// Synchronize aliases with configuration file
    Sync,
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
pub enum AddCommands {
    /// Add a new alias
    Alias {
        /// Name of the alias
        name: String,
        /// Command the alias represents
        command: String,
        /// Add alias to a  group
        #[arg(short, long)]
        group: Option<String>,
    },
    /// Create a new group
    #[command(visible_alias = "g")]
    Group {
        /// Name of the group
        name: String,
    },
}

// #[derive(Subcommand)]
// pub enum GroupCreate {
//     /// Create a new group
//     Group {
//         /// Name of the group
//         name: String,
//     },
// }

#[derive(Subcommand)]
pub enum GroupRemove {
    /// Remove a group
    Group {
        /// Name of the group
        name: String,
    },
}
