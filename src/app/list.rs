use std::io::IsTerminal;

use globset::Glob;
use owo_colors::OwoColorize;
use serde::Serialize;
use terminal_size::{Width, terminal_size};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::shell::ShellType;
use crate::catalog::types::{Alias, AliasCatalog};
use crate::cli::list::{ListColumn, ListCommand, OutputFormat};
use crate::config::{StatusColumnMode, UserConfig};
use crate::core::list::visible_aliases;
use crate::core::{Failure, Outcome};

#[derive(Serialize)]
struct JsonAlias<'a> {
    name: &'a str,
    command: &'a str,
    enabled: bool,
    global: bool,
    tags: &'a std::collections::BTreeSet<String>,
    description: Option<&'a str>,
}

fn selected_aliases<'a>(
    catalog: &'a AliasCatalog,
    cmd: &ListCommand,
    shell: &ShellType,
) -> Result<Vec<(&'a str, &'a Alias)>, Failure> {
    let matcher = cmd
        .pattern
        .as_deref()
        .map(|pattern| {
            Glob::new(pattern)
                .map(|glob| glob.compile_matcher())
                .map_err(|_| Failure::InvalidPattern)
        })
        .transpose()?;

    Ok(visible_aliases(catalog, shell)
        .filter(|(name, _)| {
            matcher
                .as_ref()
                .is_none_or(|matcher| matcher.is_match(name))
        })
        .filter(|(_, alias)| cmd.tag.iter().all(|tag| alias.tags.contains(tag)))
        .filter(|(_, alias)| {
            cmd.all
                || if cmd.disabled {
                    !alias.enabled
                } else {
                    alias.enabled
                }
        })
        .filter(|(_, alias)| !cmd.global || alias.global)
        .map(|(name, alias)| (name.as_str(), alias))
        .collect())
}

fn header(column: ListColumn) -> &'static str {
    match column {
        ListColumn::Status => "Status",
        ListColumn::Name => "Name",
        ListColumn::Command => "Command",
        ListColumn::Global => "Global",
        ListColumn::Tags => "Tags",
        ListColumn::Description => "Description",
    }
}

fn single_line(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\n' | '\r' | '\t' => ' ',
            _ => character,
        })
        .collect()
}

fn raw_cell(column: ListColumn, name: &str, alias: &Alias, config: &UserConfig) -> String {
    match column {
        ListColumn::Status if alias.enabled => config.symbols.enabled.clone(),
        ListColumn::Status => config.symbols.disabled.clone(),
        ListColumn::Name => name.to_owned(),
        ListColumn::Command => single_line(&alias.command),
        ListColumn::Global if alias.global => config.symbols.global.clone(),
        ListColumn::Global => String::new(),
        ListColumn::Tags => alias.tags.iter().cloned().collect::<Vec<_>>().join(", "),
        ListColumn::Description => alias
            .description
            .as_deref()
            .map(single_line)
            .unwrap_or_default(),
    }
}

fn truncate(value: &str, width: usize) -> String {
    if UnicodeWidthStr::width(value) <= width {
        return value.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    if width == 1 {
        return "…".into();
    }
    let target = width - 1;
    let mut used = 0;
    let mut output = String::new();
    for character in value.chars() {
        let character_width = character.width().unwrap_or(0);
        if used + character_width > target {
            break;
        }
        output.push(character);
        used += character_width;
    }
    output.push('…');
    output
}

fn column_widths(
    columns: &[ListColumn],
    rows: &[Vec<String>],
    terminal_width: Option<usize>,
) -> Vec<usize> {
    let mut widths = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            rows.iter()
                .map(|row| UnicodeWidthStr::width(row[index].as_str()))
                .chain([UnicodeWidthStr::width(header(*column))])
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();
    let Some(available) = terminal_width else {
        return widths;
    };
    let separators = columns.len().saturating_sub(1) * 2;
    let shrink_order = [
        ListColumn::Description,
        ListColumn::Command,
        ListColumn::Tags,
        ListColumn::Name,
        ListColumn::Global,
        ListColumn::Status,
    ];

    for preserve_headers in [true, false] {
        while widths.iter().sum::<usize>() + separators > available {
            let mut changed = false;
            for candidate in shrink_order {
                if let Some(index) = columns.iter().position(|column| *column == candidate) {
                    let minimum = if preserve_headers {
                        UnicodeWidthStr::width(header(candidate))
                    } else {
                        1
                    };
                    if widths[index] > minimum {
                        widths[index] -= 1;
                        changed = true;
                        if widths.iter().sum::<usize>() + separators <= available {
                            break;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }
    widths
}

fn styled_cell(
    column: ListColumn,
    raw: &str,
    alias: &Alias,
    config: &UserConfig,
    colors_enabled: bool,
) -> String {
    match column {
        ListColumn::Status if alias.enabled => config.styles.enabled.render(raw, colors_enabled),
        ListColumn::Status => config.styles.disabled.render(raw, colors_enabled),
        ListColumn::Global if alias.global => config.styles.global.render(raw, colors_enabled),
        _ => raw.to_owned(),
    }
}

fn format_human(
    aliases: &[(&str, &Alias)],
    columns: &[ListColumn],
    config: &UserConfig,
    colors_enabled: bool,
    terminal_width: Option<usize>,
) -> String {
    if aliases.is_empty() || columns.is_empty() {
        return String::new();
    }
    let raw_rows = aliases
        .iter()
        .map(|(name, alias)| {
            columns
                .iter()
                .map(|column| raw_cell(*column, name, alias, config))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let widths = column_widths(columns, &raw_rows, terminal_width);

    let header_cells = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let value = truncate(header(*column), widths[index]);
            let padding = widths[index].saturating_sub(UnicodeWidthStr::width(value.as_str()));
            let value = if config.styles.header.bold && colors_enabled {
                value.bold().to_string()
            } else {
                value
            };
            value + &" ".repeat(padding)
        })
        .collect::<Vec<_>>();
    let mut output = header_cells.join("  ").trim_end().to_owned() + "\n";
    for ((_, alias), raw_row) in aliases.iter().zip(raw_rows) {
        let cells = raw_row
            .into_iter()
            .enumerate()
            .map(|(index, raw)| {
                let value = truncate(&raw, widths[index]);
                let padding = widths[index].saturating_sub(UnicodeWidthStr::width(value.as_str()));
                styled_cell(columns[index], &value, alias, config, colors_enabled)
                    + &" ".repeat(padding)
            })
            .collect::<Vec<_>>();
        output.push_str(cells.join("  ").trim_end());
        output.push('\n');
    }
    output
}

fn format_json(aliases: &[(&str, &Alias)]) -> String {
    let aliases = aliases
        .iter()
        .map(|(name, alias)| JsonAlias {
            name,
            command: &alias.command,
            enabled: alias.enabled,
            global: alias.global,
            tags: &alias.tags,
            description: alias.description.as_deref(),
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&aliases).expect("alias list serializes") + "\n"
}

fn format_list_with_width(
    catalog: &AliasCatalog,
    cmd: &ListCommand,
    shell: &ShellType,
    config: &UserConfig,
    colors_enabled: bool,
    terminal_width: Option<usize>,
) -> Result<String, Failure> {
    let aliases = selected_aliases(catalog, cmd, shell)?;
    if let Some(columns) = &cmd.columns
        && columns
            .iter()
            .enumerate()
            .any(|(index, column)| columns[..index].contains(column))
    {
        return Err(Failure::InvalidColumns);
    }
    Ok(match cmd.format {
        OutputFormat::Json => format_json(&aliases),
        OutputFormat::Human => {
            let mut columns = if let Some(columns) = &cmd.columns {
                columns.clone()
            } else {
                let mut columns = config.list.columns.clone();
                let show_status = match config.list.status {
                    StatusColumnMode::Auto => cmd.all,
                    StatusColumnMode::Always => true,
                    StatusColumnMode::Never => false,
                };
                columns.retain(|column| *column != ListColumn::Status);
                if show_status {
                    columns.insert(0, ListColumn::Status);
                }
                columns
            };
            if *shell == ShellType::Bash {
                columns.retain(|column| *column != ListColumn::Global);
            }
            format_human(&aliases, &columns, config, colors_enabled, terminal_width)
        }
    })
}

pub fn handle_list(
    catalog: &AliasCatalog,
    cmd: ListCommand,
    shell: &ShellType,
    config: &UserConfig,
    colors_enabled: bool,
) -> Result<Outcome, Failure> {
    let width = std::io::stdout()
        .is_terminal()
        .then(|| terminal_size().map(|(Width(width), _)| usize::from(width)))
        .flatten();
    print!(
        "{}",
        format_list_with_width(catalog, &cmd, shell, config, colors_enabled, width)?
    );
    Ok(Outcome::NoChanges)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::cli::list::ListCommand;

    fn command(format: OutputFormat) -> ListCommand {
        ListCommand {
            pattern: None,
            tag: vec![],
            disabled: false,
            all: false,
            global: false,
            format,
            columns: None,
        }
    }

    fn catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        let mut alias = Alias::new("cargo test --workspace".into(), true, false);
        alias.tags.extend(["dev".into(), "rust".into()]);
        alias.description = Some("Run the complete test suite".into());
        catalog.aliases.insert("test".into(), alias);
        catalog
    }

    #[test]
    fn default_table_hides_redundant_columns_for_bash() {
        let output = format_list_with_width(
            &catalog(),
            &command(OutputFormat::Human),
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        assert!(output.lines().next().unwrap().starts_with("Name  Command"));
        let header = output.lines().next().unwrap();
        assert!(!header.contains("Status"));
        assert!(!header.contains("Global"));

        let output = format_list_with_width(
            &catalog(),
            &command(OutputFormat::Human),
            &ShellType::Zsh,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        assert!(output.lines().next().unwrap().contains("Global"));
    }

    #[test]
    fn table_headers_are_bold_by_default_and_configurable() {
        let output = format_list_with_width(
            &catalog(),
            &command(OutputFormat::Human),
            &ShellType::Bash,
            &UserConfig::default(),
            true,
            None,
        )
        .unwrap();
        assert!(
            output
                .lines()
                .next()
                .unwrap()
                .contains("\u{1b}[1mName\u{1b}[0m")
        );

        let mut config = UserConfig::default();
        config.styles.header.bold = false;
        let output = format_list_with_width(
            &catalog(),
            &command(OutputFormat::Human),
            &ShellType::Bash,
            &config,
            true,
            None,
        )
        .unwrap();
        assert!(output.starts_with("Name  Command"));
        assert!(!output.lines().next().unwrap().contains("\u{1b}["));
    }

    #[test]
    fn narrow_tables_truncate_without_dropping_columns() {
        let mut cmd = command(OutputFormat::Human);
        cmd.all = true;
        let output = format_list_with_width(
            &catalog(),
            &cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            Some(45),
        )
        .unwrap();
        assert!(output.contains('…'));
        assert_eq!(output.lines().next().unwrap().split_whitespace().count(), 5);
    }

    #[test]
    fn list_scope_defaults_to_enabled_and_all_includes_both_states() {
        let mut catalog = catalog();
        catalog.aliases.insert(
            "disabled".into(),
            Alias::new("echo disabled".into(), false, false),
        );

        let default = format_list_with_width(
            &catalog,
            &command(OutputFormat::Json),
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        let default: serde_json::Value = serde_json::from_str(&default).unwrap();
        assert_eq!(default.as_array().unwrap().len(), 1);
        assert_eq!(default[0]["name"], "test");

        let mut disabled = command(OutputFormat::Json);
        disabled.disabled = true;
        let disabled = format_list_with_width(
            &catalog,
            &disabled,
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        let disabled: serde_json::Value = serde_json::from_str(&disabled).unwrap();
        assert_eq!(disabled.as_array().unwrap().len(), 1);
        assert_eq!(disabled[0]["name"], "disabled");

        let mut all = command(OutputFormat::Json);
        all.all = true;
        let all = format_list_with_width(
            &catalog,
            &all,
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        let all: serde_json::Value = serde_json::from_str(&all).unwrap();
        assert_eq!(all.as_array().unwrap().len(), 2);
    }

    #[test]
    fn status_policy_is_dynamic_unless_columns_are_explicit() {
        let mut all = command(OutputFormat::Human);
        all.all = true;
        let output = format_list_with_width(
            &catalog(),
            &all,
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        assert!(output.starts_with("Status  Name"));

        let mut config = UserConfig::default();
        config.list.status = StatusColumnMode::Never;
        let output =
            format_list_with_width(&catalog(), &all, &ShellType::Bash, &config, false, None)
                .unwrap();
        assert!(output.starts_with("Name  Command"));

        let mut explicit = command(OutputFormat::Human);
        explicit.columns = Some(vec![ListColumn::Status, ListColumn::Name]);
        let output = format_list_with_width(
            &catalog(),
            &explicit,
            &ShellType::Bash,
            &config,
            false,
            None,
        )
        .unwrap();
        assert!(output.starts_with("Status  Name"));

        config.list.status = StatusColumnMode::Always;
        let output = format_list_with_width(
            &catalog(),
            &command(OutputFormat::Human),
            &ShellType::Bash,
            &config,
            false,
            None,
        )
        .unwrap();
        assert!(output.starts_with("Status  Name"));
    }

    #[test]
    fn bash_hides_an_explicit_global_column() {
        let mut explicit = command(OutputFormat::Human);
        explicit.columns = Some(vec![ListColumn::Name, ListColumn::Global]);
        let output = format_list_with_width(
            &catalog(),
            &explicit,
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        assert_eq!(output, "Name\ntest\n");

        explicit.columns = Some(vec![ListColumn::Global]);
        let output = format_list_with_width(
            &catalog(),
            &explicit,
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn json_always_contains_metadata() {
        let output = format_list_with_width(
            &catalog(),
            &command(OutputFormat::Json),
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(value[0]["global"], false);
        assert_eq!(value[0]["tags"], serde_json::json!(["dev", "rust"]));
        assert_eq!(value[0]["description"], "Run the complete test suite");
    }

    #[test]
    fn tag_filters_require_every_tag() {
        let mut cmd = command(OutputFormat::Json);
        cmd.tag = vec!["dev".into(), "missing".into()];
        let output = format_list_with_width(
            &catalog(),
            &cmd,
            &ShellType::Bash,
            &UserConfig::default(),
            false,
            None,
        )
        .unwrap();
        assert_eq!(output, "[]\n");
    }

    #[test]
    fn invalid_patterns_and_empty_tables_are_handled() {
        let mut cmd = command(OutputFormat::Human);
        cmd.pattern = Some("[".into());
        assert_eq!(
            format_list_with_width(
                &catalog(),
                &cmd,
                &ShellType::Bash,
                &UserConfig::default(),
                false,
                None,
            ),
            Err(Failure::InvalidPattern)
        );
        assert_eq!(
            format_human(
                &[],
                &ListColumn::DEFAULTS,
                &UserConfig::default(),
                false,
                None
            ),
            ""
        );
    }

    #[test]
    fn cells_are_single_line_styled_and_truncated_at_every_width() {
        let config = UserConfig::default();
        let mut alias = Alias::new("line one\nline two\tend".into(), false, true);
        alias.description = Some("first\rsecond".into());

        assert_eq!(raw_cell(ListColumn::Status, "name", &alias, &config), "✘");
        assert_eq!(raw_cell(ListColumn::Global, "name", &alias, &config), "⦾");
        assert_eq!(
            raw_cell(ListColumn::Command, "name", &alias, &config),
            "line one line two end"
        );
        assert_eq!(
            raw_cell(ListColumn::Description, "name", &alias, &config),
            "first second"
        );
        assert_eq!(truncate("abc", 3), "abc");
        assert_eq!(truncate("abc", 0), "");
        assert_eq!(truncate("abc", 1), "…");
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(
            styled_cell(ListColumn::Global, "⦾", &alias, &config, false),
            "⦾"
        );
        assert_eq!(
            styled_cell(ListColumn::Status, "✘", &alias, &config, false),
            "✘"
        );
    }

    #[test]
    fn extremely_narrow_tables_shrink_below_header_widths() {
        let columns = ListColumn::DEFAULTS;
        let rows = vec![vec!["long value".into(); columns.len()]];
        let widths = column_widths(&columns, &rows, Some(1));
        assert_eq!(widths, vec![1; columns.len()]);

        let columns = [ListColumn::Name];
        let rows = vec![vec!["long value".into()]];
        assert_eq!(column_widths(&columns, &rows, Some(1)), [1]);
    }
}
