use super::list::{GroupId, get_all_aliases_grouped};
use crate::config::types::Config;
use std::fmt::Write;

/// Generates the content of the alias script file based on the provided configuration.
///
/// # Arguments
/// * `config` - A reference to the configuration object containing aliases and groups.
///
/// # Returns
/// A string representing the content of the alias script file.
pub fn generate_alias_script_content(config: &Config) -> String {
    let mut content = String::new();

    // Reset all existing aliases
    writeln!(content, "unalias -a").unwrap();

    for (group, aliases) in get_all_aliases_grouped(config) {
        // Only add groups that are enabled, `ungrouped` is always enabled
        if match group {
            GroupId::Ungrouped => true,
            GroupId::Named(g) => *config.groups.get(&g).unwrap(),
        } {
            for alias in &aliases {
                let alias_obj = config.aliases.get(alias).unwrap();
                if alias_obj.enabled {
                    writeln!(content, "alias {}='{}'", alias, alias_obj.command).unwrap();
                }
            }
        }
    }

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Alias;

    #[test]
    fn empty_config_only_contains_reset_command() {
        let config = Config::new();
        let file_string = generate_alias_script_content(&config);
        assert!(file_string.contains("unalias -a"));
    }

    #[test]
    fn filled_config_contains_reset_command() {
        let mut config = Config::new();
        config.aliases.insert(
            "my_alias".to_string(),
            Alias::new("echo Hello".to_string(), true, None, false),
        );
        let file_string = generate_alias_script_content(&config);
        assert!(file_string.contains("unalias -a"));
    }

    #[test]
    fn file_content_contains_enabled_alias() {
        let mut config = Config::new();
        config.aliases.insert(
            "my_alias".to_string(),
            Alias::new("echo Hello".to_string(), true, None, false),
        );
        let file_string = generate_alias_script_content(&config);
        assert!(file_string.contains("my_alias"));
    }

    #[test]
    fn file_content_excludes_disabled_alias() {
        let mut config = Config::new();
        config.aliases.insert(
            "my_alias".to_string(),
            Alias::new("echo Hello".to_string(), false, None, true),
        );
        let file_string = generate_alias_script_content(&config);
        assert!(!file_string.contains("my_alias"));
    }

    #[test]
    fn file_content_contains_enabled_group_alias() {
        let mut config = Config::new();
        config.aliases.insert(
            "grouped_alias".to_string(),
            Alias::new(
                "echo Grouped".to_string(),
                true,
                Some("my_group".to_string()),
                false,
            ),
        );
        config.groups.insert("my_group".to_string(), true);
        let file_string = generate_alias_script_content(&config);
        assert!(file_string.contains("grouped_alias"));
    }

    #[test]
    fn file_content_excledes_disabled_groups() {
        let mut config = Config::new();
        config.aliases.insert(
            "grouped_alias".to_string(),
            Alias::new(
                "echo Grouped".to_string(),
                true,
                Some("my_group".to_string()),
                false,
            ),
        );
        config.groups.insert("my_group".to_string(), false);
        let file_string = generate_alias_script_content(&config);
        assert!(!file_string.contains("grouped_alias"));
    }
}
