use crate::app::shell::ShellType;
use crate::catalog::types::AliasCatalog;
use crate::core::conflict::conflict_warnings;

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn is_valid_alias_name(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(char::is_whitespace) && !name.contains('=')
}

pub fn validate_catalog(catalog: &AliasCatalog, shell: &ShellType) -> ValidationReport {
    let mut report = ValidationReport {
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let valid_names = catalog
        .aliases
        .keys()
        .filter(|name| is_valid_alias_name(name))
        .map(String::as_str);
    let conflicts = conflict_warnings(valid_names, shell);

    for (name, alias) in &catalog.aliases {
        if !is_valid_alias_name(name) {
            report.errors.push(format!(
                "Alias '{name}' has an invalid name; names must not be empty or contain whitespace or '='."
            ));
        }

        if let Some(group) = &alias.group
            && !catalog.groups.contains_key(group)
        {
            report.errors.push(format!(
                "Alias '{name}' references missing group '{group}'."
            ));
        }

        if alias.global && *shell != ShellType::Zsh {
            report.warnings.push(format!(
                "Global alias '{name}' is unsupported in {shell} and will be skipped."
            ));
        }

        if let Some(conflict_warnings) = conflicts.get(name) {
            report.warnings.extend(conflict_warnings.iter().cloned());
        }
    }

    report
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::{Alias, AliasCatalog};

    fn alias(group: Option<&str>, global: bool) -> Alias {
        Alias::new("echo test".into(), group.map(str::to_owned), true, global)
    }

    #[test]
    fn invalid_alias_name_is_reported() {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("invalid name".into(), alias(None, false));

        let report = validate_catalog(&catalog, &ShellType::Zsh);

        assert!(!report.is_valid());
        assert!(report.errors[0].contains("invalid name"));
    }

    #[test]
    fn missing_group_reference_is_reported() {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("build".into(), alias(Some("missing"), false));

        let report = validate_catalog(&catalog, &ShellType::Zsh);

        assert!(!report.is_valid());
        assert!(report.errors[0].contains("missing group 'missing'"));
    }

    #[test]
    fn global_alias_incompatibility_is_a_bash_warning() {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert("glob".into(), alias(None, true));

        let report = validate_catalog(&catalog, &ShellType::Bash);

        assert!(report.is_valid());
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("unsupported in BASH"));
    }

    #[test]
    fn valid_catalog_returns_success() {
        let mut catalog = AliasCatalog::new();
        catalog.groups.insert("dev".into(), true);
        catalog
            .aliases
            .insert("build".into(), alias(Some("dev"), false));
        catalog.aliases.insert("glob".into(), alias(None, true));

        let report = validate_catalog(&catalog, &ShellType::Zsh);

        assert!(report.is_valid());
        assert!(report.errors.is_empty());
        assert!(report.warnings.is_empty());
    }
}
