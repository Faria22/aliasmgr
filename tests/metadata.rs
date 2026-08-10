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

#[test]
fn add_edit_and_json_listing_preserve_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "").unwrap();

    let add = run_aliasmgr(
        &catalog,
        &[
            "add",
            "test",
            "cargo test",
            "--description",
            "Run tests",
            "--tag",
            "rust",
            "--tag",
            "dev",
            "--tag",
            "rust",
        ],
    );
    assert!(add.status.success(), "{add:?}");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("description = \"Run tests\""));
    assert!(content.contains("tags = [\"dev\", \"rust\"]"));

    let json = run_aliasmgr(&catalog, &["list", "--format", "json"]);
    let value: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(value[0]["description"], "Run tests");
    assert_eq!(value[0]["tags"], serde_json::json!(["dev", "rust"]));

    let edit = run_aliasmgr(
        &catalog,
        &["edit", "test", "--clear-description", "--remove-tag", "dev"],
    );
    assert!(edit.status.success(), "{edit:?}");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(!content.contains("description"));
    assert!(content.contains("tags = [\"rust\"]"));
}

#[test]
fn rename_tag_merges_with_existing_tag_and_changes_every_alias() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(
        &catalog,
        concat!(
            "build = { command = \"cargo build\", tags = [\"development\", \"dev\"] }\n",
            "test = { command = \"cargo test\", tags = [\"development\"] }\n",
        ),
    )
    .unwrap();
    let output = run_aliasmgr(&catalog, &["rename", "tag", "development", "dev"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Renamed tag 'development' to 'dev' on 2 aliases."
    );
    let content = fs::read_to_string(&catalog).unwrap();
    assert_eq!(content.matches("tags = [\"dev\"]").count(), 2);
}

#[test]
fn legacy_catalogs_and_duplicate_column_overrides_fail() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "[dev]\nbuild = \"cargo build\"\n").unwrap();
    let legacy = run_aliasmgr(&catalog, &["list"]);
    assert!(!legacy.status.success());
    assert!(
        String::from_utf8_lossy(&legacy.stderr).contains("legacy alias groups are unsupported")
    );

    fs::write(&catalog, "ll = \"ls -la\"\n").unwrap();
    let columns = run_aliasmgr(&catalog, &["list", "--columns", "name,name"]);
    assert!(!columns.status.success());
    assert!(String::from_utf8_lossy(&columns.stderr).contains("must not contain duplicates"));
}
