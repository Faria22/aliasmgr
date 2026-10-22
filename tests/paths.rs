use std::fs;
use std::process::{Command, Output};

fn isolated_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aliasmgr"));
    command
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("ALIASMGR_CONFIG_PATH")
        .env_remove("ALIASMGR_CATALOG_PATH")
        .env_remove("RUST_LOG")
        .env("ALIASMGR_SHELL", "bash")
        .env("RUST_BACKTRACE", "1");
    command
}

fn assert_clean_path_error(output: &Output, explicit_variable: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("ERROR:"), "{stderr}");
    for expected in ["HOME", "XDG_CONFIG_HOME", explicit_variable] {
        assert!(stderr.contains(expected), "{stderr}");
    }
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(!stderr.contains("backtrace"), "{stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn missing_base_directories_report_configuration_error() {
    for args in [&["list"][..], &["shell-sync", "--if-changed"][..]] {
        let output = isolated_command().args(args).output().unwrap();
        assert_clean_path_error(&output, "ALIASMGR_CONFIG_PATH");
    }
}

#[test]
fn explicit_configuration_does_not_hide_missing_catalog_base_directory() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    fs::write(&config, "").unwrap();
    let output = isolated_command()
        .env("ALIASMGR_CONFIG_PATH", &config)
        .arg("list")
        .output()
        .unwrap();
    assert_clean_path_error(&output, "ALIASMGR_CATALOG_PATH");
}

#[test]
fn explicit_paths_and_xdg_config_home_work_without_home() {
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.toml");
    let catalog = directory.path().join("aliases.toml");
    fs::write(&config, "").unwrap();
    for mut command in [
        {
            let mut command = isolated_command();
            command
                .env("ALIASMGR_CONFIG_PATH", &config)
                .env("ALIASMGR_CATALOG_PATH", &catalog);
            command
        },
        {
            let mut command = isolated_command();
            command.env("XDG_CONFIG_HOME", directory.path());
            command
        },
    ] {
        let output = command.args(["list", "--format", "json"]).output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "[]");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
    assert!(!catalog.exists());
    assert!(!directory.path().join("aliasmgr").exists());
}

#[cfg(unix)]
#[test]
fn non_utf8_catalog_path_is_preserved_in_cli() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let directory = tempfile::tempdir().unwrap();
    let catalog = directory
        .path()
        .join(OsString::from_vec(b"aliases-\xff.toml".to_vec()));
    let config = directory.path().join("config.toml");
    fs::write(&config, "").unwrap();

    for mode in ["--no-input", "--yes"] {
        let output = isolated_command()
            .env("ALIASMGR_CONFIG_PATH", &config)
            .env("ALIASMGR_CATALOG_PATH", &catalog)
            .args(["add", "ll", "ls", mode])
            .output()
            .unwrap();
        if mode == "--no-input" {
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("aliases-\u{fffd}.toml"), "{stderr}");
            assert!(!stderr.contains("panicked"), "{stderr}");
            assert!(!catalog.exists());
        } else {
            // macOS filesystems reject invalid UTF-8 filenames. The CLI must
            // still reach the filesystem and report that failure cleanly.
            #[cfg(target_os = "macos")]
            {
                assert_eq!(output.status.code(), Some(1), "{output:?}");
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    stderr.starts_with("ERROR: could not open catalog lock"),
                    "{stderr}"
                );
                assert!(stderr.contains("Illegal byte sequence"), "{stderr}");
                assert!(!stderr.contains("panicked"), "{stderr}");
                assert!(!catalog.exists());
            }
            #[cfg(not(target_os = "macos"))]
            {
                assert!(output.status.success(), "{output:?}");
                assert_eq!(fs::read_to_string(&catalog).unwrap(), "ll = \"ls\"\n");
            }
            assert!(!directory.path().join("aliases-\u{fffd}.toml").exists());
        }
    }
}
