use clap::Args;

#[derive(Args)]
pub struct MoveCommand {
    /// Name of the alias to move
    #[arg()]
    pub name: String,
    /// New group name. If left blank, the alias will leave the group
    #[arg()]
    pub new_group: Option<String>,
}
