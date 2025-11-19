use clap::Args;

#[derive(Args)]
pub struct MoveCommand {
    /// Name of the alias to move
    #[arg()]
    pub name: String,
    /// New group name
    #[arg()]
    pub new_group: String,
}
