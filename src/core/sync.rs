use super::add::add_alias_str;
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
                    writeln!(content, "{}", add_alias_str(alias, alias_obj)).unwrap();
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

    static SAMPLE_ALIAS_NAME: &str = "ll";

    fn sample_alias() -> Alias {
        Alias::new("ls -la".to_string(), None, true, false)
    }

    fn sample_config() -> Config {
        let mut config = Config::new();
        config
            .aliases
            .insert(SAMPLE_ALIAS_NAME.to_string(), sample_alias());
        config
    }

    #[test]
    fn empty_config_only_contains_reset_command() {
        let config = Config::new();
        let file_string = generate_alias_script_content(&config);
        assert!(file_string.contains("unalias -a"));
    }

    #[test]
    fn filled_config_contains_reset_command() {
        let config = sample_config();
        let file_string = generate_alias_script_content(&config);
        assert!(file_string.contains("unalias -a"));
    }

    #[test]
    fn file_content_contains_enabled_alias() {
        let config = sample_config();
        let file_string = generate_alias_script_content(&config);
        assert!(file_string.contains(SAMPLE_ALIAS_NAME));
    }

    #[test]
    fn file_content_excludes_disabled_alias() {
        let mut config = sample_config();
        let mut disabled_alias = sample_alias();
        disabled_alias.enabled = false;
        config
            .aliases
            .insert("disabled_alias".to_string(), disabled_alias);

        let file_string = generate_alias_script_content(&config);
        assert!(!file_string.contains("disabled_alias"));
        assert!(file_string.contains(SAMPLE_ALIAS_NAME));
    }

    #[test]
    fn file_content_contains_enabled_group_alias() {
        let mut config = Config::new();
        config.aliases.insert(
            "grouped_alias".to_string(),
            Alias::new(
                "echo Grouped".to_string(),
                Some("my_group".to_string()),
                true,
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
                Some("my_group".to_string()),
                true,
                false,
            ),
        );
        config.groups.insert("my_group".to_string(), false);
        let file_string = generate_alias_script_content(&config);
        assert!(!file_string.contains("grouped_alias"));
    }
}
