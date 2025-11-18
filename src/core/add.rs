use crate::config::types::{Alias, Config};
use crate::core::edit::edit_alias;
use log::info;

enum AddError {
    GroupAlreadyExists,
    AliasAlreadyExists,
    GroupDoesNotExist,
}

pub fn add_alias(
    config: &mut Config,
    name: &str,
    command: &str,
    group: Option<&str>,
    enabled: bool,
) -> bool {
    if let Err(e) = add_alias_to_config(config, name, command, group, enabled) {
        match e {
            AddError::AliasAlreadyExists => {
                println!("Alias '{}' already exists.", name);
                println!("Would you like to overwrite it? (y/N)");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" {
                    info!("Overwriting alias '{}'.", name);
                    return edit_alias(config, name, command);
                } else if input.trim().to_lowercase() != "n" && !input.trim().is_empty() {
                    eprintln!("Invalid input. Alias '{}' was not modified.", name);
                    return false;
                }
            }
            AddError::GroupDoesNotExist => {
                println!("Group '{:?}' does not exist.", group);
                println!("Would you like to create it? (y/N)");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" {
                    if let Some(g) = group {
                        info!("Creating group '{}'.", g);
                        add_group(config, g, enabled);
                    } else {
                        eprintln!("Error: No group name provided.");
                        return false;
                    }
                } else if input.trim().to_lowercase() != "n" && !input.trim().is_empty() {
                    eprintln!("Invalid input. Alias '{}' was not added.", name);
                    return false;
                }
            }
            _ => {
                eprintln!("Error: Failed to add alias '{}'.", name);
                return false;
            }
        }
    }

    info!("Alias '{}' added successfully.", name);
    true
}

pub fn add_group(config: &mut Config, name: &str, enabled: bool) -> bool {
    if let Err(e) = add_group_to_config(config, name, enabled) {
        match e {
            AddError::GroupAlreadyExists => {
                println!("Group '{}' already exists. No changes made.", name);
                return false;
            }
            _ => {
                eprintln!("Error: Failed to add group '{}'.", name);
                return false;
            }
        }
    }

    info!("Group '{}' added successfully.", name);
    true
}

fn add_alias_to_config(
    config: &mut Config,
    alias: &str,
    command: &str,
    group: Option<&str>,
    enabled: bool,
) -> Result<(), AddError> {
    // Check if alias already exists
    if config.aliases.contains_key(alias) {
        info!("Alias '{}' already exists.", alias);
        return Err(AddError::AliasAlreadyExists);
    }

    if group.is_some_and(|g| !config.groups.contains_key(g)) {
        info!("Group '{:?}' does not exist.", group);
        return Err(AddError::GroupDoesNotExist);
    }

    config.aliases.insert(
        alias.into(),
        Alias {
            command: command.into(),
            group: group.map(|g| g.to_string()),
            enabled,
            detailed: if enabled { false } else { true },
        },
    );

    info!("Alias '{}' added with command '{}'.", alias, command);

    Ok(())
}

fn add_group_to_config(config: &mut Config, group: &str, enabled: bool) -> Result<(), AddError> {
    if config.groups.contains_key(group) {
        info!("Group '{}' already exists.", group);
        return Err(AddError::GroupAlreadyExists);
    }

    config.groups.insert(group.into(), enabled);

    info!("Group '{}' added with enabled status '{}'.", group, enabled);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_alias_to_empty_config() {
        let mut config = Config::new();
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, true);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias {
                command: "ls -la".into(),
                group: None,
                detailed: false,
                enabled: true,
            })
        );
    }

    #[test]
    fn add_alias_to_existing_config() {
        let mut config = Config::new();
        config.aliases.insert(
            "gs".into(),
            Alias {
                command: "git status".into(),
                group: Some("git".into()),
                detailed: false,
                enabled: true,
            },
        );
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, true);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("gs"),
            Some(&Alias {
                command: "git status".into(),
                group: Some("git".into()),
                enabled: true,
                detailed: false,
            })
        );
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias {
                command: "ls -la".into(),
                group: None,
                detailed: false,
                enabled: true,
            })
        );
    }

    #[test]
    fn add_disabled_alias() {
        let mut config = Config::new();
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, false);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias {
                command: "ls -la".into(),
                group: None,
                detailed: true,
                enabled: false,
            })
        );
    }

    #[test]
    fn add_existing_alias() {
        let mut config = Config::new();
        config.aliases.insert(
            "ll".into(),
            Alias {
                command: "ls -l".into(),
                group: None,
                detailed: false,
                enabled: true,
            },
        );
        let result = add_alias_to_config(&mut config, "ll", "ls -la", None, true);
        assert!(result.is_err());
    }

    #[test]
    fn add_alias_to_nonexistent_group() {
        let mut config = Config::new();
        let result = add_alias_to_config(&mut config, "ll", "ls -la", Some("nonexistent"), true);
        assert!(result.is_err());
        assert!(matches!(result, Err(AddError::GroupDoesNotExist)));
    }

    #[test]
    fn add_alias_to_existing_group() {
        let mut config = Config::new();
        config.groups.insert("file_ops".into(), true);
        let result = add_alias_to_config(&mut config, "ll", "ls -la", Some("file_ops"), true);
        assert!(result.is_ok());
        assert_eq!(
            config.aliases.get("ll"),
            Some(&Alias {
                command: "ls -la".into(),
                group: Some("file_ops".into()),
                detailed: false,
                enabled: true,
            })
        );
        assert!(config.groups.contains_key("file_ops"))
    }

    #[test]
    fn add_group_to_new_config() {
        let mut config = Config::new();
        let result = add_group_to_config(&mut config, "dev_tools", true);
        assert!(result.is_ok());
        assert!(config.groups.contains_key("dev_tools"));
    }

    #[test]
    fn add_group_to_existing_config() {
        let mut config = Config::new();
        config.groups.insert("utils".into(), true);
        let result = add_group_to_config(&mut config, "dev_tools", true);
        assert!(result.is_ok());
        assert!(config.groups.contains_key("dev_tools"));
        assert!(config.groups.contains_key("utils"));
    }

    #[test]
    fn add_existing_group() {
        let mut config = Config::new();
        config.groups.insert("dev_tools".into(), true);
        let result = add_group_to_config(&mut config, "dev_tools", true);
        assert!(result.is_err());
        assert!(matches!(result, Err(AddError::GroupAlreadyExists)));
    }

    #[test]
    fn add_disabled_group() {
        let mut config = Config::new();
        let result = add_group_to_config(&mut config, "dev_tools", false);
        assert!(result.is_ok());
        assert_eq!(config.groups.get("dev_tools"), Some(&false));
    }
}
