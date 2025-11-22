use clap::ValueEnum;
use log::{debug, error, warn};
use std::fmt;
use std::os::fd::BorrowedFd;

#[derive(Clone, ValueEnum)]
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

pub fn send_alias_deltas_to_shell(deltas: &str) {
    let fd3 = unsafe { BorrowedFd::borrow_raw(3) };
    if let Err(e) = nix::unistd::write(fd3, deltas.as_bytes()) {
        error!(
            "Failed to send alias deltas to shell. Make sure to use aliasmgr init in your shell configuration."
        );
        error!("{}", e);
        return;
    }
    debug!("Sent alias deltas to shell: {}", deltas);
}
