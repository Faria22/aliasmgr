use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run_aliasmgr(catalog: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aliasmgr"))
        .args(args)
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_SHELL", "zsh")
        .output()
        .unwrap()
}

#[test]
fn imports_bash_and_zsh_aliases_from_multiple_files() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let bash = directory.path().join("bashrc");
    let zsh = directory.path().join("zshrc");
    fs::write(&catalog, "existing = \"echo existing\"\n").unwrap();
    fs::write(
        &bash,
        "# shell aliases\nalias ll='ls -la'\nfunction ignored() { echo no; }\n",
    )
    .unwrap();
    fs::write(&zsh, "alias -g G='| grep'\nalias gs=git\\ status\n").unwrap();

    let output = run_aliasmgr(
        &catalog,
        &[
            "import",
            bash.to_str().unwrap(),
            zsh.to_str().unwrap(),
            "-t",
            "shell",
        ],
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Imported 3 aliases; 1 unsupported lines skipped."
    );
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("existing = \"echo existing\""));
    assert!(content.contains("ll = { command = \"ls -la\""));
    assert!(content.contains("G = { command = \"| grep\", enabled = true, global = true"));
    assert!(content.contains("gs = { command = \"git status\""));
    assert_eq!(content.matches("tags = [\"shell\"]").count(), 3);
}

#[test]
fn collision_policies_preserve_identical_aliases_and_replace_differences() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let source = directory.path().join("aliases.zsh");
    let original = concat!(
        "ll = { command = \"ls -la\", enabled = false, global = false, description = \"Files\", tags = [\"existing\"] }\n",
        "gs = \"git status\"\n",
    );
    fs::write(&catalog, original).unwrap();
    fs::write(
        &source,
        "alias ll='ls -la'\nalias gs='git status --short'\n",
    )
    .unwrap();

    let skipped = run_aliasmgr(&catalog, &["import", source.to_str().unwrap(), "-s"]);
    assert!(skipped.status.success(), "{skipped:?}");
    assert!(
        String::from_utf8_lossy(&skipped.stdout)
            .contains("1 collisions found; 0 replaced and 1 skipped")
    );
    assert!(String::from_utf8_lossy(&skipped.stdout).contains("1 aliases unchanged"));
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);

    let replaced = run_aliasmgr(&catalog, &["import", source.to_str().unwrap(), "-r", "-N"]);
    assert!(replaced.status.success(), "{replaced:?}");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("gs = \"git status --short\""));
    assert!(content.contains("description = \"Files\""));
    assert!(content.contains("tags = [\"existing\"]"));
    assert!(content.contains("enabled = false"));
}

#[test]
fn prompt_modes_alias_collision_policies_and_no_input_requires_a_policy() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let source = directory.path().join("aliases");
    fs::write(&catalog, "ll = \"ls\"\n").unwrap();
    fs::write(&source, "alias ll='ls -la'\nalias gs='git status'\n").unwrap();

    let refused = run_aliasmgr(
        &catalog,
        &["import", source.to_str().unwrap(), "--no-input"],
    );
    assert_eq!(refused.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&catalog).unwrap(), "ll = \"ls\"\n");

    let skipped = run_aliasmgr(&catalog, &["import", source.to_str().unwrap(), "--no"]);
    assert!(skipped.status.success(), "{skipped:?}");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("ll = \"ls\""));
    assert!(content.contains("gs = \"git status\""));

    fs::write(&catalog, "ll = \"ls\"\n").unwrap();
    let replaced = run_aliasmgr(&catalog, &["import", source.to_str().unwrap(), "--yes"]);
    assert!(replaced.status.success(), "{replaced:?}");
    let content = fs::read_to_string(&catalog).unwrap();
    assert!(content.contains("ll = \"ls -la\""));
    assert!(content.contains("gs = \"git status\""));
}

#[test]
fn dry_run_reports_without_prompting_or_writing() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let source = directory.path().join("aliases");
    let original = "ll = \"ls\"\n";
    fs::write(&catalog, original).unwrap();
    fs::write(&source, "alias ll='ls -la'\nalias gs='git status'\n").unwrap();

    let output = run_aliasmgr(&catalog, &["import", source.to_str().unwrap(), "-d"]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run: 1 aliases would be imported"));
    assert!(stdout.contains("1 collisions found"));
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);

    for (mode, result) in [("--yes", "replaced"), ("--no", "skipped")] {
        let output = run_aliasmgr(&catalog, &["import", source.to_str().unwrap(), "-d", mode]);
        assert!(output.status.success(), "{mode}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains(&format!("1 collisions found and would be {result}"))
        );
    }

    let skipped = run_aliasmgr(
        &catalog,
        &[
            "import",
            source.to_str().unwrap(),
            "--dry-run",
            "--skip-existing",
        ],
    );
    assert!(
        String::from_utf8_lossy(&skipped.stdout)
            .contains("1 collisions found and would be skipped")
    );

    let replaced = run_aliasmgr(
        &catalog,
        &[
            "import",
            source.to_str().unwrap(),
            "--dry-run",
            "--replace-existing",
        ],
    );
    assert!(
        String::from_utf8_lossy(&replaced.stdout)
            .contains("1 collisions found and would be replaced")
    );
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);
}

#[test]
fn unreadable_sources_warn_and_do_not_prevent_other_imports() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let missing = directory.path().join("missing");
    let source = directory.path().join("aliases");
    fs::write(&catalog, "").unwrap();
    fs::write(&source, "alias ll='ls -la'\n").unwrap();

    let output = run_aliasmgr(
        &catalog,
        &[
            "import",
            missing.to_str().unwrap(),
            source.to_str().unwrap(),
        ],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("skipping file"));
    assert!(
        fs::read_to_string(&catalog)
            .unwrap()
            .contains("ll = \"ls -la\"")
    );
}

#[test]
fn all_unreadable_sources_still_succeed_without_changes() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = "ll = \"ls -la\"\n";
    fs::write(&catalog, original).unwrap();

    let output = run_aliasmgr(
        &catalog,
        &[
            "import",
            directory.path().join("missing-one").to_str().unwrap(),
            directory.path().join("missing-two").to_str().unwrap(),
        ],
    );

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Imported 0 aliases."
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr)
            .matches("skipping file")
            .count(),
        2
    );
    assert_eq!(fs::read_to_string(&catalog).unwrap(), original);
}
