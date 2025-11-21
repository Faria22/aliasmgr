use clap::{ArgGroup, Args};

#[derive(Args)]
#[command(
    group(
        ArgGroup::new("list_scope")
            .args(["group", "all", "disabled"])
            .multiple(false)
    )
)]
pub struct ListCommand {
    /// List aliases in GROUP
    #[arg(short, long, value_name = "GROUP")]
    pub group: Option<String>,

    /// List all aliases
    #[arg(short, long)]
    pub all: bool,

    /// Show only disabled aliases
    #[arg(short, long)]
    pub disabled: bool,
}
