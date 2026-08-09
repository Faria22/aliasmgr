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
fn enabling_reassigned_aliases_from_disabled_group_activates_them() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add group dormant --disabled
aliasmgr add alias wake 'echo awake' --group dormant
__aliasmgr_prompt_sync
! alias wake 2>/dev/null || exit 30

aliasmgr remove group dormant --reassign --enable-reassigned
__aliasmgr_prompt_sync
alias wake | command grep -q 'echo awake' || exit 31
"#;

    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn disabling_reassigned_aliases_from_disabled_group_keeps_them_inactive() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add group dormant --disabled
aliasmgr add alias sleep 'echo asleep' --group dormant
__aliasmgr_prompt_sync
! alias sleep 2>/dev/null || exit 36

aliasmgr remove group dormant --reassign --disable-reassigned
__aliasmgr_prompt_sync
! alias sleep 2>/dev/null || exit 37
"#;

    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn bulk_disable_and_enable_reconcile_all_managed_aliases() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add alias top 'echo top'
aliasmgr add group tools --disabled
aliasmgr add alias grouped 'echo grouped' --group tools --disabled
__aliasmgr_prompt_sync
alias top | command grep -q 'echo top' || exit 40
! alias grouped 2>/dev/null || exit 41

disable_output="$(aliasmgr disable all)"
[ "$disable_output" = 'All aliases and groups are now disabled.' ] || exit 42
__aliasmgr_prompt_sync
! alias top 2>/dev/null || exit 43

disable_output="$(aliasmgr disable all)"
[ "$disable_output" = 'All aliases and groups are already disabled.' ] || exit 44

enable_output="$(aliasmgr enable all)"
[ "$enable_output" = 'All aliases and groups are now enabled.' ] || exit 45
__aliasmgr_prompt_sync
alias top | command grep -q 'echo top' || exit 46
alias grouped | command grep -q 'echo grouped' || exit 47

enable_output="$(aliasmgr enable all)"
[ "$enable_output" = 'All aliases and groups are already enabled.' ] || exit 48
"#;

    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn bulk_enable_and_disable_handle_an_empty_catalog() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"

enable_output="$(aliasmgr enable all)"
[ "$enable_output" = 'All aliases and groups are already enabled.' ] || exit 49

disable_output="$(aliasmgr disable all)"
[ "$disable_output" = 'All aliases and groups are already disabled.' ] || exit 50

[ -z "$(aliasmgr --quiet enable all)" ] || exit 51
"#;

    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn filtered_bulk_operations_reconcile_managed_aliases_once() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add group dev
aliasmgr add alias build 'cargo build' --group dev
aliasmgr add alias bench 'cargo bench' --group dev --disabled
aliasmgr add alias deploy 'deploy' --group dev
__aliasmgr_prompt_sync
alias build >/dev/null || exit 52
! alias bench 2>/dev/null || exit 53

disable_output="$(aliasmgr disable alias --pattern 'b*' --group dev)"
[ "$disable_output" = 'Disabled 1 of 2 matching aliases.' ] || exit 54
__aliasmgr_prompt_sync
! alias build 2>/dev/null || exit 55
! alias bench 2>/dev/null || exit 56
alias deploy >/dev/null || exit 57

enable_output="$(aliasmgr enable alias --group dev)"
[ "$enable_output" = 'Enabled 2 of 3 matching aliases.' ] || exit 58
__aliasmgr_prompt_sync
alias build >/dev/null || exit 59
alias bench >/dev/null || exit 60

remove_output="$(aliasmgr remove alias --pattern 'b*' --group dev --force)"
[ "$remove_output" = 'Removed 2 of 2 matching aliases.' ] || exit 61
__aliasmgr_prompt_sync
! alias build 2>/dev/null || exit 62
! alias bench 2>/dev/null || exit 63
alias deploy >/dev/null || exit 64
command grep -q '^\[dev\]$' "$2" || exit 65
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
