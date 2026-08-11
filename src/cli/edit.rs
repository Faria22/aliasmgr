use clap::Args;

use super::validate_tag;

#[derive(Args)]
pub struct EditCommand {
    /// Alias to edit
    pub name: String,

    /// Replacement command
    pub command: Option<String>,

    /// Add a tag; repeat to add multiple tags
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub add_tag: Vec<String>,

    /// Remove a tag; repeat to remove multiple tags
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub remove_tag: Vec<String>,

    /// Set the alias description
    #[arg(short, long, conflicts_with = "clear_description")]
    pub description: Option<String>,

    /// Remove the alias description
    #[arg(long)]
    pub clear_description: bool,

    /// Make the alias global
    #[arg(short, long, conflicts_with = "no_global")]
    pub global: bool,

    /// Make the alias non-global
    #[arg(long)]
    pub no_global: bool,
}

impl EditCommand {
    pub fn has_changes(&self) -> bool {
        self.command.is_some()
            || self.description.is_some()
            || self.clear_description
            || !self.add_tag.is_empty()
            || !self.remove_tag.is_empty()
            || self.global
            || self.no_global
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn command() -> EditCommand {
        EditCommand {
            name: "test".into(),
            command: None,
            description: None,
            clear_description: false,
            add_tag: vec![],
            remove_tag: vec![],
            global: false,
            no_global: false,
        }
    }

    #[test]
    fn every_edit_option_counts_as_a_change() {
        assert!(!command().has_changes());
        let mut variants = Vec::new();
        let mut value = command();
        value.command = Some("cmd".into());
        variants.push(value);
        let mut value = command();
        value.description = Some("description".into());
        variants.push(value);
        let mut value = command();
        value.clear_description = true;
        variants.push(value);
        let mut value = command();
        value.add_tag.push("tag".into());
        variants.push(value);
        let mut value = command();
        value.remove_tag.push("tag".into());
        variants.push(value);
        let mut value = command();
        value.global = true;
        variants.push(value);
        let mut value = command();
        value.no_global = true;
        variants.push(value);
        assert!(variants.iter().all(EditCommand::has_changes));
    }
}
