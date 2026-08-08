use clap::{Args, Subcommand};

#[derive(Args)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true,
    subcommand_help_heading = "Explicit resources",
    subcommand_value_name = "RESOURCE"
)]
pub struct AddCommand {
    /// Explicit resource type to add
    #[command(subcommand)]
    pub target: Option<AddTarget>,

    #[command(flatten)]
    pub alias: ShorthandAddAliasArgs,
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

#[derive(Args, Default)]
pub struct ShorthandAddAliasArgs {
    /// Name of the alias to create
    #[arg(required = true)]
    pub name: Option<String>,

    /// Command aliased
    #[arg(required = true)]
    pub command: Option<String>,

    /// Add alias to GROUP
    #[arg(short, long, value_name = "GROUP")]
    pub group: Option<String>,

    /// Add alias as disabled
    #[arg(short, long, default_value_t = false)]
    pub disabled: bool,

    /// Add alias as a global alias
    #[arg(long, default_value_t = false)]
    pub global: bool,
}

impl ShorthandAddAliasArgs {
    pub fn into_alias_args(self) -> AddAliasArgs {
        AddAliasArgs {
            name: self
                .name
                .expect("clap requires an alias name when no subcommand is used"),
            command: self
                .command
                .expect("clap requires an alias command when no subcommand is used"),
            group: self.group,
            disabled: self.disabled,
            global: self.global,
        }
    }
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
    #[arg(long, default_value_t = false)]
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
