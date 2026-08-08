use crate::app::add::is_valid_alias_name;
use crate::app::shell::{ShellType, shell_quote};
use crate::catalog::types::{Alias, AliasCatalog};
use std::hash::{DefaultHasher, Hash, Hasher};

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

fn catalog_revision(active: &[ActiveAlias<'_>]) -> String {
    let mut aliases: Vec<_> = active.iter().collect();
    aliases.sort_unstable_by_key(|entry| entry.name);

    let mut hasher = DefaultHasher::new();
    for entry in aliases {
        (entry.name, &entry.alias.command, entry.alias.global).hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
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

    let unalias_commands = managed_aliases
        .lines()
        .filter(|name| !name.is_empty())
        .map(|name| format!("unalias -- {} 2>/dev/null || true", shell_quote(name)))
        .collect::<Vec<_>>()
        .join("\n");
    let alias_commands = active
        .iter()
        .map(|entry| {
            format!(
                "{} || __aliasmgr_sync_status=$?",
                alias_command(entry.name, entry.alias)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let names = active
        .iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"__aliasmgr_sync_status=0
{unalias_commands}
{alias_commands}
if [ "$__aliasmgr_sync_status" -eq 0 ]; then
    __aliasmgr_managed_aliases={}
    __aliasmgr_catalog_revision={}
    unset __aliasmgr_sync_status
else
    unset __aliasmgr_sync_status
    false
fi"#,
        shell_quote(&names),
        shell_quote(&revision),
    )
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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
