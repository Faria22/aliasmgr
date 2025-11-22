use super::config_path::CONFIG_FILE_ENV_VAR;
use super::shell::{SHELL_ENV_VAR, ShellType};
use crate::cli::init::InitCommand;

const ALIASMGR_SHELL_FUNCTION: &'static str = {
    r#"
# Define the aliasmgr shell function using the helper command
# This function captures alias deltas from file descriptor 3
aliasmgr() {
    # Run aliasmgr and capture deltas from FD3
    local deltas

    # Capture output from FD3 without interfering with standard output
    {
        deltas="$("$__aliasmgr_cmd" "$@" 3>&1 1>&4)"
    } 4>&1

    # Apply alias deltas if any
    if [ -n "$deltas" ]; then
        eval "$deltas"
    fi
}
"#
};

pub fn handle_init(cmd: InitCommand) -> String {
    let mut content = String::from("# Alias Manager Initialization Script\n");
    content += &format!("export {}={}\n", SHELL_ENV_VAR, cmd.shell);
    if let Some(config_path) = cmd.config {
        content += &format!("export {}={:?}\n", CONFIG_FILE_ENV_VAR, config_path);
    }

    content += "\n";
    content += "# Alias helper shell command\n";
    content += if cmd.shell == ShellType::Zsh {
        "# For zsh, we use 'whence -p' to find the command path\n__aliasmgr_cmd=$(whence -p aliasmgr)\n"
    } else {
        "# For bash and other shells, we use 'command -p' to find the command path\n__aliasmgr_cmd=$(command -p aliasmgr)\n"
    };

    content += ALIASMGR_SHELL_FUNCTION;

    content += "\n# Sync aliases on shell startup\n";
    content += "aliasmgr sync";

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_handle_init_bash_no_config() {
        let cmd = InitCommand {
            shell: ShellType::Bash,
            config: None,
        };
        let output = handle_init(cmd);
        assert!(output.contains(&ShellType::Bash.to_string()));
        assert!(output.contains("__aliasmgr_cmd=$(command -p aliasmgr)"));
        assert!(output.contains("aliasmgr sync"));
    }

    #[test]
    fn test_handle_init_zsh_no_config() {
        let cmd = InitCommand {
            shell: ShellType::Zsh,
            config: None,
        };
        let output = handle_init(cmd);
        assert!(output.contains(&ShellType::Zsh.to_string()));
        assert!(output.contains("__aliasmgr_cmd=$(whence -p aliasmgr)"));
        assert!(output.contains("aliasmgr sync"));
    }

    #[test]
    fn test_handle_init_with_config() {
        let path = PathBuf::from("/config/path");
        let cmd = InitCommand {
            shell: ShellType::Bash,
            config: Some(path.clone()),
        };
        let output = handle_init(cmd);
        assert!(output.contains(&ShellType::Bash.to_string()));
        assert!(output.contains(path.to_str().unwrap()));
        assert!(output.contains("aliasmgr sync"));
    }

    #[test]
    fn test_handle_init_with_config_zsh() {
        let path = PathBuf::from("/config/path");
        let cmd = InitCommand {
            shell: ShellType::Zsh,
            config: Some(path.clone()),
        };
        let output = handle_init(cmd);
        assert!(output.contains(&ShellType::Zsh.to_string()));
        assert!(output.contains(path.to_str().unwrap()));
        assert!(output.contains("aliasmgr sync"));
    }
}
