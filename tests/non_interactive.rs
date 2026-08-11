use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_aliasmgr(catalog: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(args)
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_SHELL", "bash")
        .output()
        .unwrap()
}

fn assert_input_required(output: &Output, action: &str) {
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(action));
    assert!(stderr.contains("--no-input was supplied"));
}

#[test]
fn yes_accepts_overwrite_and_remove_all() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls\"\n").unwrap();
    assert!(
        run_aliasmgr(&catalog, &["add", "ll", "ls -la", "--yes"])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(&catalog)
            .unwrap()
            .contains("ll = \"ls -la\"")
    );
    assert!(
        run_aliasmgr(&catalog, &["remove", "all", "-y"])
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(&catalog).unwrap(), "");
}

#[test]
fn no_declines_prompts_without_changes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = "ll = \"ls\"\n";
    fs::write(&catalog, original).unwrap();

    let overwrite = run_aliasmgr(&catalog, &["add", "ll", "ls -la", "--no"]);
    assert!(overwrite.status.success(), "{overwrite:?}");
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);

    let remove = run_aliasmgr(&catalog, &["remove", "all", "-n"]);
    assert!(remove.status.success(), "{remove:?}");
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);
}

#[test]
fn no_input_refuses_prompts_without_changes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls\"\n").unwrap();
    let overwrite = run_aliasmgr(&catalog, &["add", "ll", "ls -la", "--no-input"]);
    assert_input_required(&overwrite, "overwrite an existing alias");
    assert_eq!(fs::read_to_string(&catalog).unwrap(), "ll = \"ls\"\n");
    let remove = run_aliasmgr(&catalog, &["remove", "all", "-N"]);
    assert_input_required(&remove, "remove all aliases");
    assert_eq!(fs::read_to_string(&catalog).unwrap(), "ll = \"ls\"\n");
}

#[test]
fn metadata_editing_never_prompts() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls\"\n").unwrap();
    let output = run_aliasmgr(
        &catalog,
        &[
            "edit",
            "ll",
            "--description",
            "List files",
            "--add-tag",
            "shell",
            "--no-input",
        ],
    );
    assert!(output.status.success(), "{output:?}");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("description = \"List files\""));
    assert!(content.contains("tags = [\"shell\"]"));
}

#[test]
fn prompt_controls_conflict_across_command_scopes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "").unwrap();
    for args in [
        &["--yes", "list", "--no"][..],
        &["--no", "list", "--no-input"][..],
        &["--yes", "list", "--no-input"][..],
    ] {
        let output = run_aliasmgr(&catalog, args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
    }

    let shell_sync = run_aliasmgr(&catalog, &["shell-sync", "--force", "--if-changed"]);
    assert_eq!(shell_sync.status.code(), Some(2));
}

#[test]
fn non_terminal_prompt_fails_cleanly_without_changes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = "ll = \"ls\"\n";
    fs::write(&catalog, original).unwrap();

    let output = run_aliasmgr(&catalog, &["add", "ll", "ls -la"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Could not prompt to overwrite an existing alias"));
    assert!(stderr.contains("--yes"));
    assert!(stderr.contains("--no"));
    assert!(stderr.contains("--no-input"));
    assert!(!stderr.contains("panicked"));
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);
}

#[test]
fn declining_a_missing_catalog_path_is_a_successful_noop() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("missing.toml");

    let output = run_aliasmgr(&catalog, &["list", "--no"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Catalog path was declined. No changes made.\n"
    );
    assert!(!catalog.exists());
}

#[test]
fn yes_does_not_force_shell_reconciliation() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls -la\"\n").unwrap();

    let forced = run_aliasmgr(&catalog, &["shell-sync", "--force"]);
    assert!(forced.status.success(), "{forced:?}");
    let script = String::from_utf8(forced.stdout).unwrap();
    let revision = script
        .lines()
        .find_map(|line| {
            line.strip_prefix("    __aliasmgr_catalog_revision='")
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap();

    let if_changed = Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(["--yes", "shell-sync", "--if-changed"])
        .env("ALIASMGR_CATALOG_PATH", &catalog)
        .env("ALIASMGR_SHELL", "bash")
        .env("ALIASMGR_CATALOG_REVISION", revision)
        .output()
        .unwrap();
    assert!(if_changed.status.success(), "{if_changed:?}");
    assert!(if_changed.stdout.is_empty());
}
