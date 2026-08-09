pub(crate) mod add;
pub(crate) mod disable;
pub(crate) mod doctor;
pub(crate) mod edit;
pub(crate) mod enable;
pub(crate) mod file_path;
pub(crate) mod init;
pub(crate) mod list;
pub(crate) mod r#move;
pub(crate) mod remove;
pub(crate) mod rename;
pub(crate) mod resource;
pub(crate) mod shell;
pub(crate) mod sort;
pub(crate) mod sync;

use crate::core::Outcome;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CommandOutcome {
    pub outcome: Outcome,
    pub message: Option<String>,
}

impl CommandOutcome {
    pub fn with_message(outcome: Outcome, message: impl Into<String>) -> Self {
        Self {
            outcome,
            message: Some(message.into()),
        }
    }
}

impl From<Outcome> for CommandOutcome {
    fn from(outcome: Outcome) -> Self {
        Self {
            outcome,
            message: None,
        }
    }
}
