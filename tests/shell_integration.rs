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
aliasmgr add smoke 'echo smoke' --tag test
__aliasmgr_prompt_sync
alias smoke | command grep -q 'echo smoke' || exit 11
false
__aliasmgr_prompt_sync
[ "$?" -eq 1 ] || exit 12
aliasmgr remove alias smoke
__aliasmgr_prompt_sync
! alias smoke 2>/dev/null || exit 13
"#;
    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn tag_enable_disable_and_filtered_remove_reconcile_aliases() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add build 'cargo build' --tag dev --tag rust
aliasmgr add bench 'cargo bench' --tag dev --tag rust --disabled
aliasmgr add deploy deploy --tag dev
__aliasmgr_prompt_sync
alias build >/dev/null || exit 20
! alias bench 2>/dev/null || exit 21

disable_output="$(aliasmgr disable tag rust)"
[ "$disable_output" = "Disabled 1 of 2 aliases tagged 'rust'." ] || exit 22
__aliasmgr_prompt_sync
! alias build 2>/dev/null || exit 23
alias deploy >/dev/null || exit 24

enable_output="$(aliasmgr enable alias --tag dev --tag rust)"
[ "$enable_output" = 'Enabled 2 of 2 matching aliases.' ] || exit 25
__aliasmgr_prompt_sync
alias build >/dev/null || exit 26
alias bench >/dev/null || exit 27

remove_output="$(aliasmgr remove alias --pattern 'b*' --tag rust --force)"
[ "$remove_output" = 'Removed 2 of 2 matching aliases.' ] || exit 28
__aliasmgr_prompt_sync
! alias build 2>/dev/null || exit 29
! alias bench 2>/dev/null || exit 30
alias deploy >/dev/null || exit 31
"#;
    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn bulk_enable_and_disable_are_idempotent() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init bash --catalog "$2")"
aliasmgr add top 'echo top'
[ "$(aliasmgr disable all)" = 'All aliases are now disabled.' ] || exit 40
[ "$(aliasmgr disable all)" = 'All aliases are already disabled.' ] || exit 41
[ "$(aliasmgr enable all)" = 'All aliases are now enabled.' ] || exit 42
[ "$(aliasmgr enable all)" = 'All aliases are already enabled.' ] || exit 43
"#;
    assert_success(run_shell("bash", script, catalog.path()).unwrap());
}

#[test]
fn zsh_prompt_sync_reconciles_regular_and_global_aliases() {
    let catalog = tempfile::NamedTempFile::new().unwrap();
    let script = r#"
eval "$("$1" init zsh --catalog "$2")"
aliasmgr add smoke 'echo smoke'
aliasmgr add glob '*.rs' --global
__aliasmgr_prompt_sync
alias smoke | command grep -q 'echo smoke' || exit 50
alias -g glob | command grep -Fq '*.rs' || exit 51
"#;
    match run_shell("zsh", script, catalog.path()) {
        Ok(output) => assert_success(output),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to run zsh: {error}"),
    }
}
