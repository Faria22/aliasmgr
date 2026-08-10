use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_doctor(catalog: &Path, shell: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .arg("doctor")
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_SHELL", shell)
        .output()
        .expect("doctor command should run")
}

#[test]
fn valid_catalog_succeeds_without_modification() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = "ll = \"ls -la\"\n";
    fs::write(&catalog, original).unwrap();

    let output = run_doctor(&catalog, "bash");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("OK: Catalog is valid for BASH."));
    assert!(output.stderr.is_empty());
    assert_eq!(fs::read_to_string(catalog).unwrap(), original);
}

#[test]
fn invalid_alias_name_fails() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "\"invalid name\" = \"echo invalid\"\n").unwrap();

    let output = run_doctor(&catalog, "zsh");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ERROR: Alias 'invalid name'"));
}

#[test]
fn global_alias_on_bash_warns_but_succeeds() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "glob = { command = \"*.rs\", global = true }\n").unwrap();

    let output = run_doctor(&catalog, "bash");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("WARNING: Global alias 'glob'"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("0 errors and 1 warning"));
}

#[test]
fn malformed_catalog_fails_cleanly() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "alias = {").unwrap();

    let output = run_doctor(&catalog, "bash");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ERROR: Could not load catalog"));
}

#[test]
fn legacy_group_fails_cleanly() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "[tools]\n[tools.nested]\nll = \"ls -la\"\n").unwrap();

    let output = run_doctor(&catalog, "bash");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("legacy alias groups are unsupported")
    );
}
