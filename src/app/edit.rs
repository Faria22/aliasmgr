use crate::cli::edit::EditCommand;
use crate::config::types::{Alias, Config};
use crate::core::edit::edit_alias;
use crate::core::{Failure, Outcome};

pub fn handle_edit(config: &mut Config, cmd: EditCommand) -> Result<Outcome, Failure> {
    let mut new_alias = Alias::new("".into(), None, true, false); // Default initialization

    if let Some(old_alias) = config.aliases.get(&cmd.name) {
        new_alias = old_alias.clone();
        new_alias.command = cmd.new_command;
    };

    // If no old alias found, edit_alias will return the appropriate error, and we can just use the
    // default new_alias as a placeholder.
    edit_alias(config, &cmd.name, &new_alias)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::types::Alias;

    #[test]
    fn test_handle_edit_success() {
        let mut config = Config::new();
        config.aliases.insert(
            "test".into(),
            Alias::new("original_command".into(), None, true, false),
        );
        let cmd = EditCommand {
            name: "test".into(),
            new_command: "edited_command".into(),
        };
        let result = handle_edit(&mut config, cmd);
        assert!(result.is_ok());
        let edited_alias = config.aliases.get("test").unwrap();
        assert_eq!(edited_alias.command, "edited_command");
    }

    #[test]
    fn test_handle_edit_nonexistent() {
        let mut config = Config::new();
        let cmd = EditCommand {
            name: "nonexistent".into(),
            new_command: "edited_command".into(),
        };
        let result = handle_edit(&mut config, cmd);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Failure::AliasDoesNotExist));
    }
}
