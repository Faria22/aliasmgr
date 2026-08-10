use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_aliasmgr(catalog: &Path, args: &[&str]) -> Output {
    run_aliasmgr_with_shell(catalog, "bash", args)
}

fn run_aliasmgr_with_shell(catalog: &Path, shell: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(args)
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_SHELL", shell)
        .output()
        .unwrap()
}

#[test]
fn metadata_commands_cover_exact_shorthand_tag_filter_and_all_forms() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(
        &catalog,
        concat!(
            "one = { command = \"echo one\", tags = [\"dev\"] }\n",
            "two = { command = \"echo two\", enabled = false, tags = [\"dev\"] }\n",
            "three = \"echo three\"\n",
        ),
    )
    .unwrap();

    for args in [
        &["disable", "alias", "one"][..],
        &["enable", "one"][..],
        &["enable", "tag", "dev"][..],
        &["disable", "tag", "dev"][..],
        &["enable", "alias", "--pattern", "t*"][..],
        &["disable", "alias", "--tag", "missing"][..],
        &["enable", "all"][..],
        &["disable", "all"][..],
        &["disable", "three"][..],
        &["enable", "alias", "three"][..],
    ] {
        let output = run_aliasmgr(&catalog, args);
        assert!(output.status.success(), "{args:?}: {output:?}");
    }

    let rename_explicit = run_aliasmgr(&catalog, &["rename", "alias", "one", "first"]);
    assert!(rename_explicit.status.success(), "{rename_explicit:?}");
    let rename_shorthand = run_aliasmgr(&catalog, &["rename", "first", "one"]);
    assert!(rename_shorthand.status.success(), "{rename_shorthand:?}");

    let edit = run_aliasmgr(&catalog, &["edit", "one", "--toggle-enabled"]);
    assert!(edit.status.success(), "{edit:?}");
    let global = run_aliasmgr_with_shell(&catalog, "zsh", &["edit", "one", "--toggle-global"]);
    assert!(global.status.success(), "{global:?}");
    let global_off = run_aliasmgr_with_shell(&catalog, "bash", &["edit", "one", "--toggle-global"]);
    assert!(global_off.status.success(), "{global_off:?}");

    let remove_tag = run_aliasmgr(&catalog, &["remove", "tag", "dev"]);
    assert!(remove_tag.status.success(), "{remove_tag:?}");
    let remove_exact = run_aliasmgr(&catalog, &["remove", "alias", "two"]);
    assert!(remove_exact.status.success(), "{remove_exact:?}");
    let remove_shorthand = run_aliasmgr(&catalog, &["remove", "three"]);
    assert!(remove_shorthand.status.success(), "{remove_shorthand:?}");
}

#[test]
fn metadata_command_failures_are_reported() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(
        &catalog,
        "one = { command = \"echo one\", tags = [\"dev\"] }\ntwo = \"echo two\"\n",
    )
    .unwrap();

    for args in [
        &["add", "global", "echo global", "--global"][..],
        &["add", "bad=name", "echo bad"][..],
        &["enable", "alias", "missing"][..],
        &["disable", "missing"][..],
        &["edit", "missing", "echo changed"][..],
        &["edit", "two", "--toggle-global"][..],
        &["remove", "alias", "missing"][..],
        &["remove", "tag", "missing"][..],
        &["rename", "alias", "missing", "new"][..],
        &["rename", "alias", "one", "two"][..],
        &["rename", "tag", "missing", "new"][..],
        &["rename", "tag", "missing", "missing"][..],
        &["edit", "one"][..],
        &["add", "bad", "echo bad", "--tag", "two words"][..],
    ] {
        let output = run_aliasmgr(&catalog, args);
        assert!(!output.status.success(), "{args:?}: {output:?}");
    }

    let unchanged = run_aliasmgr(&catalog, &["rename", "tag", "dev", "dev"]);
    assert!(unchanged.status.success(), "{unchanged:?}");
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
