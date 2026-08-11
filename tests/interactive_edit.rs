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
fn interactive_edit_requires_a_terminal_without_rewriting_the_catalog() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let original = "ll = \"ls -la\"\n";
    fs::write(&catalog, original).unwrap();

    for args in [
        &["edit", "ll", "--interactive"][..],
        &["edit", "--interactive", "--all"][..],
    ] {
        let output = run_aliasmgr(&catalog, args);
        assert_eq!(output.status.code(), Some(1), "{args:?}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("interactive editing requires a terminal")
        );
        assert_eq!(fs::read_to_string(&catalog).unwrap(), original);
    }
}

#[test]
fn interactive_edit_conflicts_with_every_global_prompt_control() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    fs::write(&catalog, "ll = \"ls -la\"\n").unwrap();

    for control in ["--yes", "--no", "--no-input"] {
        let output = run_aliasmgr(&catalog, &["edit", "ll", "--interactive", control]);
        assert_eq!(output.status.code(), Some(2), "{control}: {output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("cannot be used with --interactive")
        );
    }
}
