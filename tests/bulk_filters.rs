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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn enable_and_disable_filter_by_pattern_group_and_ungrouped() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(
        &catalog,
        concat!(
            "local = \"echo local\"\n",
            "zglob = { command = \"*.rs\", global = true }\n",
            "[dev]\n",
            "build = \"cargo build\"\n",
            "bench = { command = \"cargo bench\", enabled = false }\n",
            "[ops]\n",
            "deploy = \"deploy\"\n",
        ),
    )
    .unwrap();

    let disable = run_aliasmgr(
        &catalog,
        &["disable", "alias", "--pattern", "b*", "--group", "dev"],
    );
    assert!(disable.status.success(), "command failed: {disable:?}");
    assert_eq!(stdout(&disable), "Disabled 1 of 2 matching aliases.");

    let enable = run_aliasmgr(&catalog, &["enable", "alias", "--group", "dev"]);
    assert!(enable.status.success(), "command failed: {enable:?}");
    assert_eq!(stdout(&enable), "Enabled 2 of 2 matching aliases.");

    let disable_ungrouped = run_aliasmgr(&catalog, &["disable", "alias", "--group"]);
    assert!(
        disable_ungrouped.status.success(),
        "command failed: {disable_ungrouped:?}"
    );
    assert_eq!(
        stdout(&disable_ungrouped),
        "Disabled 2 of 2 matching aliases."
    );

    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("local = { command = \"echo local\", enabled = false"));
    assert!(content.contains("zglob = { command = \"*.rs\", enabled = false"));
    assert!(content.contains("build = { command = \"cargo build\", enabled = true"));
    assert!(content.contains("bench = { command = \"cargo bench\", enabled = true"));
    assert!(content.contains("deploy = \"deploy\""));
    assert!(content.contains("[dev]"));
    assert!(content.contains("[ops]"));
}

#[test]
fn remove_by_group_prompts_once_and_preserves_the_group() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original =
        "[dev]\nbuild = \"cargo build\"\nbench = \"cargo bench\"\n[ops]\ndeploy = \"deploy\"\n";
    fs::write(&catalog, original).unwrap();

    let no_input = run_aliasmgr(
        &catalog,
        &["remove", "alias", "--group", "dev", "--no-input"],
    );
    assert_eq!(no_input.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&no_input.stderr)
            .contains("remove 2 aliases matching the selector")
    );
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);

    let force = run_aliasmgr(&catalog, &["remove", "alias", "--group", "dev", "--force"]);
    assert!(force.status.success(), "command failed: {force:?}");
    assert_eq!(stdout(&force), "Removed 2 of 2 matching aliases.");
    assert_eq!(
        fs::read_to_string(&catalog).unwrap(),
        "[dev]\n\n[ops]\ndeploy = \"deploy\"\n"
    );
}

#[test]
fn empty_match_is_a_no_op_and_invalid_selectors_do_not_modify_the_catalog() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = "ll = \"ls -la\"\n";
    fs::write(&catalog, original).unwrap();

    for command in ["enable", "disable"] {
        let empty = run_aliasmgr(&catalog, &[command, "alias", "--pattern", "missing-*"]);
        assert!(empty.status.success(), "command failed: {empty:?}");
        assert_eq!(stdout(&empty), "No aliases matched the selector.");
    }

    let empty = run_aliasmgr(
        &catalog,
        &["remove", "alias", "--pattern", "missing-*", "--no-input"],
    );
    assert!(empty.status.success(), "command failed: {empty:?}");
    assert_eq!(stdout(&empty), "No aliases matched the selector.");

    let invalid = run_aliasmgr(&catalog, &["disable", "alias", "--pattern", "["]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("invalid glob pattern"));
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);
}
