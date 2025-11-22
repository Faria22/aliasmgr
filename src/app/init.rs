use super::config_path::config_env_var;
use super::shell::shell_env_var;
use crate::cli::init::InitCommand;

fn aliasmgr_shell_function() -> &'static str {
    r#"
# Define the aliasmgr shell function
# This function captures alias deltas from file descriptor 3

__aliasmgr_cmd="$(command -v aliasmgr)"

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
}

pub fn handle_init(cmd: InitCommand) -> String {
    let mut content = String::from("# Alias Manager Initialization Script\n");
    content += &format!("export {}={}\n", shell_env_var(), cmd.shell);
    if let Some(config_path) = cmd.config {
        content += &format!("export {}={:?}\n", config_env_var(), config_path);
    }

    content += aliasmgr_shell_function();

    content += "\n# Sync aliases on shell startup\n";
    content += "aliasmgr sync\n";

    content
}
