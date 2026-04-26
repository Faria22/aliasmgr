use crate::catalog::types::AliasCatalog;
use crate::core::enable::{enable_alias, enable_group};
use crate::core::{Failure, Outcome};

use crate::cli::enable::{EnableCommand, EnableTarget};

use super::shell::ShellType;

pub fn handle_enable(
    catalog: &mut AliasCatalog,
    cmd: EnableCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    match cmd.target {
        EnableTarget::Alias(args) => enable_alias(catalog, &args.name),
        EnableTarget::Group(args) => enable_group(catalog, &args.name, shell),
    }
}
