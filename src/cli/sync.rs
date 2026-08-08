use clap::Args;

#[derive(Args)]
pub struct SyncCommand {}

#[derive(Args)]
pub struct ShellSyncCommand {
    /// Skip reconciliation when this terminal already has the current revision
    #[arg(long, conflicts_with = "force")]
    pub if_changed: bool,

    /// Reconcile even when this terminal already has the current revision
    #[arg(long, conflicts_with = "if_changed")]
    pub force: bool,
}
