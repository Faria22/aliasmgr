use crate::cli::list::ListCommand;
use crate::config::types::Config;
use crate::core::{Failure, Outcome};

use crate::core::list::{GroupId, get_single_group};

pub fn handle_list(config: &Config, cmd: ListCommand) -> Result<Outcome, Failure> {
    if let Some(group) = cmd.group {
        match get_single_group(config, GroupId::Named(group.clone())) {
            Err(Failure::GroupDoesNotExist) => {
                eprintln!("Group '{}' does not exist.", group);
                Err(Failure::GroupDoesNotExist)
            }
            Ok(aliases) => {
                todo!()
            }
            Err(e) => unreachable!("unexpected error: {:?}", e),
        }
    } else if cmd.all {
        todo!()
    } else if cmd.disabled {
        todo!()
    } else {
        todo!()
    }
}
