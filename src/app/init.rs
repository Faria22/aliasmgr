use super::file_path::CATALOG_FILE_ENV_VAR;
use super::shell::{SHELL_ENV_VAR, ShellType, shell_quote};
use crate::cli::init::InitCommand;

const COMMON_SHELL_FUNCTIONS: &str = r#"
: "${__aliasmgr_managed_aliases:=}"
: "${__aliasmgr_catalog_revision:=}"
: "${__aliasmgr_sync_in_progress:=0}"

__aliasmgr_apply_sync() {
    local mode="$1"
    local changes
    local sync_status

    changes="$(
        ALIASMGR_MANAGED_ALIASES="$__aliasmgr_managed_aliases" \
        ALIASMGR_CATALOG_REVISION="$__aliasmgr_catalog_revision" \
        "$__aliasmgr_cmd" shell-sync "$mode"
    )"
    sync_status=$?

    if [ "$sync_status" -ne 0 ]; then
        return "$sync_status"
    fi

    if [ -n "$changes" ]; then
        eval "$changes"
    fi
}

aliasmgr() {
    if [ "$#" -gt 0 ] && [ "$1" = "sync" ]; then
        shift
        if [ "$#" -ne 0 ]; then
            "$__aliasmgr_cmd" sync "$@"
            return $?
        fi
        __aliasmgr_apply_sync --force
        return $?
    fi

    "$__aliasmgr_cmd" "$@"
}

__aliasmgr_prompt_sync() {
    local previous_status=$?

    if [ "$__aliasmgr_sync_in_progress" -eq 1 ]; then
        return "$previous_status"
    fi

    __aliasmgr_sync_in_progress=1
    __aliasmgr_apply_sync --if-changed || true
    __aliasmgr_sync_in_progress=0
    return "$previous_status"
}
"#;

const BASH_PROMPT_HOOK: &str = r#"
case "$(declare -p PROMPT_COMMAND 2>/dev/null)" in
    'declare -a '*)
        __aliasmgr_hook_registered=0
        for __aliasmgr_hook in "${PROMPT_COMMAND[@]}"; do
            if [ "$__aliasmgr_hook" = "__aliasmgr_prompt_sync" ]; then
                __aliasmgr_hook_registered=1
                break
            fi
        done
        if [ "$__aliasmgr_hook_registered" -eq 0 ]; then
            PROMPT_COMMAND=(__aliasmgr_prompt_sync "${PROMPT_COMMAND[@]}")
        fi
        unset __aliasmgr_hook __aliasmgr_hook_registered
        ;;
    *)
        case ";${PROMPT_COMMAND-};" in
            *';__aliasmgr_prompt_sync;'*) ;;
            *) PROMPT_COMMAND="__aliasmgr_prompt_sync${PROMPT_COMMAND:+;$PROMPT_COMMAND}" ;;
        esac
        ;;
esac
"#;

const ZSH_PROMPT_HOOK: &str = r#"
autoload -Uz add-zsh-hook
add-zsh-hook -d precmd __aliasmgr_prompt_sync 2>/dev/null
add-zsh-hook precmd __aliasmgr_prompt_sync
precmd_functions=(__aliasmgr_prompt_sync ${precmd_functions:#__aliasmgr_prompt_sync})
"#;

fn helper_shell_command(shell: &ShellType) -> &'static str {
    match shell {
        ShellType::Zsh => "whence -p aliasmgr",
        ShellType::Bash => "type -P aliasmgr",
    }
}

pub fn handle_init(cmd: InitCommand) -> String {
    let mut content = String::from("# Alias Manager Initialization Script\n");
    content += &format!("export {}={}\n", SHELL_ENV_VAR, cmd.shell);
    if let Some(catalog_path) = cmd.catalog {
        content += &format!(
            "export {}={}\n",
            CATALOG_FILE_ENV_VAR,
            shell_quote(&catalog_path.to_string_lossy())
        );
    }

    content += "\n# Resolve the executable before defining the wrapper function\n";
    content += &format!("__aliasmgr_cmd=$({})\n", helper_shell_command(&cmd.shell));
    content += COMMON_SHELL_FUNCTIONS;

    if !cmd.no_auto_sync {
        content += "\n# Synchronize when the shell is about to display a prompt\n";
        content += match cmd.shell {
            ShellType::Bash => BASH_PROMPT_HOOK,
            ShellType::Zsh => ZSH_PROMPT_HOOK,
        };
    }

    content += "\n# Load aliases into this shell\n";
    content += "__aliasmgr_apply_sync --force";
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn command(shell: ShellType) -> InitCommand {
        InitCommand {
            shell,
            catalog: None,
            no_auto_sync: false,
        }
    }

    #[test]
    fn bash_init_installs_wrapper_prompt_hook_and_initial_sync() {
        let output = handle_init(command(ShellType::Bash));
        assert!(output.contains("__aliasmgr_cmd=$(type -P aliasmgr)"));
        assert!(output.contains("aliasmgr()"));
        assert!(output.contains("shell-sync \"$mode\""));
        assert!(output.contains("PROMPT_COMMAND"));
        assert!(output.contains("__aliasmgr_apply_sync --force"));
        assert!(!output.contains("3>&1"));
    }

    #[test]
    fn zsh_init_uses_precommand_hook() {
        let output = handle_init(command(ShellType::Zsh));
        assert!(output.contains("__aliasmgr_cmd=$(whence -p aliasmgr)"));
        assert!(output.contains("add-zsh-hook precmd __aliasmgr_prompt_sync"));
    }

    #[test]
    fn no_auto_sync_keeps_initial_sync_without_prompt_hook() {
        let output = handle_init(InitCommand {
            no_auto_sync: true,
            ..command(ShellType::Bash)
        });
        assert!(!output.contains("PROMPT_COMMAND"));
        assert!(output.contains("__aliasmgr_apply_sync --force"));
    }

    #[test]
    fn custom_catalog_path_is_shell_quoted() {
        let output = handle_init(InitCommand {
            shell: ShellType::Bash,
            catalog: Some(PathBuf::from("/catalog/it's here.toml")),
            no_auto_sync: false,
        });
        assert!(output.contains("ALIASMGR_CATALOG_PATH='/catalog/it'\"'\"'s here.toml'"));
    }
}
