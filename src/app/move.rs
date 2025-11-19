use crate::core::{Failure, Outcome};

use crate::core::add::add_group;
use crate::core::r#move::move_alias;

use crate::config::types::Config;

use crate::cli::MoveCommand;
use crate::cli::interaction::create_non_existent_group;

use log::info;

pub fn handle_move(config: &mut Config, command: MoveCommand) -> Result<Outcome, Failure> {
    todo!()
}
