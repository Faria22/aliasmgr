use clap::{ArgGroup, Args};

#[derive(Args)]
#[command(
    group(
        ArgGroup::new("alias_selector")
            .args(["name", "pattern", "group"])
            .required(true)
            .multiple(true)
    )
)]
pub struct AliasSelectorArgs {
    /// Exact alias name
    #[arg(conflicts_with_all = ["pattern", "group"])]
    pub name: Option<String>,

    /// Select aliases whose names match GLOB
    #[arg(long, value_name = "GLOB", value_parser = validate_glob)]
    pub pattern: Option<String>,

    /// Select aliases in GROUP. If left empty, select ungrouped aliases.
    #[arg(short, long, num_args = 0..=1, value_name = "GROUP")]
    pub group: Option<Option<String>>,
}

impl AliasSelectorArgs {
    pub fn is_filter(&self) -> bool {
        self.pattern.is_some() || self.group.is_some()
    }
}

fn validate_glob(pattern: &str) -> Result<String, String> {
    globset::Glob::new(pattern)
        .map(|_| pattern.to_string())
        .map_err(|error| format!("invalid glob pattern: {error}"))
}
