pub(crate) mod add;
pub(crate) mod disable;
pub(crate) mod edit;
pub(crate) mod enable;
pub(crate) mod list;
pub(crate) mod r#move;
pub(crate) mod remove;
pub(crate) mod rename;
pub(crate) mod sort;
pub(crate) mod sync;

/// Represents possible failure cases in core operations.
#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    InvalidAliasName,
    UnsupportedGlobalAlias,
    AliasDoesNotExist,
    GroupDoesNotExist,
    AliasAlreadyExists,
    GroupAlreadyExists,
}

/// Represents the outcome of core operations.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The catalog changed and must be saved.
    CatalogChanged,

    /// No changes were made
    NoChanges,
}
