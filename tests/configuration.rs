use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_aliasmgr(catalog: &Path, config: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_CONFIG_PATH", config)
        .env("ALIASMGR_SHELL", "zsh")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn explicit_configuration_controls_symbols_and_styles() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let config = directory.path().join("config.toml");
    fs::write(&catalog, "ll = \"ls -la\"\n").unwrap();
    fs::write(
        &config,
        r##"
        [color]
        mode = "never"
        [symbols]
        enabled = "+"
        [list]
        columns = ["status", "name"]
        status = "always"
        [styles.enabled]
        foreground = "#ff00aa"
        bold = false
        "##,
    )
    .unwrap();

    let plain = run_aliasmgr(&catalog, &config, &["list"]);
    assert!(plain.status.success());
    assert_eq!(
        String::from_utf8(plain.stdout).unwrap(),
        "Status  Name\n+       ll\n"
    );

    let colored = run_aliasmgr(&catalog, &config, &["--color", "always", "list"]);
    assert!(colored.status.success());
    assert_eq!(
        String::from_utf8(colored.stdout).unwrap(),
        "Status  Name\n\u{1b}[38;2;255;0;170m+\u{1b}[0m       ll\n"
    );

    let override_columns = run_aliasmgr(&catalog, &config, &["list", "--columns", "name,command"]);
    assert_eq!(
        String::from_utf8(override_columns.stdout).unwrap(),
        "Name  Command\nll    ls -la\n"
    );
}

#[test]
fn missing_explicit_configuration_fails_clearly() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let config = directory.path().join("missing.toml");
    fs::write(&catalog, "").unwrap();

    let output = run_aliasmgr(&catalog, &config, &["list"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("configured file"));
}

#[test]
fn unknown_configuration_settings_warn_without_blocking() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let config = directory.path().join("config.toml");
    fs::write(&catalog, "ll = \"ls -la\"\n").unwrap();
    fs::write(&config, "future = true\n").unwrap();

    let output = run_aliasmgr(&catalog, &config, &["list"]);
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Unknown configuration setting 'future' ignored.")
    );
}
