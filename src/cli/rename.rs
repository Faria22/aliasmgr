use clap::{Args, Subcommand};

#[derive(Args)]
pub struct RenameCommand {
    /// What to rename
    #[command(subcommand)]
    pub target: RenameTarget,
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
