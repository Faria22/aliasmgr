use crate::app::shell::ShellType;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitCommand {
    /// Shell type
    #[arg(value_enum, ignore_case = true)]
    pub shell: ShellType,

    /// Custom location of the alias catalog file
    #[arg(long)]
    pub catalog: Option<PathBuf>,

    /// Custom location of the aliasmgr configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Do not synchronize aliases automatically before each prompt
    #[arg(long, default_value_t = false)]
    pub no_auto_sync: bool,
}
