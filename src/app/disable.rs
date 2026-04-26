use crate::catalog::types::AliasCatalog;
use crate::core::disable::{disable_alias, disable_group};
use crate::core::{Failure, Outcome};

use crate::cli::disable::{DisableCommand, DisableTarget};

use super::shell::ShellType;

pub fn handle_disable(
    catalog: &mut AliasCatalog,
    cmd: DisableCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        DisableTarget::Alias(args) => disable_alias(catalog, &args.name),
        DisableTarget::Group(args) => disable_group(catalog, &args.name, shell),
    }
}
