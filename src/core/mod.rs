pub(crate) mod add;
pub(crate) mod edit;
pub(crate) mod list;
pub(crate) mod sync;

use edit::EditError;

pub enum ReturnError {
    Edit(EditError),
    InvalidInput,
}

pub enum ReturnStatus {
    Success,
    NoChanges,
}
