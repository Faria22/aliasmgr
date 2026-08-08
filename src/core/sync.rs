use crate::app::add::is_valid_alias_name;
use crate::app::shell::ShellType;
use crate::catalog::types::{Alias, AliasCatalog};

pub const MANAGED_ALIASES_ENV_VAR: &str = "ALIASMGR_MANAGED_ALIASES";
pub const CATALOG_REVISION_ENV_VAR: &str = "ALIASMGR_CATALOG_REVISION";

struct ActiveAlias<'a> {
    name: &'a str,
    alias: &'a Alias,
}

fn active_aliases<'a>(catalog: &'a AliasCatalog, shell: &ShellType) -> Vec<ActiveAlias<'a>> {
    catalog
        .aliases
        .iter()
        .filter(|(name, alias)| {
            is_valid_alias_name(name)
                && alias.enabled
                && (!alias.global || *shell == ShellType::Zsh)
                && alias
                    .group
                    .as_ref()
                    .is_none_or(|group| catalog.groups.get(group) == Some(&true))
        })
        .map(|(name, alias)| ActiveAlias { name, alias })
        .collect()
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn hash_field(hash: &mut u64, value: &str) {
    hash_bytes(hash, &value.len().to_le_bytes());
    hash_bytes(hash, value.as_bytes());
}

fn catalog_revision(active: &[ActiveAlias<'_>]) -> String {
    let mut aliases: Vec<_> = active.iter().collect();
    aliases.sort_unstable_by_key(|entry| entry.name);

    let mut hash = 0xcbf29ce484222325;
    for entry in aliases {
        hash_field(&mut hash, entry.name);
        hash_field(&mut hash, &entry.alias.command);
        hash_bytes(&mut hash, &[u8::from(entry.alias.global)]);
    }
    format!("{hash:016x}")
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn alias_command(name: &str, alias: &Alias) -> String {
    format!(
        "alias{} -- {}",
        if alias.global { " -g" } else { "" },
        shell_quote(&format!("{name}={}", alias.command))
    )
}

pub fn generate_reconciliation_script(
    catalog: &AliasCatalog,
    shell: &ShellType,
    managed_aliases: &str,
    applied_revision: &str,
    if_changed: bool,
) -> String {
    let active = active_aliases(catalog, shell);
    let revision = catalog_revision(&active);
    if if_changed && applied_revision == revision {
        return String::new();
    }

    let mut lines = vec!["__aliasmgr_sync_status=0".to_string()];
    for name in managed_aliases.lines().filter(|name| !name.is_empty()) {
        lines.push(format!(
            "unalias -- {} 2>/dev/null || true",
            shell_quote(name)
        ));
    }
    for entry in &active {
        lines.push(format!(
            "{} || __aliasmgr_sync_status=$?",
            alias_command(entry.name, entry.alias)
        ));
    }

    let names = active
        .iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>()
        .join("\n");
    lines.extend([
        "if [ \"$__aliasmgr_sync_status\" -eq 0 ]; then".to_string(),
        format!("    __aliasmgr_managed_aliases={}", shell_quote(&names)),
        format!("    __aliasmgr_catalog_revision={}", shell_quote(&revision)),
        "    unset __aliasmgr_sync_status".to_string(),
        "else".to_string(),
        "    unset __aliasmgr_sync_status".to_string(),
        "    false".to_string(),
        "fi".to_string(),
    ]);
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alias(command: &str) -> Alias {
        Alias::new(command.into(), None, true, false)
    }

    #[test]
    fn unchanged_revision_emits_nothing() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert("ll".into(), alias("ls -la"));
        let revision = catalog_revision(&active_aliases(&catalog, &ShellType::Bash));

        assert!(
            generate_reconciliation_script(&catalog, &ShellType::Bash, "ll", &revision, true)
                .is_empty()
        );
    }

    #[test]
    fn changed_catalog_removes_tracked_and_adds_all_active_aliases() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert("ll".into(), alias("ls -la"));
        catalog.aliases.insert("py".into(), alias("python3"));

        let script = generate_reconciliation_script(
            &catalog,
            &ShellType::Bash,
            "ll\nold",
            "old-revision",
            true,
        );

        assert!(script.contains("unalias -- 'll' 2>/dev/null || true"));
        assert!(script.contains("unalias -- 'old' 2>/dev/null || true"));
        assert!(script.contains("alias -- 'll=ls -la'"));
        assert!(script.contains("alias -- 'py=python3'"));
        assert!(script.contains("__aliasmgr_managed_aliases='ll\npy'"));
    }

    #[test]
    fn force_reconciles_an_unchanged_revision() {
        let catalog = AliasCatalog::new();
        let revision = catalog_revision(&active_aliases(&catalog, &ShellType::Bash));

        assert!(
            !generate_reconciliation_script(&catalog, &ShellType::Bash, "", &revision, false)
                .is_empty()
        );
    }

    #[test]
    fn effective_catalog_controls_revision_and_aliases() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("off".into(), false);
        catalog.aliases.insert(
            "disabled_group".into(),
            Alias::new("nope".into(), Some("off".into()), true, false),
        );
        catalog
            .aliases
            .insert("global".into(), Alias::new("*.rs".into(), None, true, true));
        catalog.aliases.insert("invalid name".into(), alias("nope"));

        let bash =
            generate_reconciliation_script(&catalog, &ShellType::Bash, "", "different", true);
        assert!(!bash.contains("disabled_group"));
        assert!(!bash.contains("global="));
        assert!(!bash.contains("invalid name="));

        let zsh = generate_reconciliation_script(&catalog, &ShellType::Zsh, "", "different", true);
        assert!(zsh.contains("alias -g -- 'global=*.rs'"));
    }

    #[test]
    fn shell_values_are_safely_quoted() {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("quote'alias".into(), alias("printf '%s' \"$HOME\""));

        let script =
            generate_reconciliation_script(&catalog, &ShellType::Bash, "old'alias", "", false);
        assert!(script.contains("unalias -- 'old'\"'\"'alias'"));
        assert!(script.contains("alias -- 'quote'\"'\"'alias=printf '"));
        assert!(!script.contains("alias -- 'quote'alias="));
    }

    #[test]
    fn sorting_does_not_change_revision() {
        let mut first = AliasCatalog::new();
        first.aliases.insert("b".into(), alias("two"));
        first.aliases.insert("a".into(), alias("one"));
        let mut second = AliasCatalog::new();
        second.aliases.insert("a".into(), alias("one"));
        second.aliases.insert("b".into(), alias("two"));

        assert_eq!(
            catalog_revision(&active_aliases(&first, &ShellType::Bash)),
            catalog_revision(&active_aliases(&second, &ShellType::Bash))
        );
    }
}
