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

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[test]
fn filters_by_pattern_and_every_repeated_tag() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(
        &catalog,
        concat!(
            "bench = { command = \"cargo bench\", tags = [\"dev\", \"rust\"] }\n",
            "build = { command = \"cargo build\", tags = [\"dev\", \"rust\"] }\n",
            "bundle = { command = \"bundle\", tags = [\"dev\", \"ruby\"] }\n",
        ),
    )
    .unwrap();

    let disable = run_aliasmgr(
        &catalog,
        &[
            "disable",
            "alias",
            "--pattern",
            "b*",
            "--tag",
            "dev",
            "--tag",
            "rust",
        ],
    );
    assert!(disable.status.success(), "{disable:?}");
    assert_eq!(stdout(&disable), "Disabled 2 of 2 matching aliases.");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("bench = { command = \"cargo bench\", enabled = false"));
    assert!(content.contains("build = { command = \"cargo build\", enabled = false"));
    assert!(content.contains("bundle = { command = \"bundle\", enabled = true"));
}

#[test]
fn tag_removal_detaches_without_removing_aliases() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(
        &catalog,
        "build = { command = \"cargo build\", tags = [\"dev\", \"rust\"] }\n",
    )
    .unwrap();
    let output = run_aliasmgr(&catalog, &["remove", "tag", "dev"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "Removed tag 'dev' from 1 aliases.");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("build ="));
    assert!(content.contains("tags = [\"rust\"]"));
}

#[test]
fn tag_removal_can_delete_tagged_aliases_after_one_confirmation() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = concat!(
        "build = { command = \"cargo build\", tags = [\"dev\", \"rust\"] }\n",
        "test = { command = \"cargo test\", tags = [\"dev\"] }\n",
        "release = { command = \"cargo release\", tags = [\"ops\"] }\n",
    );
    fs::write(&catalog, original).unwrap();

    let no_input = run_aliasmgr(
        &catalog,
        &["remove", "tag", "dev", "--aliases", "--no-input"],
    );
    assert_eq!(no_input.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&no_input.stderr).contains("remove 2 aliases tagged 'dev'"));
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);

    let force = run_aliasmgr(&catalog, &["remove", "tag", "dev", "--aliases", "--force"]);
    assert!(force.status.success(), "{force:?}");
    assert_eq!(stdout(&force), "Removed 2 of 2 aliases tagged 'dev'.");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(!content.contains("build ="));
    assert!(!content.contains("test ="));
    assert!(content.contains("release ="));
}

#[test]
fn filtered_remove_prompts_once_and_empty_matches_are_noops() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = "build = { command = \"cargo build\", tags = [\"dev\"] }\nbench = { command = \"cargo bench\", tags = [\"dev\"] }\n";
    fs::write(&catalog, original).unwrap();

    let no_input = run_aliasmgr(&catalog, &["remove", "alias", "--tag", "dev", "--no-input"]);
    assert_eq!(no_input.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&no_input.stderr)
            .contains("remove 2 aliases matching the selector")
    );
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);

    let force = run_aliasmgr(&catalog, &["remove", "alias", "--tag", "dev", "--force"]);
    assert!(force.status.success());
    assert_eq!(stdout(&force), "Removed 2 of 2 matching aliases.");

    fs::write(&catalog, "ll = \"ls -la\"\n").unwrap();
    let empty = run_aliasmgr(&catalog, &["enable", "alias", "--tag", "missing"]);
    assert!(empty.status.success());
    assert_eq!(stdout(&empty), "No aliases matched the selector.");
}
