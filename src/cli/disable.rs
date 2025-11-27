use clap::{Args, Subcommand};

#[derive(Args)]
pub struct DisableCommand {
    /// What to disable
    #[command(subcommand)]
    pub target: DisableTarget,
}

#[derive(Subcommand)]
pub enum DisableTarget {
    /// Disable an alias
    #[command(visible_alias = "a")]
    Alias(DisableArgs),

    /// Disable a group
    #[command(visible_alias = "g")]
    Group(DisableArgs),
}

#[derive(Args)]
pub struct DisableArgs {
    // name
    #[arg()]
    pub name: String,
}
