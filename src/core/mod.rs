pub(crate) mod add;
pub(crate) mod edit;
pub(crate) mod list;
pub(crate) mod r#move;
pub(crate) mod remove;
pub(crate) mod sync;

/// Represents possible failure cases in core operations.
#[derive(Debug)]
pub enum Failure {
    InvalidAliasName,
    AliasDoesNotExist,
    GroupDoesNotExist,
    AliasAlreadyExists,
    GroupAlreadyExists,
}

/// Represents the outcome of core operations.
#[derive(Debug)]
pub enum Outcome {
    /// Contains the command that has to be executed by the shell once everything is done
    /// It is assumed that the config also needs to be updated in this case
    Command(String),

    /// Configuration has changed but shell does not need to be updated
    /// If the configuration has changed, we need to know so that we can update the config file
    ConfigChanged,

    /// No changes were made
    NoChanges,
}
