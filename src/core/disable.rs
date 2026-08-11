use super::status::{set_alias, set_aliases, set_all};
use super::{Failure, Outcome};
use crate::catalog::types::AliasCatalog;

pub fn disable_alias(catalog: &mut AliasCatalog, name: &str) -> Result<Outcome, Failure> {
    set_alias(catalog, name, false)
}

pub fn disable_aliases(catalog: &mut AliasCatalog, names: &[String]) -> (Outcome, usize) {
    set_aliases(catalog, names, false)
}

pub fn disable_all(catalog: &mut AliasCatalog) -> Outcome {
    set_all(catalog, false)
}
