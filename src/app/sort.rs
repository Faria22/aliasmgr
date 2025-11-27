use crate::cli::sort::{SortCommand, SortTarget};
use crate::config::types::Config;
use crate::core::sort::{sort_aliases_in_group, sort_all_aliases, sort_groups};
use crate::core::{Failure, Outcome};

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn handle_sort(config: &mut Config, cmd: SortCommand) -> Result<Outcome, Failure> {
    match &cmd.target {
        SortTarget::Aliases(args) => {
            if let Some(group) = &args.group {
                sort_aliases_in_group(config, group.as_deref())
            } else {
                sort_all_aliases(config)
            }
        }
        SortTarget::Groups => sort_groups(config),
    }
}
