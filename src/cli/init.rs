use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct InitCommand {
    /// Shell type
    #[arg(value_parser = ["bash", "zsh"])]
    pub shell: String,

    /// Custom location of the configuration file
    #[arg(long, hide = true)]
    pub config: Option<PathBuf>,
}
