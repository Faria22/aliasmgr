use clap::Args;

#[derive(Args)]
pub struct EditCommand {
    /// Name of the alias to edit
    #[arg()]
    pub name: String,
    /// Replacement command
    #[arg()]
    pub new_command: String,
}
