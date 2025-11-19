use crate::core::{Failure, Outcome};

use crate::core::add::{add_alias, add_group};
use crate::core::edit::edit_alias;
use crate::core::r#move::move_alias;

use crate::config::types::Config;

use crate::cli::interaction::{create_non_existent_group, overwrite_existing_alias};
use crate::cli::{AddCommand, AddTarget};

use log::info;

pub fn handle_add(cmd: AddCommand, config: &mut Config) -> Result<Outcome, Failure> {
    match cmd.target {
        // Add alias
        AddTarget::Alias(args) => match add_alias(
            config,
            &args.name,
            &args.command,
            args.group.as_deref(),
            !args.disabled,
        ) {
            // Alias added successfully
            Ok(outcome) => Ok(outcome),

            // Handle errors
            Err(e) => match e {
                // Alias already exists
                Failure::AliasAlreadyExists => {
                    // If the alias already exists, we check if the user wants to overwrite it
                    if overwrite_existing_alias(&args.name) {
                        // User wants to overwrite the existing alias
                        info!("Overwriting existing alias '{}'.", &args.name);
                        let command = edit_alias(config, &args.name, &args.command)?;

                        // Move alias to new group if it is different from the previous one
                        if args.group
                            != config.aliases.get(&args.name).and_then(|a| a.group.clone())
                        {
                            info!("Moving alias '{}' to group '{:?}'.", &args.name, args.group);
                            if let Err(Failure::GroupDoesNotExist) =
                                move_alias(&args.name, &args.group)
                            {
                                // If the group does not exist, we ask the user if they want to create it
                                if create_non_existent_group(
                                    args.group
                                        .as_deref()
                                        .expect("group has to be `Some` for this error to arise"),
                                ) {
                                    // User wants to create the group
                                    info!(
                                        "Creating group '{:?}' for alias '{}'.",
                                        args.group, &args.name
                                    );
                                    add_group(
                                        config,
                                        args.group.as_deref().expect(
                                            "group has to be `Some` for this error to arise",
                                        ),
                                        true,
                                    )?;
                                } else {
                                    // User does not want to create the group
                                    info!(
                                        "Alias '{}' was not moved due to missing group '{:?}' not being added",
                                        &args.name, args.group
                                    );
                                    return Ok(Outcome::NoChanges);
                                }
                            }
                        }

                        // Update enabled status if it is different from the previous one
                        if !args.disabled != !config.aliases.get(&args.name).unwrap().enabled {
                            let alias = config.aliases.get_mut(&args.name).unwrap();
                            alias.enabled = !args.disabled;
                            info!(
                                "Setting alias '{}' enabled status to '{}'.",
                                &args.name, alias.enabled
                            );
                        }

                        // Returns command to edit the alias in the shell
                        Ok(command)
                    } else {
                        // User does not want to overwrite the existing alias
                        info!("Not overwriting existing alias '{}'.", &args.name);
                        Ok(Outcome::NoChanges)
                    }
                }
                // Group that alias will belong to does not exist
                Failure::GroupDoesNotExist => {
                    if create_non_existent_group(
                        args.group
                            .as_deref()
                            .expect("group has to be `Some` for this error to arise"),
                    ) {
                        // User wants to create the group
                        info!(
                            "Creating group '{:?}' for alias '{}'.",
                            args.group, &args.name
                        );
                        add_group(
                            config,
                            args.group
                                .as_deref()
                                .expect("group has to be `Some` for this error to arise"),
                            true,
                        )?;
                        // Retry adding the alias after creating the group
                        add_alias(
                            config,
                            &args.name,
                            &args.command,
                            args.group.as_deref(),
                            !args.disabled,
                        )
                    } else {
                        // User does not want to create the group
                        info!(
                            "Alias '{}' was not added due to missing group '{:?}' not being added",
                            &args.name, args.group
                        );
                        Ok(Outcome::NoChanges)
                    }
                }
                _ => unreachable!("Unexpected error encountered: {:?}", e),
            },
        },

        // Add group
        AddTarget::Group(args) => add_group(config, &args.name, !args.disabled),
    }
}
