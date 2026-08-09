use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use log::warn;
use owo_colors::{DynColors, Style};
use serde::Deserialize;

pub const CONFIG_FILE_ENV_VAR: &str = "ALIASMGR_CONFIG_PATH";

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl ColorMode {
    pub fn enabled(self) -> bool {
        self.enabled_for(
            std::io::stdout().is_terminal(),
            env::var_os("NO_COLOR").is_some(),
        )
    }

    fn enabled_for(self, is_terminal: bool, no_color: bool) -> bool {
        match self {
            Self::Auto => is_terminal && !no_color,
            Self::Always => true,
            Self::Never => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolConfig {
    pub enabled: String,
    pub disabled: String,
    pub global: String,
}

impl Default for SymbolConfig {
    fn default() -> Self {
        Self {
            enabled: "✔".into(),
            disabled: "✘".into(),
            global: "⦾".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateStyle {
    pub foreground: String,
    pub bold: bool,
}

impl StateStyle {
    pub fn render(&self, value: &str, colors_enabled: bool) -> String {
        if !colors_enabled {
            return value.to_owned();
        }

        let color = self
            .foreground
            .parse::<DynColors>()
            .expect("configuration colors are validated while loading");
        let mut style = Style::new().color(color);
        if self.bold {
            style = style.bold();
        }
        style.style(value).to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleConfig {
    pub enabled: StateStyle,
    pub disabled: StateStyle,
    pub global: StateStyle,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            enabled: StateStyle {
                foreground: "green".into(),
                bold: true,
            },
            disabled: StateStyle {
                foreground: "red".into(),
                bold: true,
            },
            global: StateStyle {
                foreground: "blue".into(),
                bold: true,
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserConfig {
    pub color: ColorMode,
    pub symbols: SymbolConfig,
    pub styles: StyleConfig,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawConfig {
    color: RawColorConfig,
    symbols: RawSymbolConfig,
    styles: RawStyleConfig,
    #[serde(flatten)]
    unknown: HashMap<String, toml::Value>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawColorConfig {
    mode: Option<ColorMode>,
    #[serde(flatten)]
    unknown: HashMap<String, toml::Value>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawSymbolConfig {
    enabled: Option<String>,
    disabled: Option<String>,
    global: Option<String>,
    #[serde(flatten)]
    unknown: HashMap<String, toml::Value>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawStyleConfig {
    enabled: Option<RawStateStyle>,
    disabled: Option<RawStateStyle>,
    global: Option<RawStateStyle>,
    #[serde(flatten)]
    unknown: HashMap<String, toml::Value>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct RawStateStyle {
    foreground: Option<String>,
    bold: Option<bool>,
    #[serde(flatten)]
    unknown: HashMap<String, toml::Value>,
}

fn warn_unknown(section: Option<&str>, fields: &HashMap<String, toml::Value>) {
    let mut names = fields.keys().collect::<Vec<_>>();
    names.sort_unstable();
    for name in names {
        if let Some(section) = section {
            warn!("Unknown configuration setting '{section}.{name}' ignored.");
        } else {
            warn!("Unknown configuration setting '{name}' ignored.");
        }
    }
}

fn apply_style(name: &str, target: &mut StateStyle, raw: Option<RawStateStyle>) -> Result<()> {
    let Some(raw) = raw else {
        return Ok(());
    };
    warn_unknown(Some(&format!("styles.{name}")), &raw.unknown);
    if let Some(foreground) = raw.foreground {
        foreground.parse::<DynColors>().map_err(|_| {
            anyhow::anyhow!(
                "invalid color '{foreground}' for 'styles.{name}.foreground'; use an ANSI color name or #RRGGBB"
            )
        })?;
        target.foreground = foreground;
    }
    if let Some(bold) = raw.bold {
        target.bold = bold;
    }
    Ok(())
}

fn parse_config(content: &str) -> Result<UserConfig> {
    let raw: RawConfig = toml::from_str(content)?;
    warn_unknown(None, &raw.unknown);
    warn_unknown(Some("color"), &raw.color.unknown);
    warn_unknown(Some("symbols"), &raw.symbols.unknown);
    warn_unknown(Some("styles"), &raw.styles.unknown);

    let mut config = UserConfig::default();
    if let Some(mode) = raw.color.mode {
        config.color = mode;
    }
    if let Some(enabled) = raw.symbols.enabled {
        config.symbols.enabled = enabled;
    }
    if let Some(disabled) = raw.symbols.disabled {
        config.symbols.disabled = disabled;
    }
    if let Some(global) = raw.symbols.global {
        config.symbols.global = global;
    }

    apply_style("enabled", &mut config.styles.enabled, raw.styles.enabled)?;
    apply_style("disabled", &mut config.styles.disabled, raw.styles.disabled)?;
    apply_style("global", &mut config.styles.global, raw.styles.global)?;
    Ok(config)
}

pub fn config_path(path: Option<&Path>) -> PathBuf {
    path.map(Path::to_path_buf).unwrap_or_else(|| {
        cross_xdg::BaseDirs::new()
            .expect("could not determine XDG base directories")
            .config_home()
            .join("aliasmgr")
            .join("config.toml")
    })
}

fn load_config_from(path: &Path, required: bool) -> Result<UserConfig> {
    if !path.exists() {
        if required {
            bail!("configured file '{}' does not exist", path.display());
        }
        return Ok(UserConfig::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("could not read configuration '{}'", path.display()))?;
    parse_config(&content)
        .with_context(|| format!("could not parse configuration '{}'", path.display()))
}

pub fn load_config() -> Result<UserConfig> {
    let explicit = env::var_os(CONFIG_FILE_ENV_VAR).map(PathBuf::from);
    let path = config_path(explicit.as_deref());
    load_config_from(&path, explicit.is_some())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_existing_presentation() {
        let config = UserConfig::default();
        assert_eq!(config.color, ColorMode::Auto);
        assert_eq!(config.symbols.enabled, "✔");
        assert_eq!(config.symbols.disabled, "✘");
        assert_eq!(config.symbols.global, "⦾");
        assert_eq!(config.styles.enabled.foreground, "green");
        assert!(config.styles.enabled.bold);
    }

    #[test]
    fn partial_configuration_inherits_defaults() {
        let config = parse_config(
            r##"
            [color]
            mode = "never"
            [symbols]
            enabled = "+"
            disabled = "-"
            [styles.disabled]
            foreground = "#ff00aa"
            bold = false
            "##,
        )
        .unwrap();

        assert_eq!(config.color, ColorMode::Never);
        assert_eq!(config.symbols.enabled, "+");
        assert_eq!(config.symbols.disabled, "-");
        assert_eq!(config.styles.disabled.foreground, "#ff00aa");
        assert!(!config.styles.disabled.bold);
    }

    #[test]
    fn unknown_settings_are_accepted() {
        let config = parse_config(
            r#"
            future = true
            [symbols]
            enabled = "+"
            future = "value"
            "#,
        )
        .unwrap();
        assert_eq!(config.symbols.enabled, "+");
    }

    #[test]
    fn invalid_known_settings_fail() {
        assert!(parse_config("[color]\nmode = \"sometimes\"\n").is_err());
        assert!(parse_config("[styles.enabled]\nforeground = \"not-a-color\"\n").is_err());
        assert!(parse_config("[styles.enabled]\nbold = \"yes\"\n").is_err());
    }

    #[test]
    fn missing_default_configuration_uses_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("missing.toml");
        assert_eq!(
            load_config_from(&path, false).unwrap(),
            UserConfig::default()
        );
        assert!(!path.exists());
    }

    #[test]
    fn missing_explicit_configuration_fails() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("missing.toml");
        assert!(load_config_from(&path, true).is_err());
    }

    #[test]
    fn loads_configuration_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        fs::write(&path, "[symbols]\nglobal = \"G\"\n").unwrap();
        let config = load_config_from(&path, true).unwrap();
        assert_eq!(config.symbols.global, "G");
    }

    #[test]
    fn custom_and_default_paths_are_resolved() {
        let custom = Path::new("/custom/config.toml");
        assert_eq!(config_path(Some(custom)), custom);
        assert!(config_path(None).ends_with(".config/aliasmgr/config.toml"));
    }

    #[test]
    fn color_mode_respects_explicit_modes() {
        assert!(ColorMode::Always.enabled());
        assert!(!ColorMode::Never.enabled());
        assert!(ColorMode::Auto.enabled_for(true, false));
        assert!(!ColorMode::Auto.enabled_for(false, false));
        assert!(!ColorMode::Auto.enabled_for(true, true));
    }

    #[test]
    fn styles_can_render_plain_or_colored_symbols() {
        let style = StateStyle {
            foreground: "red".into(),
            bold: true,
        };
        assert_eq!(style.render("x", false), "x");
        assert_eq!(style.render("x", true), "\u{1b}[31;1mx\u{1b}[0m");
    }
}
