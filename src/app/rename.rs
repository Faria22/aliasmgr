use crate::catalog::types::AliasCatalog;
use crate::cli::rename::{RenameCommand, RenameTarget};
use crate::core::rename::{rename_alias, rename_group};
use crate::core::{Failure, Outcome};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn handle_rename(catalog: &mut AliasCatalog, cmd: RenameCommand) -> Result<Outcome, Failure> {
    match cmd.target {
        RenameTarget::Alias(args) => rename_alias(catalog, &args.old_name, &args.new_name),
        RenameTarget::Group(args) => rename_group(catalog, &args.old_name, &args.new_name),
    }
}
