use super::{Failure, Outcome};
use crate::config::types::Config;
use log::error;

pub fn remove_alias(config: &mut Config, name: &str) -> Result<Outcome, Failure> {
    match config.aliases.shift_remove(name) {
        Some(_) => Ok(Outcome::Command(format!("unalias {}", name))),
        None => {
            error!("Alias '{}' does not exist", name);
            Err(Failure::AliasDoesNotExist)
        }
    }
}

pub fn remove_group(config: &mut Config, name: &str) -> Result<Outcome, Failure> {
    match config.aliases.shift_remove(name) {
        Some(_) => Ok(Outcome::ConfigChanged),
        None => {
            error!("Group '{}' does not exist", name);
            Err(Failure::GroupDoesNotExist)
        }
    }
}
