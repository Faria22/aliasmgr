use clap::Args;

#[derive(Args)]
pub struct EditCommand {
    /// Name of the alias to edit
    pub name: String,

    /// Replacement command
    pub new_command: String,

    /// Toggle enable/disable status
    #[arg(long, short = 'e')]
    pub toggle_enable: bool,

    /// Toggle global status
    #[arg(long, short = 'b')]
    pub toggle_global: bool,

    /// Change alias group. If left empty, removes the alias from any group.
    #[arg(long, short)]
    pub group: Option<Option<String>>,
}
