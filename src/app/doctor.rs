use crate::app::shell::ShellType;
use crate::catalog::types::AliasCatalog;
use crate::cli::doctor::DoctorCommand;
use crate::core::validation::{ValidationReport, validate_catalog};
use crate::core::{Failure, Outcome};

fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

fn format_report(report: &ValidationReport, shell: &ShellType, quiet: bool) -> (String, String) {
    let mut standard = String::new();
    let mut diagnostics = String::new();

    for error in &report.errors {
        diagnostics.push_str(&format!("ERROR: {error}\n"));
    }
    if !quiet {
        for warning in &report.warnings {
            diagnostics.push_str(&format!("WARNING: {warning}\n"));
        }
    }

    if quiet {
        return (standard, diagnostics);
    } else if report.errors.is_empty() && report.warnings.is_empty() {
        standard.push_str(&format!("OK: Catalog is valid for {shell}.\n"));
    } else {
        standard.push_str(&format!(
            "Validation found {} {} and {} {}.\n",
            report.errors.len(),
            plural(report.errors.len(), "error", "errors"),
            report.warnings.len(),
            plural(report.warnings.len(), "warning", "warnings"),
        ));
    }

    (standard, diagnostics)
}

pub fn handle_doctor(
    catalog: &AliasCatalog,
    _cmd: DoctorCommand,
    shell: &ShellType,
    quiet: bool,
) -> Result<Outcome, Failure> {
    let report = validate_catalog(catalog, shell);
    let (standard, diagnostics) = format_report(&report, shell, quiet);
    print!("{standard}");
    eprint!("{diagnostics}");

    if report.is_valid() {
        Ok(Outcome::NoChanges)
    } else {
        Err(Failure::InvalidCatalog)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn report_distinguishes_errors_and_warnings() {
        let report = ValidationReport {
            errors: vec!["bad alias".into()],
            warnings: vec!["shell mismatch".into()],
        };

        let (standard, diagnostics) = format_report(&report, &ShellType::Bash, false);

        assert_eq!(standard, "Validation found 1 error and 1 warning.\n");
        assert!(diagnostics.contains("ERROR: bad alias"));
        assert!(diagnostics.contains("WARNING: shell mismatch"));
    }

    #[test]
    fn valid_report_names_active_shell() {
        let report = ValidationReport {
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        let (standard, diagnostics) = format_report(&report, &ShellType::Zsh, false);

        assert_eq!(standard, "OK: Catalog is valid for ZSH.\n");
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn quiet_report_only_prints_errors() {
        let report = ValidationReport {
            errors: vec!["bad alias".into()],
            warnings: vec!["shell mismatch".into()],
        };

        let (standard, diagnostics) = format_report(&report, &ShellType::Bash, true);

        assert!(standard.is_empty());
        assert_eq!(diagnostics, "ERROR: bad alias\n");
    }
}
