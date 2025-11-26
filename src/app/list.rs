use owo_colors::OwoColorize;

use super::shell::ShellType;
use crate::cli::list::ListCommand;
use crate::config::types::Config;
use crate::core::list::{get_all_aliases_grouped, get_single_group};
use crate::core::{Failure, Outcome};

use globset::Glob;

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
fn group_header(config: &Config, group: &Option<String>) -> Result<String, Failure> {
    let group_enabled;
    let group_name;
    if let Some(g) = group {
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
    group_id: &Option<String>,
    aliases: &Vec<String>,
) -> Result<String, Failure> {
    let mut content = String::new();
    content += &(group_header(config, group_id)? + "\n");
    content += &format_aliases_list(config, aliases)?;
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

/// If ungrouped, will remove the group header
fn format_group_and_aliases_single_group(
    config: &Config,
    group_id: &Option<String>,
    aliases: &Vec<String>,
) -> Result<String, Failure> {
    let mut content = String::new();
    if group_id.is_some() {
        content += &(group_header(config, group_id)? + "\n");
    }
    content += &format_aliases_list(config, aliases)?;
    Ok(content)
}

fn retain_aliases(config: &Config, aliases: &mut Vec<String>, cmd: &ListCommand) {
    if let Some(pattern) = &cmd.pattern {
        let glob = Glob::new(pattern).unwrap().compile_matcher();
        aliases.retain(|alias| glob.is_match(alias));
    }
    if cmd.enabled {
        aliases.retain(|alias| config.aliases[alias].enabled);
    } else if cmd.disabled {
        aliases.retain(|alias| !config.aliases[alias].enabled);
    }

    if cmd.global {
        aliases.retain(|alias| config.aliases[alias].global);
    }
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
pub fn handle_list(
    config: &Config,
    cmd: ListCommand,
    shell: &ShellType,
) -> Result<Outcome, Failure> {
    // List aliases in a specific group
    if let Some(group) = &cmd.group {
        // User provided a group name
        let group_id;
        if let Some(group_name) = group {
            group_id = Some(group_name.clone())
        } else {
            // User wants ungrouped aliases
            group_id = None;
        };

        let mut aliases = get_single_group(config, &group_id, shell)?;
        retain_aliases(config, &mut aliases, &cmd);
        print!(
            "{}",
            format_group_and_aliases_single_group(config, &group_id, &aliases)?
        );
        Ok(Outcome::NoChanges)
    } else {
        // Default: list enabled aliases
        for (group_id, mut aliases) in get_all_aliases_grouped(config, shell) {
            retain_aliases(config, &mut aliases, &cmd);
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

        let result = group_header(&config, &Some("dev".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Group: dev"));
    }

    #[test]
    fn test_format_group_and_aliases_valid() {
        let config = create_test_config();

        let aliases = vec!["test".to_string()];
        let result = format_group_and_aliases(&config, &Some("dev".to_string()), &aliases);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Group: dev"));
        assert!(output.contains("test -> echo test"));
    }

    #[test]
    fn test_handle_list_specific_existing_group() {
        let config = create_test_config();

        let cmd = ListCommand {
            pattern: None,
            group: Some(Some("dev".to_string())),
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_specific_nonexistent_group() {
        let config = create_test_config();
        let cmd = ListCommand {
            pattern: None,
            group: Some(Some("nonexistent".to_string())),
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_handle_list_all() {
        let config = create_test_config();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_enabled() {
        let config = create_test_config();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: true,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_disabled() {
        let config = create_test_config();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: true,
            global: false,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_no_aliases() {
        let config = Config::new();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: true,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_ungrouped() {
        let config = create_test_config();
        let cmd = ListCommand {
            pattern: None,
            group: Some(None),
            enabled: false,
            disabled: false,
            global: false,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_global() {
        let config = create_test_config();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: true,
        };
        let result = handle_list(&config, cmd, &ShellType::Bash);
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
        let result = group_header(&config, &Some("nonexistent".to_string()));
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_format_group_and_aliases_nonexistent_group() {
        let config = create_test_config();
        let aliases = vec!["test".to_string()];
        let result = format_group_and_aliases(&config, &Some("nonexistent".to_string()), &aliases);
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_format_group_and_aliases_nonexistent_alias() {
        let config = create_test_config();
        let aliases = vec!["nonexistent".to_string()];
        let result = format_group_and_aliases(&config, &Some("dev".to_string()), &aliases);
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

    #[test]
    fn test_format_group_and_aliases_single_group_ungrouped() {
        let config = create_test_config();
        let aliases = vec!["test".to_string()];
        let result = format_group_and_aliases_single_group(&config, &None, &aliases);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.contains("Group:"));
        assert!(output.contains("test -> echo test"));
    }

    #[test]
    fn test_format_group_and_aliases_single_group_named() {
        let config = create_test_config();
        let aliases = vec!["build".to_string()];
        let result =
            format_group_and_aliases_single_group(&config, &Some("dev".to_string()), &aliases);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Group: dev"));
        assert!(output.contains("build -> cargo build"));
    }

    #[test]
    fn test_retain_aliases_empty() {
        let config = create_test_config();
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: false,
        };
        retain_aliases(&config, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 2);
        assert!(aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_enabled() {
        let mut config = create_test_config();
        config.aliases.get_mut("build").unwrap().enabled = false;
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: true,
            disabled: false,
            global: false,
        };
        retain_aliases(&config, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(aliases.contains(&"test".to_string()));
        assert!(!aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_disabled() {
        let mut config = create_test_config();
        config.aliases.get_mut("build").unwrap().enabled = false;
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: true,
            global: false,
        };
        retain_aliases(&config, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(!aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_pattern() {
        let config = create_test_config();
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: Some("b*".to_string()),
            group: None,
            enabled: false,
            disabled: false,
            global: false,
        };
        retain_aliases(&config, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(!aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_global() {
        let mut config = create_test_config();
        config.aliases.get_mut("build").unwrap().global = true;
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: true,
        };
        retain_aliases(&config, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(!aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }
}
