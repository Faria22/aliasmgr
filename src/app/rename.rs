use crate::cli::rename::{RenameCommand, RenameTarget};
use crate::config::types::Config;
use crate::core::rename::{rename_alias, rename_group};
use crate::core::{Failure, Outcome};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn handle_rename(config: &mut Config, cmd: RenameCommand) -> Result<Outcome, Failure> {
    match cmd.target {
        RenameTarget::Alias(args) => rename_alias(config, &args.old_name, &args.new_name),
        RenameTarget::Group(args) => rename_group(config, &args.old_name, &args.new_name),
    }
}
