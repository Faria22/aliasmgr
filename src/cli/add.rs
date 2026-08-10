use clap::{Args, Subcommand};

use super::validate_tag;

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true, subcommand_negates_reqs = true)]
pub struct AddCommand {
    #[command(subcommand)]
    pub target: Option<AddTarget>,

    #[command(flatten)]
    pub alias: ShorthandAddAliasArgs,
}

#[derive(Subcommand)]
pub enum AddTarget {
    /// Add a new alias
    #[command(visible_alias = "a")]
    Alias(AddAliasArgs),
}

#[derive(Args)]
pub struct ShorthandAddAliasArgs {
    #[arg(required = true)]
    pub name: Option<String>,
    #[arg(required = true)]
    pub command: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
    #[arg(short, long)]
    pub disabled: bool,
    #[arg(long)]
    pub global: bool,
}

impl ShorthandAddAliasArgs {
    pub fn into_alias_args(self) -> AddAliasArgs {
        AddAliasArgs {
            name: self.name.expect("clap requires a name"),
            command: self.command.expect("clap requires a command"),
            description: self.description,
            tag: self.tag,
            disabled: self.disabled,
            global: self.global,
        }
    }
}

#[derive(Args)]
pub struct AddAliasArgs {
    pub name: String,
    pub command: String,
    #[arg(long)]
    pub description: Option<String>,
    /// Add a tag; repeat to add multiple tags
    #[arg(short, long, value_name = "TAG", value_parser = validate_tag)]
    pub tag: Vec<String>,
    #[arg(short, long)]
    pub disabled: bool,
    #[arg(long)]
    pub global: bool,
}
