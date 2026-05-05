use super::shell::ShellType;
use crate::catalog::types::AliasCatalog;
use crate::cli::sync::SyncCommand;
use crate::core::add::add_all_active_aliases;
use crate::core::sync::generate_alias_script_content;
use crate::core::{Failure, Outcome};

use log::info;
use std::path::PathBuf;

/// Handles alias synchronization.
pub fn handle_sync(
    catalog: &mut AliasCatalog,
    shell: &ShellType,
    cmd: SyncCommand,
    last_synced_catalog_path: &PathBuf,
) -> Result<Outcome, Failure> {
    if cmd.startup {
        info!("Startup sync: Adding aliases");
        Ok(Outcome::Command(add_all_active_aliases(catalog, shell)))
    } else {
        info!("Syncing aliases");
        Ok(Outcome::Command(generate_alias_script_content(
            catalog,
            shell,
            last_synced_catalog_path,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;
    use assert_fs::TempDir;

    fn sample_catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "current_alias".to_string(),
            Alias::new("echo current".to_string(), None, true, false),
        );
        catalog
    }

    fn last_synced_catalog_with_old_alias() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("last_synced_catalog.toml");
        std::fs::write(&path, "old_alias = \"echo old\"\n").unwrap();
        (temp_dir, path)
    }

    #[test]
    fn startup_sync_only_adds_current_aliases() {
        let mut catalog = sample_catalog();
        let (_temp_dir, last_synced_path) = last_synced_catalog_with_old_alias();
        let cmd = SyncCommand { startup: true };

        let result = handle_sync(&mut catalog, &ShellType::Bash, cmd, &last_synced_path);

        assert_eq!(
            result,
            Ok(Outcome::Command(
                "alias -- 'current_alias'='echo current'\n".to_string()
            ))
        );
    }

    #[test]
    fn startup_sync_does_not_remove_last_synced_aliases() {
        let mut catalog = sample_catalog();
        let (_temp_dir, last_synced_path) = last_synced_catalog_with_old_alias();
        let cmd = SyncCommand { startup: true };

        let result = handle_sync(&mut catalog, &ShellType::Bash, cmd, &last_synced_path);

        let Outcome::Command(commands) = result.unwrap() else {
            panic!("expected shell commands");
        };
        assert!(!commands.contains("unalias"));
        assert!(!commands.contains("old_alias"));
    }

    #[test]
    fn regular_sync_removes_last_synced_aliases() {
        let mut catalog = sample_catalog();
        let (_temp_dir, last_synced_path) = last_synced_catalog_with_old_alias();
        let cmd = SyncCommand { startup: false };

        let result = handle_sync(&mut catalog, &ShellType::Bash, cmd, &last_synced_path);

        let Outcome::Command(commands) = result.unwrap() else {
            panic!("expected shell commands");
        };
        assert!(commands.contains("unalias 'old_alias'"));
        assert!(commands.contains("alias -- 'current_alias'='echo current'"));
    }
}
