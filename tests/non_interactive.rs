use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_aliasmgr(catalog: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(args)
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_SHELL", "bash")
        .output()
        .expect("aliasmgr command should run")
}

fn assert_input_required(output: &Output, action: &str) {
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("ERROR: Input required to"));
    assert!(
        stderr.contains(action),
        "stderr {stderr:?} did not contain {action:?}"
    );
    assert!(stderr.contains("--no-input was supplied"));
}

#[test]
fn force_accepts_overwrite_missing_group_and_remove_all() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls\"\n").unwrap();

    let overwrite = run_aliasmgr(&catalog, &["add", "ll", "ls -la", "--force"]);
    assert!(overwrite.status.success());
    assert!(
        fs::read_to_string(&catalog)
            .unwrap()
            .contains("ll = \"ls -la\"")
    );

    let missing_group = run_aliasmgr(
        &catalog,
        &[
            "add",
            "build",
            "cargo build",
            "--group",
            "development",
            "--force",
        ],
    );
    assert!(missing_group.status.success());
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("[development]"));
    assert!(content.contains("build = \"cargo build\""));

    let remove_all = run_aliasmgr(&catalog, &["remove", "all", "--force"]);
    assert!(remove_all.status.success());
    assert_eq!(fs::read_to_string(&catalog).unwrap(), "");
}

#[test]
fn no_input_fails_overwrite_missing_group_and_remove_all_without_changes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");

    for (content, args, action) in [
        (
            "ll = \"ls\"\n",
            vec!["add", "ll", "ls -la", "--no-input"],
            "overwrite an existing alias",
        ),
        (
            "ll = \"ls\"\n",
            vec![
                "add",
                "build",
                "cargo build",
                "--group",
                "development",
                "--no-input",
            ],
            "create missing group 'development'",
        ),
        (
            "ll = \"ls\"\n",
            vec!["remove", "all", "--no-input"],
            "remove all aliases and groups",
        ),
    ] {
        fs::write(&catalog, content).unwrap();
        let output = run_aliasmgr(&catalog, &args);
        assert_input_required(&output, action);
        assert_eq!(fs::read_to_string(&catalog).unwrap(), content);
    }
}

#[test]
fn prompt_controls_cover_missing_catalogs_and_resource_choices() {
    let directory = tempfile::tempdir().unwrap();
    let missing_catalog = directory.path().join("missing.toml");

    let no_input = run_aliasmgr(&missing_catalog, &["add", "group", "tools", "--no-input"]);
    assert_input_required(&no_input, "use missing catalog path");
    assert!(!missing_catalog.exists());

    let force = run_aliasmgr(&missing_catalog, &["add", "group", "tools", "--force"]);
    assert!(force.status.success());
    assert!(
        fs::read_to_string(&missing_catalog)
            .unwrap()
            .contains("[tools]")
    );

    fs::write(
        &missing_catalog,
        "[tools]\n\"aliasmgr ungrouped alias\" = \"echo tools\"\n",
    )
    .unwrap();
    let remove_alias = run_aliasmgr(&missing_catalog, &["remove", "tools", "--force"]);
    assert!(remove_alias.status.success());
    assert_eq!(fs::read_to_string(&missing_catalog).unwrap(), "[tools]\n");
}

#[test]
fn force_accepts_reassignment_and_enabling_prompts() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(
        &catalog,
        "[tools]\nenabled = false\nbuild = \"cargo build\"\n",
    )
    .unwrap();

    let output = run_aliasmgr(&catalog, &["remove", "tools", "--force"]);

    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(&catalog).unwrap(),
        "build = \"cargo build\"\n"
    );
}

#[test]
fn force_and_no_input_conflict_across_command_scopes() {
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
    assert!(
        String::from_utf8_lossy(&shell_sync.stderr)
            .contains("--force cannot be used with --if-changed")
    );
}
