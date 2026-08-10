use clap::Args;

use super::validate_tag;

#[derive(Args)]
pub struct EditCommand {
    /// Alias to edit
    pub name: String,

    /// Replacement command
    pub command: Option<String>,

    /// Set the alias description
    #[arg(long, conflicts_with = "clear_description")]
    pub description: Option<String>,

    /// Remove the alias description
    #[arg(long)]
    pub clear_description: bool,

    /// Add a tag; repeat to add multiple tags
    #[arg(long, value_name = "TAG", value_parser = validate_tag)]
    pub add_tag: Vec<String>,

    /// Remove a tag; repeat to remove multiple tags
    #[arg(long, value_name = "TAG", value_parser = validate_tag)]
    pub remove_tag: Vec<String>,

    /// Toggle whether the alias is enabled
    #[arg(long, short = 'e')]
    pub toggle_enabled: bool,

    /// Toggle whether the alias is global
    #[arg(long, short = 'b')]
    pub toggle_global: bool,
}

impl EditCommand {
    pub fn has_changes(&self) -> bool {
        self.command.is_some()
            || self.description.is_some()
            || self.clear_description
            || !self.add_tag.is_empty()
            || !self.remove_tag.is_empty()
            || self.toggle_enabled
            || self.toggle_global
    }
}
