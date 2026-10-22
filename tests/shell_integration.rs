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
    // Homebrew can shadow the macOS system shells on PATH.
    let program = if cfg!(target_os = "macos") {
        match shell {
            "bash" => "/bin/bash",
            "zsh" => "/bin/zsh",
            _ => unreachable!(),
        }
    } else {
        shell
    };
    let mut command = Command::new(program);
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
fn missing_custom_catalog_initializes_and_syncs_silently_in_both_shells() {
    for shell in ["bash", "zsh"] {
        let directory = tempfile::tempdir().unwrap();
        let catalog = directory.path().join("missing.toml");
        let script = format!(
            r#"
unset ALIASMGR_MANAGED_ALIASES ALIASMGR_CATALOG_REVISION
unset __aliasmgr_managed_aliases __aliasmgr_catalog_revision
"$1" init {shell} --catalog "$2" > "$2.init" || exit 80
source "$2.init" || exit 81
[ -n "$__aliasmgr_catalog_revision" ] || exit 82
[ -z "$__aliasmgr_managed_aliases" ] || exit 83
__aliasmgr_prompt_sync
__aliasmgr_prompt_sync
aliasmgr sync || exit 84
[ ! -e "$2" ] || exit 85
"#
        );
        let output = run_shell(shell, &script, &catalog).unwrap();
        assert!(output.stdout.is_empty(), "{shell}: {output:?}");
        assert!(output.stderr.is_empty(), "{shell}: {output:?}");
        assert_success(output);
        assert!(!catalog.exists());
        assert!(!directory.path().join("missing.toml.lock").exists());
    }
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

remove_output="$(aliasmgr remove alias -p 'b*' -t rust --yes)"
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
    assert_success(
        run_shell("zsh", script, catalog.path()).expect("Zsh is required for shell tests"),
    );
}

#[test]
fn bash_init_preserves_scalar_prompt_hook_and_is_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("catalog.toml");
    std::fs::write(&catalog, "").unwrap();
    let script = r#"
existing_hook() { hook_ran=yes; }
PROMPT_COMMAND=existing_hook
"$1" init bash --catalog "$2" > "$2.init" || exit 80
source "$2.init" || exit 81
"$1" init bash --catalog "$2" > "$2.init" || exit 80
source "$2.init" || exit 81
[ "$PROMPT_COMMAND" = '__aliasmgr_prompt_sync;existing_hook' ] || exit 60
aliasmgr add smoke 'echo smoke' || exit 61
eval "$PROMPT_COMMAND"
[ "$hook_ran" = yes ] || exit 62
alias smoke | command grep -q 'echo smoke' || exit 63
"#;
    assert_success(run_shell("bash", script, &catalog).unwrap());
}

#[test]
#[cfg(target_os = "macos")]
fn macos_system_bash_3_2_sources_init() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("catalog.toml");
    std::fs::write(&catalog, "").unwrap();
    let script = r#"
[[ "$BASH_VERSION" == 3.2.* ]] || exit 70
unset PROMPT_COMMAND
"$1" init bash --catalog "$2" > "$2.init" || exit 80
source "$2.init" || exit 81
[ "$PROMPT_COMMAND" = __aliasmgr_prompt_sync ] || exit 71
aliasmgr add smoke 'echo smoke' || exit 72
"$PROMPT_COMMAND"
alias smoke | command grep -q 'echo smoke' || exit 73
"#;
    assert_success(run_shell("bash", script, &catalog).unwrap());
}
