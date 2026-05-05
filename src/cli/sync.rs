use clap::Args;

#[derive(Args)]
pub struct SyncCommand {
    #[arg(long, hide = true)]
    pub startup: bool,
}
