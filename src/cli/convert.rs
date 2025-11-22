use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ConvertCommand {
    /// Source configuration file to convert
    pub source: PathBuf,

    /// Target configuration file
    /// If not provided, the converted configuration will be appended to aliasmgr's configuration file
    pub target: Option<PathBuf>,
}
