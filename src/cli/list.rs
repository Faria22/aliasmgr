use clap::{ArgGroup, Args};

#[derive(Args)]
#[command(
    group(
        ArgGroup::new("list_scope")
            .args(["group", "ungrouped", "enabled", "disabled"])
            .multiple(false)
    )
)]
pub struct ListCommand {
    /// List aliases in GROUP
    #[arg(short, long, value_name = "GROUP")]
    pub group: Option<String>,

    /// List all aliases not in any group
    #[arg(short, long)]
    pub ungrouped: bool,

    /// List only enabled aliases
    #[arg(short, long)]
    pub enabled: bool,

    /// Show only disabled aliases
    #[arg(short, long)]
    pub disabled: bool,
}
