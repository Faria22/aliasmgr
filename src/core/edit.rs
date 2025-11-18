use crate::config::types::Config;

enum EditError {
    AliasDoesNotExist,
}

pub fn edit_alias(config: &mut Config, name: &str, new_command: &str) {
    if let Err(e) = edit_alias_in_config(config, name, new_command) {
        match e {
            EditError::AliasDoesNotExist => {
                eprintln!("Error: Alias '{}' does not exist.", name);
            }
        }
    }
}

fn edit_alias_in_config(
    config: &mut Config,
    name: &str,
    new_command: &str,
) -> Result<(), EditError> {
    if !config.aliases.contains_key(name) {
        return Err(EditError::AliasDoesNotExist);
    }

    config.aliases.get_mut(name).unwrap().command = new_command.into();

    Ok(())
}
