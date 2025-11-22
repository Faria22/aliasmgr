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
