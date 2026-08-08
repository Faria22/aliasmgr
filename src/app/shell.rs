use clap::ValueEnum;
use log::warn;
use std::fmt;

#[derive(Clone, ValueEnum, Debug, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
}

impl fmt::Display for ShellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellType::Bash => write!(f, "BASH"),
            ShellType::Zsh => write!(f, "ZSH"),
        }
    }
}

pub const DEFAULT_SHELL: ShellType = ShellType::Bash;

pub const SHELL_ENV_VAR: &str = "ALIASMGR_SHELL";

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn determine_shell() -> ShellType {
    match std::env::var(SHELL_ENV_VAR) {
        Ok(val) => match ShellType::from_str(&val, true) {
            Ok(shell) => shell,
            Err(_) => {
                warn!(
                    "Invalid {} value: {}. Using {} as default shell.",
                    SHELL_ENV_VAR, val, DEFAULT_SHELL
                );
                DEFAULT_SHELL
            }
        },
        Err(_) => {
            warn!(
                "{} environment variable not set. Please set it using the init command.",
                SHELL_ENV_VAR
            );
            warn!("Using {} as default shell.", DEFAULT_SHELL);
            DEFAULT_SHELL
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use temp_env::with_var;

    #[test]
    fn test_shell_type_display() {
        assert_eq!(ShellType::Bash.to_string(), "BASH");
        assert_eq!(ShellType::Zsh.to_string(), "ZSH");
    }

    #[test]
    fn test_shell_quote() {
        assert_eq!(shell_quote("plain value"), "'plain value'");
        assert_eq!(shell_quote("it's quoted"), "'it'\"'\"'s quoted'");
    }

    #[test]
    fn test_determine_shell_default() {
        with_var(SHELL_ENV_VAR, None as Option<&str>, || {
            let shell = determine_shell();
            assert_eq!(shell, DEFAULT_SHELL);
        });
    }

    #[test]
    fn test_determine_shell_invalid() {
        with_var(SHELL_ENV_VAR, Some("INVALID_SHELL"), || {
            let shell = determine_shell();
            assert_eq!(shell, DEFAULT_SHELL);
        });
    }

    #[test]
    fn test_determine_shell_valid() {
        with_var(SHELL_ENV_VAR, Some("ZSH"), || {
            let shell = determine_shell();
            assert_eq!(shell, ShellType::Zsh);
        });
    }

    #[test]
    fn test_determine_shell_case_insensitive() {
        with_var(SHELL_ENV_VAR, Some("bash"), || {
            let shell = determine_shell();
            assert_eq!(shell, ShellType::Bash);
        });
        with_var(SHELL_ENV_VAR, Some("zSh"), || {
            let shell = determine_shell();
            assert_eq!(shell, ShellType::Zsh);
        });
    }
}
