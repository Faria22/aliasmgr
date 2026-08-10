pub(crate) mod add;
pub(crate) mod conflict;
pub(crate) mod disable;
pub(crate) mod edit;
pub(crate) mod enable;
pub(crate) mod list;
pub(crate) mod remove;
pub(crate) mod rename;
pub(crate) mod selector;
pub(crate) mod sync;
pub(crate) mod validation;

#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    InvalidAliasName,
    UnsupportedGlobalAlias,
    AliasDoesNotExist,
    AliasAlreadyExists,
    TagDoesNotExist,
    InvalidCatalog,
    InvalidPattern,
    InvalidColumns,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidAliasName => "invalid alias name",
            Self::UnsupportedGlobalAlias => "global aliases are only supported in zsh",
            Self::AliasDoesNotExist => "alias does not exist",
            Self::AliasAlreadyExists => "alias already exists",
            Self::TagDoesNotExist => "tag does not exist",
            Self::InvalidCatalog => "catalog is invalid",
            Self::InvalidPattern => "invalid glob pattern",
            Self::InvalidColumns => "list columns must not contain duplicates",
        };
        formatter.write_str(message)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    CatalogChanged,
    NoChanges,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn every_failure_has_a_human_readable_message() {
        let cases = [
            (Failure::InvalidAliasName, "invalid alias name"),
            (
                Failure::UnsupportedGlobalAlias,
                "global aliases are only supported in zsh",
            ),
            (Failure::AliasDoesNotExist, "alias does not exist"),
            (Failure::AliasAlreadyExists, "alias already exists"),
            (Failure::TagDoesNotExist, "tag does not exist"),
            (Failure::InvalidCatalog, "catalog is invalid"),
            (Failure::InvalidPattern, "invalid glob pattern"),
            (
                Failure::InvalidColumns,
                "list columns must not contain duplicates",
            ),
        ];
        for (failure, message) in cases {
            assert_eq!(failure.to_string(), message);
        }
    }
}
