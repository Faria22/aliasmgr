use crate::config::types::Config;

/// Generates the content of the alias script file based on the provided configuration.
///
/// # Arguments
/// * `config` - A reference to the configuration object containing aliases and groups.
///
/// # Returns
/// A string representing the content of the alias script file.
pub fn generate_alias_script_content(config: &Config) -> String {
    todo!("Implement file string generation based on Config")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Alias;

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
