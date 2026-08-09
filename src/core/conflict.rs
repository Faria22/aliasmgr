use crate::app::shell::ShellType;
use indexmap::IndexMap;
use log::debug;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const BASH_BUILTIN_QUERY: &str = r#"
for name do
    if [ "$(builtin type -t -- "$name")" = "builtin" ]; then
        printf '%s\0' "$name"
    fi
done
"#;

const ZSH_BUILTIN_QUERY: &str = r#"
for name do
    if [ "$(builtin whence -w -- "$name")" = "$name: builtin" ]; then
        printf '%s\0' "$name"
    fi
done
"#;

fn shell_builtin_names(names: &[String], shell: &ShellType) -> HashSet<String> {
    let (program, options, query) = match shell {
        ShellType::Bash => (
            "bash",
            ["--noprofile", "--norc", "-c"].as_slice(),
            BASH_BUILTIN_QUERY,
        ),
        ShellType::Zsh => ("zsh", ["-dfc"].as_slice(), ZSH_BUILTIN_QUERY),
    };

    let output = Command::new(program)
        .args(options)
        .arg(query)
        .arg("aliasmgr")
        .args(names)
        .env_remove("BASH_ENV")
        .output();

    let Ok(output) = output else {
        debug!("Could not query {shell} builtins; continuing with PATH checks.");
        return HashSet::new();
    };

    if !output.status.success() {
        debug!("{shell} builtin query failed; continuing with PATH checks.");
        return HashSet::new();
    }

    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .filter_map(|name| String::from_utf8(name.to_vec()).ok())
        .collect()
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn executable_on_path(name: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    if name.contains('/') {
        return None;
    }

    path.and_then(|path| {
        env::split_paths(path)
            .map(|directory| directory.join(name))
            .find(|candidate| is_executable(candidate))
    })
}

fn conflict_warnings_with(
    names: &[String],
    shell: &ShellType,
    builtin_names: &HashSet<String>,
    path: Option<&OsStr>,
) -> IndexMap<String, Vec<String>> {
    let mut warnings = IndexMap::new();

    for name in names {
        let mut alias_warnings = Vec::new();

        if builtin_names.contains(name) {
            alias_warnings.push(format!(
                "Alias '{name}' conflicts with a {shell} shell builtin."
            ));
        }

        if let Some(executable) = executable_on_path(name, path) {
            alias_warnings.push(format!(
                "Alias '{name}' shadows executable '{}' found on PATH.",
                executable.display()
            ));
        }

        if !alias_warnings.is_empty() {
            warnings.insert(name.clone(), alias_warnings);
        }
    }

    warnings
}

pub fn conflict_warnings<'a>(
    names: impl IntoIterator<Item = &'a str>,
    shell: &ShellType,
) -> IndexMap<String, Vec<String>> {
    let names = names.into_iter().map(str::to_owned).collect::<Vec<_>>();
    let builtin_names = shell_builtin_names(&names, shell);
    let path = env::var_os("PATH");
    conflict_warnings_with(&names, shell, &builtin_names, path.as_deref())
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::fs;

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        fs::write(path, "executable").unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn reports_builtin_and_path_conflicts_separately() {
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("echo");
        make_executable(&executable);
        let path = env::join_paths([directory.path()]).unwrap();
        let names = vec!["echo".to_string()];
        let builtin_names = HashSet::from(["echo".to_string()]);

        let warnings = conflict_warnings_with(
            &names,
            &ShellType::Bash,
            &builtin_names,
            Some(path.as_os_str()),
        );

        assert_eq!(warnings["echo"].len(), 2);
        assert!(warnings["echo"][0].contains("BASH shell builtin"));
        assert!(warnings["echo"][1].contains(executable.to_string_lossy().as_ref()));
    }

    #[cfg(unix)]
    #[test]
    fn ignores_non_executable_files_on_path() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("notes"), "not executable").unwrap();
        let path = env::join_paths([directory.path()]).unwrap();
        let names = vec!["notes".to_string()];

        let warnings = conflict_warnings_with(
            &names,
            &ShellType::Zsh,
            &HashSet::new(),
            Some(path.as_os_str()),
        );

        assert!(warnings.is_empty());
    }

    #[test]
    fn non_conflicting_alias_has_no_warning() {
        let names = vec!["aliasmgr_definitely_unique".to_string()];
        let warnings = conflict_warnings_with(&names, &ShellType::Bash, &HashSet::new(), None);

        assert!(warnings.is_empty());
    }

    #[test]
    fn shell_queries_are_batched_and_pass_names_as_arguments() {
        assert!(BASH_BUILTIN_QUERY.contains("builtin type -t -- \"$name\""));
        assert!(ZSH_BUILTIN_QUERY.contains("builtin whence -w -- \"$name\""));
        assert!(BASH_BUILTIN_QUERY.contains("for name do"));
        assert!(ZSH_BUILTIN_QUERY.contains("for name do"));
    }
}
