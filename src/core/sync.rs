use super::Outcome;
use super::add::add_alias_str;
use super::list::get_all_aliases_grouped;
use crate::app::add::is_valid_alias_name;
use crate::app::shell::ShellType;
use crate::catalog::io::load_catalog;
use crate::catalog::types::AliasCatalog;
use crate::core::remove::remove_all_aliases;
use log::{info, warn};
use std::fmt::Write;
use std::path::PathBuf;

/// Generates the content of the alias script file based on the provided catalog.
///
/// # Arguments
/// * `catalog` - A reference to the catalog object containing aliases and groups.
///
/// # Returns
/// A string representing the content of the alias script file.
pub fn generate_alias_script_content(
    catalog: &AliasCatalog,
    shell: &ShellType,
    last_synced_catalog_path: &PathBuf,
) -> String {
    let mut content = String::new();

    let mut last_synced_catalog = match load_catalog(last_synced_catalog_path) {
        Ok(c) => c,
        Err(e) => {
            info!(
                "Failed to load last synced catalog from '{}': {}. Proceeding with empty catalog.",
                last_synced_catalog_path.display(),
                e
            );
            AliasCatalog::new()
        }
    };

    info!("Removing old aliases from the shell...");
    match remove_all_aliases(&mut last_synced_catalog, shell) {
        Err(_) => warn!(
            "Failed to generate remove commands for old aliases. Proceeding without removing old aliases.",
        ),
        Ok(Outcome::Command(cmds)) if !cmds.is_empty() => {
            writeln!(content, "{}", cmds).unwrap();
        }
        Ok(_) => {}
    };

    info!("Adding new aliases to the shell...");
    for (group, aliases) in get_all_aliases_grouped(catalog, &shell) {
        // Only add groups that are enabled, `ungrouped` is always enabled
        if match group {
            None => true,
            Some(g) => *catalog.groups.get(&g).unwrap(),
        } {
            for alias in &aliases {
                let alias_obj = catalog.aliases.get(alias).unwrap();
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
    use assert_fs::TempDir;

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

    fn missing_last_synced_path() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("last_synced_catalog.toml");
        (temp_dir, path)
    }

    #[test]
    fn empty_catalog_without_last_synced_catalog_is_empty() {
        let catalog = AliasCatalog::new();
        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);
        assert!(file_string.is_empty());
    }

    #[test]
    fn missing_last_synced_catalog_adds_current_aliases_without_removals() {
        let catalog = sample_catalog();
        let (_temp_dir, last_synced_path) = missing_last_synced_path();

        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);

        assert!(!last_synced_path.exists());
        assert!(!file_string.contains("unalias"));
        assert_eq!(file_string, "alias -- 'll'='ls -la'\n");
    }

    #[test]
    fn file_content_removes_aliases_from_last_synced_catalog() {
        let temp_dir = TempDir::new().unwrap();
        let last_synced_path = temp_dir.path().join("last_synced_catalog.toml");
        std::fs::write(&last_synced_path, "old_alias = \"echo old\"\n").unwrap();

        let catalog = sample_catalog();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);

        assert!(file_string.contains("unalias 'old_alias'"));
        assert!(file_string.contains("unalias 'old_alias'\nalias -- 'll'='ls -la'"));
        assert!(file_string.contains(SAMPLE_ALIAS_NAME));
    }

    #[test]
    fn file_content_contains_enabled_alias() {
        let catalog = sample_catalog();
        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);
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

        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);
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
        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);
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
        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);
        assert!(!file_string.contains("grouped_alias"));
    }

    #[test]
    fn file_content_excludes_global_alias_in_non_zsh_shell() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "global_alias".to_string(),
            Alias::new("echo Global".to_string(), None, true, true),
        );
        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);
        assert!(!file_string.contains("global_alias"));
    }

    #[test]
    fn file_content_includes_global_alias_in_zsh_shell() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "global_alias".to_string(),
            Alias::new("echo Global".to_string(), None, true, true),
        );
        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Zsh, &last_synced_path);
        assert!(file_string.contains("global_alias"));
    }

    #[test]
    fn file_content_excludes_invalid_alias_names() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "invalid alias".to_string(),
            Alias::new("echo Invalid".to_string(), None, true, false),
        );
        let (_temp_dir, last_synced_path) = missing_last_synced_path();
        let file_string =
            generate_alias_script_content(&catalog, &ShellType::Bash, &last_synced_path);
        assert!(!file_string.contains("invalid alias"));
    }
}
