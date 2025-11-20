use crate::core::{Failure, Outcome};

use crate::core::add::add_group;
use crate::core::r#move::move_alias;

use crate::config::types::Config;

use crate::cli::MoveCommand;
use crate::cli::interaction::create_non_existent_group;

use log::info;

pub fn handle_move(config: &mut Config, cmd: MoveCommand) -> Result<Outcome, Failure> {
    match move_alias(config, &cmd.name, &cmd.new_group) {
        Ok(outcome) => Ok(outcome),
        Err(e) => match e {
            Failure::GroupDoesNotExist => {
                let new_group = cmd
                    .new_group
                    .expect("new_group has to be `Some` for this error to arise");
                if create_non_existent_group(&new_group) {
                    info!("Created new group '{}'", new_group);
                    add_group(config, &new_group, true)?;
                    move_alias(config, &cmd.name, &Some(new_group))
                } else {
                    info!(
                        "User aborted moving alias '{}' to non-existent group '{}'",
                        &cmd.name, &new_group
                    );
                    Ok(Outcome::NoChanges)
                }
            }
            _ => Err(e),
        },
    }
}
