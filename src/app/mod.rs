pub(crate) mod add;
pub(crate) mod config_path;
pub(crate) mod r#move;
pub(crate) mod shell;

/// Returns the unique command header string used to identify when output to the user stops and
/// command output begins.
///
/// Command output is designed so that the shell can read it without having to use a file or other
/// methods.
///
/// It is also designed to only have the smallest amount of data it needs, that is why it is called
/// delta, because it only contains the changes needed to be applied to the current aliases.
pub const COMMAND_HEADER: &str = "__ALIASMGR_DELTA__";
