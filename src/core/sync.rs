use super::add::add_alias_str;
use super::list::get_all_aliases_grouped;
use crate::app::add::is_valid_alias_name;
use crate::app::shell::ShellType;
use crate::catalog::types::AliasCatalog;
use log::warn;
use std::fmt::Write;

/// Generates the content of the alias script file based on the provided catalog.
///
/// # Arguments
/// * `catalog` - A reference to the catalog object containing aliases and groups.
///
/// # Returns
/// A string representing the content of the alias script file.
pub fn generate_alias_script_content(catalog: &AliasCatalog, shell: ShellType) -> String {
    let mut content = String::new();

    // Reset all existing aliases
    writeln!(content, "unalias -a").unwrap();

    for (group, aliases) in get_all_aliases_grouped(catalog, &shell) {
        // Only add groups that are enabled, `ungrouped` is always enabled
        if match group {
            None => true,
            Some(g) => *catalog.groups.get(&g).unwrap(),
        } {
            for alias in &aliases {
                let alias_obj = catalog.aliases.get(alias).unwrap();
                if alias_obj.global && shell != ShellType::Zsh {
                    warn!(
                        "Global aliases are only supported in zsh. Skipping alias '{}'.",
                        alias
                    );
                    continue;
                }
                if !is_valid_alias_name(alias) {
                    warn!(
                        "Alias name '{}' contains invalid characters. Skipping.",
                        alias
                    );
                    continue;
                }
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
    use crate::catalog::types::Alias;

    static SAMPLE_ALIAS_NAME: &str = "ll";

    fn sample_alias() -> Alias {
        Alias::new("ls -la".to_string(), None, true, false)
    }

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert(SAMPLE_ALIAS_NAME.to_string(), sample_alias());
        catalog
    }

    #[test]
    fn empty_catalog_only_contains_reset_command() {
        let catalog = AliasCatalog::new();
        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(file_string.contains("unalias -a"));
    }

    #[test]
    fn filled_catalog_contains_reset_command() {
        let catalog = sample_catalog();
        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(file_string.contains("unalias -a"));
    }

    #[test]
    fn file_content_contains_enabled_alias() {
        let catalog = sample_catalog();
        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(file_string.contains(SAMPLE_ALIAS_NAME));
    }

    #[test]
    fn file_content_excludes_disabled_alias() {
        let mut catalog = sample_catalog();
        let mut disabled_alias = sample_alias();
        disabled_alias.enabled = false;
        catalog
            .aliases
            .insert("disabled_alias".to_string(), disabled_alias);

        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(!file_string.contains("disabled_alias"));
        assert!(file_string.contains(SAMPLE_ALIAS_NAME));
    }

    #[test]
    fn file_content_contains_enabled_group_alias() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "grouped_alias".to_string(),
            Alias::new(
                "echo Grouped".to_string(),
                Some("my_group".to_string()),
                true,
                false,
            ),
        );
        catalog.groups.insert("my_group".to_string(), true);
        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(file_string.contains("grouped_alias"));
    }

    #[test]
    fn file_content_excledes_disabled_groups() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "grouped_alias".to_string(),
            Alias::new(
                "echo Grouped".to_string(),
                Some("my_group".to_string()),
                true,
                false,
            ),
        );
        catalog.groups.insert("my_group".to_string(), false);
        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(!file_string.contains("grouped_alias"));
    }

    #[test]
    fn file_content_excludes_global_alias_in_non_zsh_shell() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "global_alias".to_string(),
            Alias::new("echo Global".to_string(), None, true, true),
        );
        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(!file_string.contains("global_alias"));
    }

    #[test]
    fn file_content_includes_global_alias_in_zsh_shell() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "global_alias".to_string(),
            Alias::new("echo Global".to_string(), None, true, true),
        );
        let file_string = generate_alias_script_content(&catalog, ShellType::Zsh);
        assert!(file_string.contains("global_alias"));
    }

    #[test]
    fn file_content_excludes_invalid_alias_names() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "invalid alias".to_string(),
            Alias::new("echo Invalid".to_string(), None, true, false),
        );
        let file_string = generate_alias_script_content(&catalog, ShellType::Bash);
        assert!(!file_string.contains("invalid alias"));
    }
}
