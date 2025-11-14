use clap::{ArgGroup, Args};

#[derive(Args)]
#[command(
    group(
        ArgGroup::new("list_scope")
            .args(["group", "all"])
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
}
