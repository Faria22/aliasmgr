use owo_colors::OwoColorize;

use crate::cli::list::ListCommand;
use crate::config::types::Config;
use crate::core::list::{
    GroupId, get_all_aliases_grouped, get_disabled_aliases_grouped, get_enabled_aliases_grouped,
    get_single_group,
};
use crate::core::{Failure, Outcome};

/// Returns a colored symbol representing the enabled status.
fn enabled_symbol(enabled: bool) -> String {
    if enabled {
        "✔".green().bold().to_string()
    } else {
        "✘".red().bold().to_string()
    }
}

fn globe_symbol(global: bool) -> String {
    if global {
        // " 🌐 ".to_string()
        " ⦾".blue().bold().to_string()
    } else {
        String::new()
    }
}

/// Formats the information of a single alias.
pub fn format_alias_info(config: &Config, alias: &str) -> Result<String, Failure> {
    if let Some(alias_info) = config.aliases.get(alias) {
        Ok(format!(
            "{}{} {} -> {}",
            enabled_symbol(alias_info.enabled),
            globe_symbol(alias_info.global),
            alias,
            alias_info.command
        ))
    } else {
        eprintln!("Alias '{}' not found in configuration.", alias);
        Err(Failure::AliasDoesNotExist)
    }
}

/// Generates a header string for a group of aliases.
fn group_header(config: &Config, group: &GroupId) -> Result<String, Failure> {
    let group_enabled;
    let group_name;
    if let GroupId::Named(g) = group {
        match config.groups.get(g) {
            Some(enabled) => {
                group_enabled = enabled;
                group_name = g.clone();
            }
            None => {
                eprintln!("Group '{}' does not exist in configuration.", g);
                return Err(Failure::GroupDoesNotExist);
            }
        }
    } else {
        // Ungrouped aliases are always considered enabled
        group_enabled = &true;
        group_name = "ungrouped".to_string();
    }

    let header_message = format!(
        " Group: {} {} ",
        &group_name,
        enabled_symbol(*group_enabled)
    );
    Ok(format!("{:=^width$}", header_message, width = 50))
}

/// Formats a group header along with its aliases.
fn format_group_and_aliases(
    config: &Config,
    group_id: &GroupId,
    aliases: &Vec<String>,
) -> Result<String, Failure> {
    let mut content = String::new();
    content += &(group_header(config, group_id)? + "\n");
    for alias in aliases {
        content += &(format_alias_info(config, alias)? + "\n");
    }
    Ok(content)
}

/// Formats a list of aliases without a group header.
fn format_aliases_list(config: &Config, aliases: &Vec<String>) -> Result<String, Failure> {
    let mut content = String::new();
    for alias in aliases {
        content += &(format_alias_info(config, alias)? + "\n");
    }
    Ok(content)
}

/// Handle the 'list' command based on the provided options.
/// This function lists aliases according to the specified criteria:
/// - If a specific group is provided, it lists aliases in that group.
/// - If the 'all' flag is set, it lists all aliases.
/// - If the 'disabled' flag is set, it lists only disabled aliases.
/// - By default, it lists only enabled aliases.
///
/// # Arguments
/// - `config`: Reference to the configuration containing aliases and groups.
/// - `cmd`: The ListCommand containing options for listing.
///
/// # Returns
/// - `Outcome::NoChanges` if the operation is successful.
/// - `Failure::GroupDoesNotExist` if the specified group does not exist.
/// - Other failures as defined in the `Failure` enum.
pub fn handle_list(config: &Config, cmd: ListCommand) -> Result<Outcome, Failure> {
    if let Some(group) = cmd.group {
        // List aliases in a specific group
        match get_single_group(config, GroupId::Named(group.clone())) {
            Err(Failure::GroupDoesNotExist) => {
                eprintln!("Group '{}' does not exist.", group);
                Err(Failure::GroupDoesNotExist)
            }
            Ok(aliases) => {
                let group_id = GroupId::Named(group);
                print!("{}", format_group_and_aliases(config, &group_id, &aliases)?);
                Ok(Outcome::NoChanges)
            }
            Err(e) => unreachable!("unexpected error: {:?}", e),
        }
    } else if cmd.ungrouped {
        // List ungrouped aliases
        match get_single_group(config, GroupId::Ungrouped) {
            Ok(aliases) => {
                print!("{}", format_aliases_list(config, &aliases)?);
                Ok(Outcome::NoChanges)
            }
            Err(e) => unreachable!("ungrouped 'group' should always exist. Error: {:?}", e),
        }
    } else if cmd.enabled {
        // List only the enabled aliases
        for (group_id, aliases) in get_enabled_aliases_grouped(config) {
            print!("{}", format_group_and_aliases(config, &group_id, &aliases)?);
        }
        Ok(Outcome::NoChanges)
    } else if cmd.disabled {
        // List disabled aliases
        for (group_id, aliases) in get_disabled_aliases_grouped(config) {
            print!("{}", format_group_and_aliases(config, &group_id, &aliases)?);
        }
        Ok(Outcome::NoChanges)
    } else {
        // Default: list enabled aliases
        for (group_id, aliases) in get_all_aliases_grouped(config) {
            print!("{}", format_group_and_aliases(config, &group_id, &aliases)?);
        }
        Ok(Outcome::NoChanges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Alias, Config};
    use assert_matches::assert_matches;

    fn create_test_config() -> Config {
        let mut config = Config::new();
        // Ungrouped alias
        config.aliases.insert(
            "test".to_string(),
            Alias::new("echo test".to_string(), None, true, false),
        );
        // Grouped alias
        config.aliases.insert(
            "build".to_string(),
            Alias::new(
                "cargo build".to_string(),
                Some("dev".to_string()),
                true,
                false,
            ),
        );
        config.groups.insert("dev".to_string(), true);
        config
    }

    #[test]
    fn test_enabled_symbol() {
        assert_eq!(enabled_symbol(true), "✔".green().bold().to_string());
        assert_eq!(enabled_symbol(false), "✘".red().bold().to_string());
    }

    #[test]
    fn test_print_alias_valid() {
        let config = create_test_config();

        let result = format_alias_info(&config, "test");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{} test -> echo test", enabled_symbol(true))
        );
    }

    #[test]
    fn test_group_header_valid() {
        let config = create_test_config();

        let result = group_header(&config, &GroupId::Named("dev".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Group: dev"));
    }

    #[test]
    fn test_format_group_and_aliases_valid() {
        let config = create_test_config();

        let aliases = vec!["test".to_string()];
        let result =
            format_group_and_aliases(&config, &GroupId::Named("dev".to_string()), &aliases);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Group: dev"));
        assert!(output.contains("test -> echo test"));
    }

    #[test]
    fn test_handle_list_specific_existing_group() {
        let config = create_test_config();

        let cmd = ListCommand {
            group: Some("dev".to_string()),
            ungrouped: false,
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_specific_nonexistent_group() {
        let config = create_test_config();
        let cmd = ListCommand {
            group: Some("nonexistent".to_string()),
            ungrouped: false,
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd);
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_handle_list_all() {
        let config = create_test_config();
        let cmd = ListCommand {
            group: None,
            ungrouped: false,
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_enabled() {
        let config = create_test_config();
        let cmd = ListCommand {
            group: None,
            ungrouped: false,
            enabled: true,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_disabled() {
        let config = create_test_config();
        let cmd = ListCommand {
            group: None,
            ungrouped: false,
            enabled: false,
            disabled: true,
            global: false,
        };
        let result = handle_list(&config, cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_no_aliases() {
        let config = Config::new();
        let cmd = ListCommand {
            group: None,
            ungrouped: false,
            enabled: true,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_ungrouped() {
        let config = create_test_config();
        let cmd = ListCommand {
            group: None,
            ungrouped: true,
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_global() {
        let config = create_test_config();
        let cmd = ListCommand {
            group: None,
            ungrouped: false,
            enabled: false,
            disabled: false,
            global: true,
        };
        let result = handle_list(&config, cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_alias_info_nonexistent_alias() {
        let config = create_test_config();
        let result = format_alias_info(&config, "nonexistent");
        assert_matches!(result, Err(Failure::AliasDoesNotExist));
    }

    #[test]
    fn test_group_header_nonexistent_group() {
        let config = create_test_config();
        let result = group_header(&config, &GroupId::Named("nonexistent".to_string()));
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_format_group_and_aliases_nonexistent_group() {
        let config = create_test_config();
        let aliases = vec!["test".to_string()];
        let result = format_group_and_aliases(
            &config,
            &GroupId::Named("nonexistent".to_string()),
            &aliases,
        );
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_format_group_and_aliases_nonexistent_alias() {
        let config = create_test_config();
        let aliases = vec!["nonexistent".to_string()];
        let result =
            format_group_and_aliases(&config, &GroupId::Named("dev".to_string()), &aliases);
        assert!(matches!(result, Err(Failure::AliasDoesNotExist)));
    }

    #[test]
    fn test_format_aliases_list_nonexistent_alias() {
        let config = create_test_config();
        let aliases = vec!["nonexistent".to_string()];
        let result = format_aliases_list(&config, &aliases);
        assert!(matches!(result, Err(Failure::AliasDoesNotExist)));
    }

    #[test]
    fn test_global_symbol() {
        assert_eq!(globe_symbol(true), " ⦾".blue().bold().to_string());
        assert_eq!(globe_symbol(false), "".to_string());
    }
}
