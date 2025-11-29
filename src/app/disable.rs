use crate::config::types::Config;
use crate::core::disable::{disable_alias, disable_group};
use crate::core::{Failure, Outcome};

use crate::cli::disable::{DisableCommand, DisableTarget};

use super::shell::ShellType;

pub fn handle_disable(
    config: &mut Config,
    cmd: DisableCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        DisableTarget::Alias(args) => disable_alias(config, &args.name),
        DisableTarget::Group(args) => disable_group(config, &args.name, shell),
    }
}
