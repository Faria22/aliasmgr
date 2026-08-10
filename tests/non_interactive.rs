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
fn force_accepts_overwrite_and_remove_all() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls\"\n").unwrap();
    assert!(
        run_aliasmgr(&catalog, &["add", "ll", "ls -la", "--force"])
            .status
            .success()
    );
    assert!(
        fs::read_to_string(&catalog)
            .unwrap()
            .contains("ll = \"ls -la\"")
    );
    assert!(
        run_aliasmgr(&catalog, &["remove", "all", "--force"])
            .status
            .success()
    );
    assert_eq!(fs::read_to_string(&catalog).unwrap(), "");
}

#[test]
fn no_input_refuses_prompts_without_changes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls\"\n").unwrap();
    let overwrite = run_aliasmgr(&catalog, &["add", "ll", "ls -la", "--no-input"]);
    assert_input_required(&overwrite, "overwrite an existing alias");
    assert_eq!(fs::read_to_string(&catalog).unwrap(), "ll = \"ls\"\n");
    let remove = run_aliasmgr(&catalog, &["remove", "all", "--no-input"]);
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
    let output = run_aliasmgr(&catalog, &["--force", "list", "--no-input"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("--force cannot be used with --no-input")
    );
    let shell_sync = run_aliasmgr(&catalog, &["--force", "shell-sync", "--if-changed"]);
    assert_eq!(shell_sync.status.code(), Some(2));
}
