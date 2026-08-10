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

pub fn is_valid_tag(tag: &str) -> bool {
    !tag.is_empty() && tag.trim() == tag && !tag.chars().any(char::is_whitespace)
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
            report.errors.push(format!("Alias '{name}' has an invalid name; names must not be empty or contain whitespace or '='."));
        }
        for tag in &alias.tags {
            if !is_valid_tag(tag) {
                report.errors.push(format!("Alias '{name}' has invalid tag '{tag}'; tags must not be empty or contain whitespace."));
            }
        }
        if alias.global && *shell != ShellType::Zsh {
            report.warnings.push(format!(
                "Global alias '{name}' is unsupported in {shell} and will be skipped."
            ));
        }
        if let Some(warnings) = conflicts.get(name) {
            report.warnings.extend(warnings.iter().cloned());
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    #[test]
    fn invalid_tags_are_reported() {
        let mut catalog = AliasCatalog::new();
        let mut alias = Alias::new("cmd".into(), true, false);
        alias.tags.insert("bad tag".into());
        catalog.aliases.insert("test".into(), alias);
        assert!(validate_catalog(&catalog, &ShellType::Zsh).errors[0].contains("bad tag"));
    }
}
