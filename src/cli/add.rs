use clap::{Args, Subcommand};

#[derive(Args)]
pub struct AddCommand {
    /// What to add
    #[command(subcommand)]
    pub target: AddTarget,
}

#[derive(Subcommand)]
pub enum AddTarget {
    /// Add a new alias
    #[command(visible_alias = "a")]
    Alias(AddAliasArgs),

    /// Create a new group
    #[command(visible_alias = "g")]
    Group(AddGroupArgs),
}

#[derive(Args)]
pub struct AddAliasArgs {
    /// Name of the alias to create
    #[arg()]
    pub name: String,

    /// Command aliased
    #[arg()]
    pub command: String,

    /// Add alias to GROUP
    #[arg(short, long, value_name = "GROUP")]
    pub group: Option<String>,

    /// Add alias as disabled
    #[arg(short, long, default_value_t = false)]
    pub disabled: bool,

    /// Add alias as a global alias
    #[arg(short, long, default_value_t = false)]
    pub global: bool,
}

#[derive(Args)]
pub struct AddGroupArgs {
    /// Name of the group to create
    #[arg()]
    pub name: String,

    /// Create group as disabled
    #[arg(short, long, default_value_t = false)]
    pub disabled: bool,
}
