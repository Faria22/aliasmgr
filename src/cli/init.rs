use crate::app::shell::ShellType;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitCommand {
    /// Shell type
    #[arg(value_enum, ignore_case = true)]
    pub shell: ShellType,

    /// Custom location of the configuration file
    #[arg(long)]
    pub config: Option<PathBuf>,
}
