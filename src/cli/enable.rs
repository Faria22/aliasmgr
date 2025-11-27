use clap::{Args, Subcommand};

#[derive(Args)]
pub struct EnableCommand {
    /// What to enable
    #[command(subcommand)]
    pub target: EnableTarget,
}

#[derive(Subcommand)]
pub enum EnableTarget {
    /// Enable an alias
    #[command(visible_alias = "a")]
    Alias(EnableArgs),

    /// Enable a group
    #[command(visible_alias = "g")]
    Group(EnableArgs),
}

#[derive(Args)]
pub struct EnableArgs {
    // name
    #[arg()]
    pub name: String,
}
