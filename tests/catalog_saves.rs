use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn command(catalog: &Path, config: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aliasmgr"));
    command
        .env("ALIASMGR_CATALOG_PATH", catalog)
        .env("ALIASMGR_CONFIG_PATH", config)
        .env("ALIASMGR_SHELL", "bash")
        .env_remove("RUST_LOG");
    command
}

// Reap children even if a lock or assertion fails, rather than leaving a test
// process waiting forever on a lock held by this process.
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn wait_for_lock_attempt(child: &mut Process) {
    let stderr = child.0.stderr.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let line = line.unwrap();
            if line.contains("Waiting for catalog lock") {
                let _ = sender.send(());
            }
        }
    });
    receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("command did not attempt catalog lock");
    assert!(child.0.try_wait().unwrap().is_none());
}

fn wait_for_success(child: &mut Process) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "command exited with {status}");
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "command stayed blocked"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn queued_mutations_load_after_lock_and_preserve_every_change() {
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let config = directory.path().join("config.toml");
    fs::write(&config, "").unwrap();
    // Start with no catalog to cover creation as well as later replacements.
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(directory.path().join("aliases.toml.lock"))
        .unwrap();
    lock.lock().unwrap();
    let mut children = Vec::new();
    for index in 0..8 {
        let mut child = Process(
            command(&catalog, &config)
                .args([
                    "--debug",
                    "--yes",
                    "add",
                    &format!("entry{index}"),
                    &format!("echo {index}"),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        wait_for_lock_attempt(&mut child);
        children.push(child);
    }
    // A change made while all commands are queued must also survive: none may
    // load a stale snapshot before obtaining the transaction lock.
    fs::write(&catalog, "seed = \"echo seed\"\n").unwrap();
    drop(lock);
    for child in &mut children {
        wait_for_success(child);
    }
    let saved: toml::Table = toml::from_str(&fs::read_to_string(&catalog).unwrap()).unwrap();
    assert_eq!(saved.len(), 9);
    assert_eq!(saved["seed"].as_str(), Some("echo seed"));
    for index in 0..8 {
        assert_eq!(
            saved[&format!("entry{index}")].as_str(),
            Some(format!("echo {index}").as_str())
        );
    }
}

#[cfg(unix)]
#[test]
fn direct_and_symlinked_paths_share_the_transaction_lock() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let link = directory.path().join("linked.toml");
    let config = directory.path().join("config.toml");
    fs::write(&config, "").unwrap();
    fs::write(&catalog, "").unwrap();
    symlink("aliases.toml", &link).unwrap();
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(directory.path().join("aliases.toml.lock"))
        .unwrap();
    lock.lock().unwrap();
    let mut children = Vec::new();
    for (path, name) in [(&catalog, "direct"), (&link, "linked")] {
        let mut child = Process(
            command(path, &config)
                .args(["--debug", "--yes", "add", name, "echo test"])
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        wait_for_lock_attempt(&mut child);
        children.push(child);
    }
    // Read-only commands must finish even while a mutation lock is held.
    for args in [
        &["list"][..],
        &["doctor"][..],
        &["shell-sync", "--force"][..],
    ] {
        let mut reader = Process(
            command(&catalog, &config)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        wait_for_success(&mut reader);
    }
    drop(lock);
    for child in &mut children {
        wait_for_success(child);
    }
    let saved: toml::Table = toml::from_str(&fs::read_to_string(&catalog).unwrap()).unwrap();
    assert_eq!(saved.len(), 2);
    assert!(saved.contains_key("direct") && saved.contains_key("linked"));
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!directory.path().join("linked.toml.lock").exists());
}

#[cfg(unix)]
#[test]
fn save_failure_reports_path_and_cause_and_exits_nonzero() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let catalog = directory.path().join("aliases.toml");
    let config = directory.path().join("config.toml");
    fs::write(&config, "").unwrap();
    fs::write(&catalog, "original = \"echo original\"\n").unwrap();
    fs::set_permissions(&catalog, fs::Permissions::from_mode(0o444)).unwrap();
    let output = command(&catalog, &config)
        .args(["--yes", "add", "new", "echo new"])
        .output()
        .unwrap();
    fs::set_permissions(&catalog, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(!output.status.success(), "{output:?}");
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("ERROR: could not save catalog"), "{error}");
    assert!(
        error.contains(catalog.canonicalize().unwrap().to_str().unwrap()),
        "{error}"
    );
    assert!(
        error.to_lowercase().contains("permission denied"),
        "{error}"
    );
    assert_eq!(
        fs::read_to_string(&catalog).unwrap(),
        "original = \"echo original\"\n"
    );
}
