#![cfg(unix)]

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

fn make_executable(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn run_aliasmgr(catalog: &Path, shell: &str, path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(args)
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_SHELL", shell)
        .env("PATH", path)
        .output()
        .expect("aliasmgr command should run")
}

#[test]
fn add_warns_for_executable_on_path() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let executable = directory.path().join("shadowed-tool");
    fs::write(&catalog, "").unwrap();
    make_executable(&executable, "executable");

    let output = run_aliasmgr(
        &catalog,
        "bash",
        directory.path(),
        &["add", "shadowed-tool", "echo shadowed"],
    );

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(&format!(
        "Alias 'shadowed-tool' shadows executable '{}' found on PATH.",
        executable.display()
    )));
}

#[test]
fn add_warns_for_bash_builtin() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "").unwrap();
    let path = env::var_os("PATH").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(["add", "cd", "echo cd"])
        .env("ALIASMGR_CATALOG_PATH", &catalog)
        .env("ALIASMGR_SHELL", "bash")
        .env("PATH", path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Alias 'cd' conflicts with a BASH shell builtin.")
    );
}

#[test]
fn add_uses_zsh_builtin_query_without_a_static_list() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let fake_zsh = directory.path().join("zsh");
    fs::write(&catalog, "").unwrap();
    make_executable(&fake_zsh, "#!/bin/sh\nprintf 'cd\\0'\n");

    let output = run_aliasmgr(&catalog, "zsh", directory.path(), &["add", "cd", "echo cd"]);

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Alias 'cd' conflicts with a ZSH shell builtin.")
    );
}

#[test]
fn edit_and_doctor_warn_for_existing_conflict() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "cd = \"echo old\"\n").unwrap();
    let path = env::var_os("PATH").unwrap();

    let edit = Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(["edit", "cd", "echo new"])
        .env("ALIASMGR_CATALOG_PATH", &catalog)
        .env("ALIASMGR_SHELL", "bash")
        .env("PATH", &path)
        .output()
        .unwrap();
    let doctor = Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .arg("doctor")
        .env("ALIASMGR_CATALOG_PATH", &catalog)
        .env("ALIASMGR_SHELL", "bash")
        .env("PATH", path)
        .output()
        .unwrap();

    assert!(edit.status.success());
    assert!(
        String::from_utf8_lossy(&edit.stderr)
            .contains("Alias 'cd' conflicts with a BASH shell builtin.")
    );
    assert!(doctor.status.success());
    assert!(
        String::from_utf8_lossy(&doctor.stderr)
            .contains("WARNING: Alias 'cd' conflicts with a BASH shell builtin.")
    );
    assert!(String::from_utf8_lossy(&doctor.stdout).contains("0 errors and"));
}

#[test]
fn non_conflicting_add_is_quiet() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "").unwrap();

    let output = run_aliasmgr(
        &catalog,
        "bash",
        directory.path(),
        &["add", "aliasmgr_definitely_unique", "echo unique"],
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
}

#[test]
fn quiet_suppresses_conflict_warnings() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "cd = \"echo cd\"\n").unwrap();
    let path = env::var_os("PATH").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(["--quiet", "doctor"])
        .env("ALIASMGR_CATALOG_PATH", &catalog)
        .env("ALIASMGR_SHELL", "bash")
        .env("PATH", path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
