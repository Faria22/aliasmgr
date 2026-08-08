use std::env;
use std::path::Path;
use std::process::{Command, Output};

fn path_with_binary(binary: &Path) -> String {
    let mut paths = vec![binary.parent().unwrap().to_path_buf()];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    env::join_paths(paths)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn run_shell(shell: &str, script: &str, catalog: &Path) -> std::io::Result<Output> {
    let binary = Path::new(env!("CARGO_BIN_EXE_aliasmgr"));
    let mut command = Command::new(shell);
    match shell {
        "bash" => command.args(["--noprofile", "--norc", "-c", script, "aliasmgr-test"]),
        "zsh" => command.args(["-f", "-c", script, "aliasmgr-test"]),
        _ => unreachable!(),
    };
    command
        .arg(binary)
        .arg(catalog)
        .env("PATH", path_with_binary(binary))
        .output()
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "shell integration failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn bash_prompt_sync_reconciles_catalog_and_preserves_hooks_and_status() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
existing_hook() { :; }
PROMPT_COMMAND=(existing_hook)
eval "$("$1" init bash --catalog "$2")"
[ "${#PROMPT_COMMAND[@]}" -eq 2 ] || exit 10
[ "${PROMPT_COMMAND[0]}" = __aliasmgr_prompt_sync ] || exit 11
[ "${PROMPT_COMMAND[1]}" = existing_hook ] || exit 12

eval "$("$1" init bash --catalog "$2")"
[ "${#PROMPT_COMMAND[@]}" -eq 2 ] || exit 13

aliasmgr add alias smoke 'echo smoke'
__aliasmgr_prompt_sync
alias smoke | command grep -q 'echo smoke' || exit 14

alias smoke='echo overwritten'
aliasmgr sync
alias smoke | command grep -q 'echo smoke' || exit 17

false
__aliasmgr_prompt_sync
[ "$?" -eq 1 ] || exit 15

aliasmgr remove alias smoke
__aliasmgr_prompt_sync
! alias smoke 2>/dev/null || exit 16
"#;
    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn removing_disabled_group_with_reassign_activates_enabled_alias() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add group dormant --disabled
aliasmgr add alias wake 'echo awake' --group dormant
__aliasmgr_prompt_sync
! alias wake 2>/dev/null || exit 30

aliasmgr remove group dormant --reassign
__aliasmgr_prompt_sync
alias wake | command grep -q 'echo awake' || exit 31
"#;

    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn removing_enabled_group_with_reassign_preserves_enabled_alias() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add group active
aliasmgr add alias stay 'echo present' --group active
__aliasmgr_prompt_sync
alias stay | command grep -q 'echo present' || exit 32

aliasmgr remove group active --reassign
__aliasmgr_prompt_sync
alias stay | command grep -q 'echo present' || exit 33
"#;

    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn removing_group_with_reassign_keeps_disabled_alias_inactive() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add group active
aliasmgr add alias sleeping 'echo asleep' --group active --disabled
__aliasmgr_prompt_sync
! alias sleeping 2>/dev/null || exit 34

aliasmgr remove group active --reassign
__aliasmgr_prompt_sync
! alias sleeping 2>/dev/null || exit 35
"#;

    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn zsh_prompt_sync_reconciles_regular_and_global_aliases() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
existing_hook() { :; }
precmd_functions=(existing_hook)
eval "$("$1" init zsh --catalog "$2")"
[ "${precmd_functions[1]}" = __aliasmgr_prompt_sync ] || exit 20
__aliasmgr_matches=(${(M)precmd_functions:#__aliasmgr_prompt_sync})
[ "${#__aliasmgr_matches[@]}" -eq 1 ] || exit 21

eval "$("$1" init zsh --catalog "$2")"
__aliasmgr_matches=(${(M)precmd_functions:#__aliasmgr_prompt_sync})
[ "${#__aliasmgr_matches[@]}" -eq 1 ] || exit 22

aliasmgr add alias smoke 'echo smoke'
aliasmgr add alias glob '*.rs' --global
__aliasmgr_prompt_sync
alias smoke | command grep -q 'echo smoke' || exit 23
alias -g glob | command grep -Fq '*.rs' || exit 24

false
__aliasmgr_prompt_sync
[ "$?" -eq 1 ] || exit 25
"#;

    match run_shell("zsh", script, catalog.path()) {
        Ok(output) => assert_success(output),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to run zsh: {error}"),
    }
}
