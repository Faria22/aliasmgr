use clap::{ArgGroup, Args};

#[derive(Args)]
#[command(
    group(
        ArgGroup::new("list_scope")
            .args(["enabled", "disabled"])
            .multiple(false)
    )
)]
pub struct ListCommand {
    /// List alias by name pattern
    pub pattern: Option<String>,

    /// List aliases in GROUP. If left empty, list ungrouped aliases.
    #[arg(short, long, num_args=0..=1, value_name = "GROUP")]
    pub group: Option<Option<String>>,

    /// List only enabled aliases
    #[arg(short, long)]
    pub enabled: bool,

    /// Show only disabled aliases
    #[arg(short, long)]
    pub disabled: bool,

    /// Show only global aliases
    #[arg(long)]
    pub global: bool,
}
