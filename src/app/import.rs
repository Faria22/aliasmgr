use std::fs;

use log::warn;

use crate::catalog::types::{Alias, AliasCatalog};
use crate::cli::import::ImportCommand;
use crate::cli::interaction::{InteractionMode, prompt_replace_imported_alias};
use crate::core::import::{ParsedLine, is_identical, parse_alias_line};
use crate::core::{Failure, Outcome};

use super::CommandOutcome;

#[derive(Clone, Copy)]
enum CollisionPolicy {
    Prompt,
    Skip,
    Replace,
}

#[derive(Default)]
struct ImportSummary {
    imported: usize,
    collisions: usize,
    replaced: usize,
    skipped_collisions: usize,
    unsupported: usize,
    unchanged: usize,
}

pub fn handle_import(
    catalog: &mut AliasCatalog,
    args: ImportCommand,
    interaction_mode: InteractionMode,
    force: bool,
) -> Result<CommandOutcome, Failure> {
    let policy = if force || args.replace_existing {
        CollisionPolicy::Replace
    } else if args.skip_existing {
        CollisionPolicy::Skip
    } else {
        CollisionPolicy::Prompt
    };
    let mut candidate = catalog.clone();
    let mut summary = ImportSummary::default();

    for path in &args.paths {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) => {
                warn!(
                    "Could not import '{}': {error}; skipping file.",
                    path.display()
                );
                continue;
            }
        };

        for line in content.lines() {
            let (name, command, global) = match parse_alias_line(line) {
                ParsedLine::Alias {
                    name,
                    command,
                    global,
                } => (name, command, global),
                ParsedLine::Unsupported => {
                    summary.unsupported += 1;
                    continue;
                }
                ParsedLine::Ignored => continue,
            };

            if let Some(existing) = candidate.aliases.get(&name) {
                if is_identical(existing, &command, global) {
                    summary.unchanged += 1;
                    continue;
                }

                summary.collisions += 1;
                let replace = match policy {
                    CollisionPolicy::Replace => true,
                    CollisionPolicy::Skip => false,
                    CollisionPolicy::Prompt if args.dry_run => false,
                    CollisionPolicy::Prompt => {
                        prompt_replace_imported_alias(interaction_mode, &name)
                    }
                };
                if !replace {
                    if !args.dry_run || matches!(policy, CollisionPolicy::Skip) {
                        summary.skipped_collisions += 1;
                    }
                    continue;
                }
                summary.replaced += 1;
            } else {
                summary.imported += 1;
            }

            let mut alias = Alias::new(command, true, global);
            alias.tags.extend(args.tag.iter().cloned());
            alias.refresh_representation();
            candidate.aliases.insert(name, alias);
        }
    }

    let changed = summary.imported + summary.replaced > 0;
    if changed && !args.dry_run {
        *catalog = candidate;
    }
    let outcome = if changed && !args.dry_run {
        Outcome::CatalogChanged
    } else {
        Outcome::NoChanges
    };
    Ok(CommandOutcome::with_message(
        outcome,
        format_summary(&summary, args.dry_run, policy),
    ))
}

fn format_summary(summary: &ImportSummary, dry_run: bool, policy: CollisionPolicy) -> String {
    let mut parts = if dry_run {
        vec![format!(
            "Dry run: {} aliases would be imported",
            summary.imported
        )]
    } else {
        vec![format!("Imported {} aliases", summary.imported)]
    };

    if summary.collisions > 0 {
        let collision_result = if dry_run {
            match policy {
                CollisionPolicy::Prompt => String::new(),
                CollisionPolicy::Skip => " and would be skipped".into(),
                CollisionPolicy::Replace => " and would be replaced".into(),
            }
        } else {
            format!(
                "; {} replaced and {} skipped",
                summary.replaced, summary.skipped_collisions
            )
        };
        parts.push(format!(
            "{} collisions found{collision_result}",
            summary.collisions
        ));
    }
    if summary.unchanged > 0 {
        parts.push(format!("{} aliases unchanged", summary.unchanged));
    }
    if summary.unsupported > 0 {
        parts.push(format!("{} unsupported lines skipped", summary.unsupported));
    }
    format!("{}.", parts.join("; "))
}
