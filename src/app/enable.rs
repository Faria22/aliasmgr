use crate::config::types::Config;
use crate::core::enable::{enable_alias, enable_group};
use crate::core::{Failure, Outcome};

use crate::cli::enable::{EnableCommand, EnableTarget};

use super::shell::ShellType;

pub fn handle_enable(
    config: &mut Config,
    cmd: EnableCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        EnableTarget::Alias(args) => enable_alias(config, &args.name),
        EnableTarget::Group(args) => enable_group(config, &args.name, shell),
    }
}
