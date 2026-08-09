use serde::Serialize;
use std::collections::BTreeMap;

use super::shell::ShellType;
use crate::catalog::types::AliasCatalog;
use crate::cli::list::{ListCommand, OutputFormat};
use crate::config::UserConfig;
use crate::core::list::{get_aliases_from_single_group, get_all_aliases_grouped};
use crate::core::{Failure, Outcome};

use globset::Glob;

/// Returns a colored symbol representing the enabled status.
struct HumanFormatter<'a> {
    config: &'a UserConfig,
    colors_enabled: bool,
}

impl HumanFormatter<'_> {
    fn enabled_symbol(&self, enabled: bool) -> String {
        if enabled {
            self.config
                .styles
                .enabled
                .render(&self.config.symbols.enabled, self.colors_enabled)
        } else {
            self.config
                .styles
                .disabled
                .render(&self.config.symbols.disabled, self.colors_enabled)
        }
    }

    fn globe_symbol(&self, global: bool) -> String {
        if global {
            format!(
                " {}",
                self.config
                    .styles
                    .global
                    .render(&self.config.symbols.global, self.colors_enabled)
            )
        } else {
            String::new()
        }
    }

    fn format_alias_info(&self, catalog: &AliasCatalog, alias: &str) -> Result<String, Failure> {
        if let Some(alias_info) = catalog.aliases.get(alias) {
            Ok(format!(
                "{}{} {} -> {}",
                self.enabled_symbol(alias_info.enabled),
                self.globe_symbol(alias_info.global),
                alias,
                alias_info.command
            ))
        } else {
            eprintln!("Alias '{}' not found in catalog.", alias);
            Err(Failure::AliasDoesNotExist)
        }
    }

    fn group_header(
        &self,
        catalog: &AliasCatalog,
        group: &Option<String>,
    ) -> Result<String, Failure> {
        let group_enabled;
        let group_name;
        if let Some(g) = group {
            match catalog.groups.get(g) {
                Some(enabled) => {
                    group_enabled = enabled;
                    group_name = g.clone();
                }
                None => {
                    eprintln!("Group '{}' does not exist in catalog.", g);
                    return Err(Failure::GroupDoesNotExist);
                }
            }
        } else {
            group_enabled = &true;
            group_name = "ungrouped".to_string();
        }

        let header_message = format!(
            " Group: {} {} ",
            group_name,
            self.enabled_symbol(*group_enabled)
        );
        Ok(format!("{:=^width$}", header_message, width = 50))
    }

    fn format_aliases_list(
        &self,
        catalog: &AliasCatalog,
        aliases: &[String],
    ) -> Result<String, Failure> {
        let mut content = String::new();
        for alias in aliases {
            content += &(self.format_alias_info(catalog, alias)? + "\n");
        }
        Ok(content)
    }
}

#[cfg(test)]
fn enabled_symbol(enabled: bool) -> String {
    HumanFormatter {
        config: &UserConfig::default(),
        colors_enabled: true,
    }
    .enabled_symbol(enabled)
}

#[cfg(test)]
fn globe_symbol(global: bool) -> String {
    HumanFormatter {
        config: &UserConfig::default(),
        colors_enabled: true,
    }
    .globe_symbol(global)
}

pub fn format_alias_info(catalog: &AliasCatalog, alias: &str) -> Result<String, Failure> {
    HumanFormatter {
        config: &UserConfig::default(),
        colors_enabled: true,
    }
    .format_alias_info(catalog, alias)
}

#[cfg(test)]
fn group_header(catalog: &AliasCatalog, group: &Option<String>) -> Result<String, Failure> {
    HumanFormatter {
        config: &UserConfig::default(),
        colors_enabled: true,
    }
    .group_header(catalog, group)
}

#[cfg(test)]
fn format_group_and_aliases(
    catalog: &AliasCatalog,
    group_id: &Option<String>,
    aliases: &[String],
) -> Result<String, Failure> {
    let mut content = String::new();
    content += &(group_header(catalog, group_id)? + "\n");
    content += &format_aliases_list(catalog, aliases)?;
    Ok(content)
}

/// Formats a list of aliases without a group header.
#[cfg(test)]
fn format_aliases_list(catalog: &AliasCatalog, aliases: &[String]) -> Result<String, Failure> {
    HumanFormatter {
        config: &UserConfig::default(),
        colors_enabled: true,
    }
    .format_aliases_list(catalog, aliases)
}

/// If ungrouped, will remove the group header
#[cfg(test)]
fn format_group_and_aliases_single_group(
    catalog: &AliasCatalog,
    group_id: &Option<String>,
    aliases: &[String],
) -> Result<String, Failure> {
    let mut content = String::new();
    if group_id.is_some() {
        content += &(group_header(catalog, group_id)? + "\n");
    }
    content += &format_aliases_list(catalog, aliases)?;
    Ok(content)
}

fn retain_aliases(catalog: &AliasCatalog, aliases: &mut Vec<String>, cmd: &ListCommand) {
    if let Some(pattern) = &cmd.pattern {
        let glob = Glob::new(pattern).unwrap().compile_matcher();
        aliases.retain(|alias| glob.is_match(alias));
    }
    if cmd.enabled {
        aliases.retain(|alias| catalog.aliases[alias].enabled);
    } else if cmd.disabled {
        aliases.retain(|alias| !catalog.aliases[alias].enabled);
    }

    if cmd.global {
        aliases.retain(|alias| catalog.aliases[alias].global);
    }
}

fn select_aliases(
    catalog: &AliasCatalog,
    cmd: &ListCommand,
    shell: &ShellType,
) -> Result<BTreeMap<Option<String>, Vec<String>>, Failure> {
    let mut groups = if let Some(group) = &cmd.group {
        let group_id = group.clone();
        let aliases = get_aliases_from_single_group(catalog, group_id.as_deref(), shell)?;
        BTreeMap::from([(group_id, aliases)])
    } else {
        get_all_aliases_grouped(catalog, shell)
    };

    for aliases in groups.values_mut() {
        retain_aliases(catalog, aliases, cmd);
    }
    groups.retain(|_, aliases| !aliases.is_empty());
    Ok(groups)
}

#[derive(Serialize)]
struct JsonAlias<'a> {
    name: &'a str,
    command: &'a str,
    group: Option<&'a str>,
    enabled: bool,
    global: bool,
}

fn format_json(catalog: &AliasCatalog, groups: &BTreeMap<Option<String>, Vec<String>>) -> String {
    let aliases = groups
        .values()
        .flatten()
        .map(|name| {
            let alias = &catalog.aliases[name];
            JsonAlias {
                name,
                command: &alias.command,
                group: alias.group.as_deref(),
                enabled: alias.enabled,
                global: alias.global,
            }
        })
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&aliases).expect("alias list is JSON serializable") + "\n"
}

fn format_human(
    catalog: &AliasCatalog,
    groups: &BTreeMap<Option<String>, Vec<String>>,
    focused_group: bool,
    config: &UserConfig,
    colors_enabled: bool,
) -> Result<String, Failure> {
    let formatter = HumanFormatter {
        config,
        colors_enabled,
    };
    let mut content = String::new();
    for (group_id, aliases) in groups {
        if focused_group {
            if group_id.is_some() {
                content += &(formatter.group_header(catalog, group_id)? + "\n");
            }
            content += &formatter.format_aliases_list(catalog, aliases)?;
        } else {
            content += &(formatter.group_header(catalog, group_id)? + "\n");
            content += &formatter.format_aliases_list(catalog, aliases)?;
        }
    }
    Ok(content)
}

fn format_list_with_config(
    catalog: &AliasCatalog,
    cmd: &ListCommand,
    shell: &ShellType,
    config: &UserConfig,
    colors_enabled: bool,
) -> Result<String, Failure> {
    let groups = select_aliases(catalog, cmd, shell)?;
    match cmd.format {
        OutputFormat::Human => format_human(
            catalog,
            &groups,
            cmd.group.is_some(),
            config,
            colors_enabled,
        ),
        OutputFormat::Json => Ok(format_json(catalog, &groups)),
    }
}

#[cfg(test)]
fn format_list(
    catalog: &AliasCatalog,
    cmd: &ListCommand,
    shell: &ShellType,
) -> Result<String, Failure> {
    format_list_with_config(catalog, cmd, shell, &UserConfig::default(), true)
}

/// Handle the 'list' command based on the provided options.
/// This function lists aliases according to the specified criteria:
/// - If a specific group is provided, it lists aliases in that group.
/// - If the 'all' flag is set, it lists all aliases.
/// - If the 'disabled' flag is set, it lists only disabled aliases.
/// - By default, it lists only enabled aliases.
///
/// # Arguments
/// - `catalog`: Reference to the catalog containing aliases and groups.
/// - `cmd`: The ListCommand containing options for listing.
///
/// # Returns
/// - `Outcome::NoChanges` if the operation is successful.
/// - `Failure::GroupDoesNotExist` if the specified group does not exist.
/// - Other failures as defined in the `Failure` enum.
pub fn handle_list(
    catalog: &AliasCatalog,
    cmd: ListCommand,
    shell: &ShellType,
    config: &UserConfig,
    colors_enabled: bool,
) -> Result<Outcome, Failure> {
    print!(
        "{}",
        format_list_with_config(catalog, &cmd, shell, config, colors_enabled)?
    );
    Ok(Outcome::NoChanges)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::{Alias, AliasCatalog};
    use assert_matches::assert_matches;

    fn create_test_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        // Ungrouped alias
        catalog.aliases.insert(
            "test".to_string(),
            Alias::new("echo test".to_string(), None, true, false),
        );
        // Grouped alias
        catalog.aliases.insert(
            "build".to_string(),
            Alias::new(
                "cargo build".to_string(),
                Some("dev".to_string()),
                true,
                false,
            ),
        );
        catalog.groups.insert("dev".to_string(), true);
        catalog
    }

    fn create_filtered_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("dev".to_string(), true);
        catalog.groups.insert("ops".to_string(), true);
        catalog.groups.insert("zsh".to_string(), true);
        catalog.aliases.insert(
            "build".to_string(),
            Alias::new(
                "cargo build".to_string(),
                Some("dev".to_string()),
                true,
                false,
            ),
        );
        catalog.aliases.insert(
            "deploy".to_string(),
            Alias::new("deploy".to_string(), Some("ops".to_string()), false, false),
        );
        catalog.aliases.insert(
            "glob".to_string(),
            Alias::new("*.rs".to_string(), Some("zsh".to_string()), true, true),
        );
        catalog
    }

    #[test]
    fn configured_symbols_and_styles_are_used() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: Some(None),
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let mut config = UserConfig::default();
        config.symbols.enabled = "+".into();
        config.styles.enabled.foreground = "magenta".into();

        let plain =
            format_list_with_config(&catalog, &cmd, &ShellType::Bash, &config, false).unwrap();
        let colored =
            format_list_with_config(&catalog, &cmd, &ShellType::Bash, &config, true).unwrap();
        assert!(plain.starts_with("+ test"));
        assert!(colored.starts_with("\u{1b}[35;1m+\u{1b}[0m test"));
    }

    #[test]
    fn test_enabled_symbol() {
        assert_eq!(enabled_symbol(true), "\u{1b}[32;1m✔\u{1b}[0m");
        assert_eq!(enabled_symbol(false), "\u{1b}[31;1m✘\u{1b}[0m");
    }

    #[test]
    fn test_print_alias_valid() {
        let catalog = create_test_catalog();

        let result = format_alias_info(&catalog, "test");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{} test -> echo test", enabled_symbol(true))
        );
    }

    #[test]
    fn test_group_header_valid() {
        let catalog = create_test_catalog();

        let result = group_header(&catalog, &Some("dev".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Group: dev"));
    }

    #[test]
    fn test_format_group_and_aliases_valid() {
        let catalog = create_test_catalog();

        let aliases = vec!["test".to_string()];
        let result = format_group_and_aliases(&catalog, &Some("dev".to_string()), &aliases);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Group: dev"));
        assert!(output.contains("test -> echo test"));
    }

    #[test]
    fn list_omits_empty_groups_and_sorts_nonempty_groups() {
        let catalog = create_filtered_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };

        let output = format_list(&catalog, &cmd, &ShellType::Bash).unwrap();

        assert!(!output.contains("Group: ungrouped"));
        assert!(!output.contains("Group: zsh"));
        assert!(output.find("Group: dev").unwrap() < output.find("Group: ops").unwrap());
    }

    #[test]
    fn pattern_filter_only_formats_groups_with_matches() {
        let catalog = create_filtered_catalog();
        let cmd = ListCommand {
            pattern: Some("b*".to_string()),
            group: None,
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };

        let output = format_list(&catalog, &cmd, &ShellType::Zsh).unwrap();

        assert!(output.contains("Group: dev"));
        assert!(!output.contains("Group: ops"));
        assert!(!output.contains("Group: zsh"));
    }

    #[test]
    fn enabled_and_disabled_filters_omit_empty_groups() {
        let catalog = create_filtered_catalog();
        let enabled = ListCommand {
            pattern: None,
            group: None,
            enabled: true,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let disabled = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: true,
            global: false,
            format: OutputFormat::Human,
        };

        let enabled_output = format_list(&catalog, &enabled, &ShellType::Bash).unwrap();
        let disabled_output = format_list(&catalog, &disabled, &ShellType::Bash).unwrap();

        assert!(enabled_output.contains("Group: dev"));
        assert!(!enabled_output.contains("Group: ops"));
        assert!(!enabled_output.contains("Group: zsh"));
        assert!(!disabled_output.contains("Group: dev"));
        assert!(disabled_output.contains("Group: ops"));
        assert!(!disabled_output.contains("Group: zsh"));
    }

    #[test]
    fn global_filter_omits_shell_incompatible_and_nonmatching_groups() {
        let catalog = create_filtered_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: true,
            format: OutputFormat::Human,
        };

        assert_eq!(format_list(&catalog, &cmd, &ShellType::Bash).unwrap(), "");

        let zsh_output = format_list(&catalog, &cmd, &ShellType::Zsh).unwrap();
        assert!(!zsh_output.contains("Group: dev"));
        assert!(!zsh_output.contains("Group: ops"));
        assert!(zsh_output.contains("Group: zsh"));
    }

    #[test]
    fn json_lists_ungrouped_and_grouped_aliases() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Json,
        };

        let output = format_list(&catalog, &cmd, &ShellType::Bash).unwrap();
        let aliases: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(
            aliases,
            serde_json::json!([
                {
                    "name": "test",
                    "command": "echo test",
                    "group": null,
                    "enabled": true,
                    "global": false
                },
                {
                    "name": "build",
                    "command": "cargo build",
                    "group": "dev",
                    "enabled": true,
                    "global": false
                }
            ])
        );
    }

    #[test]
    fn json_respects_pattern_group_and_status_filters() {
        let catalog = create_filtered_catalog();
        let cmd = ListCommand {
            pattern: Some("d*".to_string()),
            group: Some(Some("ops".to_string())),
            enabled: false,
            disabled: true,
            global: false,
            format: OutputFormat::Json,
        };

        let output = format_list(&catalog, &cmd, &ShellType::Zsh).unwrap();
        let aliases: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(
            aliases,
            serde_json::json!([{
                "name": "deploy",
                "command": "deploy",
                "group": "ops",
                "enabled": false,
                "global": false
            }])
        );
    }

    #[test]
    fn json_only_includes_global_aliases_for_zsh() {
        let catalog = create_filtered_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: true,
            format: OutputFormat::Json,
        };

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &format_list(&catalog, &cmd, &ShellType::Bash).unwrap()
            )
            .unwrap(),
            serde_json::json!([])
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &format_list(&catalog, &cmd, &ShellType::Zsh).unwrap()
            )
            .unwrap(),
            serde_json::json!([{
                "name": "glob",
                "command": "*.rs",
                "group": "zsh",
                "enabled": true,
                "global": true
            }])
        );
    }

    #[test]
    fn no_matches_or_empty_focused_group_produce_no_output() {
        let catalog = create_filtered_catalog();
        let no_matches = ListCommand {
            pattern: Some("missing*".to_string()),
            group: None,
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let empty_focused_group = ListCommand {
            pattern: Some("missing*".to_string()),
            group: Some(Some("dev".to_string())),
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };

        assert_eq!(
            format_list(&catalog, &no_matches, &ShellType::Zsh).unwrap(),
            ""
        );
        assert_eq!(
            format_list(&catalog, &empty_focused_group, &ShellType::Zsh).unwrap(),
            ""
        );
    }

    #[test]
    fn test_handle_list_specific_existing_group() {
        let catalog = create_test_catalog();

        let cmd = ListCommand {
            pattern: None,
            group: Some(Some("dev".to_string())),
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_specific_nonexistent_group() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: Some(Some("nonexistent".to_string())),
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_handle_list_all() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_enabled() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: true,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_disabled() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: true,
            global: false,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_no_aliases() {
        let catalog = AliasCatalog::new();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: true,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_ungrouped() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: Some(None),
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_list_global() {
        let catalog = create_test_catalog();
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: true,
            format: OutputFormat::Human,
        };
        let result = handle_list(
            &catalog,
            cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            true,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_alias_info_nonexistent_alias() {
        let catalog = create_test_catalog();
        let result = format_alias_info(&catalog, "nonexistent");
        assert_matches!(result, Err(Failure::AliasDoesNotExist));
    }

    #[test]
    fn test_group_header_nonexistent_group() {
        let catalog = create_test_catalog();
        let result = group_header(&catalog, &Some("nonexistent".to_string()));
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_format_group_and_aliases_nonexistent_group() {
        let catalog = create_test_catalog();
        let aliases = vec!["test".to_string()];
        let result = format_group_and_aliases(&catalog, &Some("nonexistent".to_string()), &aliases);
        assert_matches!(result, Err(Failure::GroupDoesNotExist));
    }

    #[test]
    fn test_format_group_and_aliases_nonexistent_alias() {
        let catalog = create_test_catalog();
        let aliases = vec!["nonexistent".to_string()];
        let result = format_group_and_aliases(&catalog, &Some("dev".to_string()), &aliases);
        assert!(matches!(result, Err(Failure::AliasDoesNotExist)));
    }

    #[test]
    fn test_format_aliases_list_nonexistent_alias() {
        let catalog = create_test_catalog();
        let aliases = vec!["nonexistent".to_string()];
        let result = format_aliases_list(&catalog, &aliases);
        assert!(matches!(result, Err(Failure::AliasDoesNotExist)));
    }

    #[test]
    fn test_global_symbol() {
        assert_eq!(globe_symbol(true), " \u{1b}[34;1m⦾\u{1b}[0m");
        assert_eq!(globe_symbol(false), "".to_string());
    }

    #[test]
    fn test_format_group_and_aliases_single_group_ungrouped() {
        let catalog = create_test_catalog();
        let aliases = vec!["test".to_string()];
        let result = format_group_and_aliases_single_group(&catalog, &None, &aliases);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.contains("Group:"));
        assert!(output.contains("test -> echo test"));
    }

    #[test]
    fn test_format_group_and_aliases_single_group_named() {
        let catalog = create_test_catalog();
        let aliases = vec!["build".to_string()];
        let result =
            format_group_and_aliases_single_group(&catalog, &Some("dev".to_string()), &aliases);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Group: dev"));
        assert!(output.contains("build -> cargo build"));
    }

    #[test]
    fn test_retain_aliases_empty() {
        let catalog = create_test_catalog();
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        retain_aliases(&catalog, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 2);
        assert!(aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_enabled() {
        let mut catalog = create_test_catalog();
        catalog.aliases.get_mut("build").unwrap().enabled = false;
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: true,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        retain_aliases(&catalog, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(aliases.contains(&"test".to_string()));
        assert!(!aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_disabled() {
        let mut catalog = create_test_catalog();
        catalog.aliases.get_mut("build").unwrap().enabled = false;
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: true,
            global: false,
            format: OutputFormat::Human,
        };
        retain_aliases(&catalog, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(!aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_pattern() {
        let catalog = create_test_catalog();
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: Some("b*".to_string()),
            group: None,
            enabled: false,
            disabled: false,
            global: false,
            format: OutputFormat::Human,
        };
        retain_aliases(&catalog, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(!aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }

    #[test]
    fn test_retain_aliases_global() {
        let mut catalog = create_test_catalog();
        catalog.aliases.get_mut("build").unwrap().global = true;
        let mut aliases = vec!["test".to_string(), "build".to_string()];
        let cmd = ListCommand {
            pattern: None,
            group: None,
            enabled: false,
            disabled: false,
            global: true,
            format: OutputFormat::Human,
        };
        retain_aliases(&catalog, &mut aliases, &cmd);
        assert!(!aliases.is_empty());
        assert_eq!(aliases.len(), 1);
        assert!(!aliases.contains(&"test".to_string()));
        assert!(aliases.contains(&"build".to_string()));
    }
}
