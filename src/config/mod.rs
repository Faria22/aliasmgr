pub(crate) mod io;
pub(crate) mod spec;
pub(crate) mod types;

#[cfg(test)]
mod tests {
    use crate::cli::{Cli, Commands};
    use crate::config::io::{load_config, save_config};
    use crate::config::spec::{ConfigSpec, convert_config_to_spec, convert_spec_to_config};
    use crate::config::types::{Alias, Config, Group};
    use assert_fs::TempDir;
    use std::collections::HashMap;
    use std::fs;

    fn sample_toml() -> &'static str {
        r#"
        py = "python3"
        js = { command = "node", enable = false }
        [git]
        ga = "git add"
        gc = { command = "git commit", enable = true }
        [foo]
        enable = false
        bar = "echo 'Hello World'"
        ll = { command = "ls -la", enable = true }
        "#
    }

    fn expected_config() -> Config {
        let mut aliases = HashMap::new();
        let mut groups = HashMap::new();
        aliases.insert(
            "py".into(),
            Alias {
                command: "python3".into(),
                enable: true,
                group: None,
                detailed: false,
            },
        );

        aliases.insert(
            "js".into(),
            Alias {
                command: "node".into(),
                enable: false,
                group: None,
                detailed: true,
            },
        );

        aliases.insert(
            "ga".into(),
            Alias {
                command: "git add".into(),
                enable: true,
                group: Some("git".into()),
                detailed: false,
            },
        );

        aliases.insert(
            "gc".into(),
            Alias {
                command: "git commit".into(),
                enable: true,
                group: Some("git".into()),
                detailed: true,
            },
        );

        aliases.insert(
            "bar".into(),
            Alias {
                command: "echo 'Hello World'".into(),
                enable: true,
                group: Some("foo".into()),
                detailed: false,
            },
        );

        aliases.insert(
            "ll".into(),
            Alias {
                command: "ls -la".into(),
                enable: true,
                group: Some("foo".into()),
                detailed: true,
            },
        );

        groups.insert("git".into(), Group { enable: true });
        groups.insert("foo".into(), Group { enable: false });

        Config { aliases, groups }
    }

    fn sample_cli(config_path: &std::path::Path) -> Cli {
        Cli {
            config: Some(config_path.to_path_buf()),
            command: Commands::Sync,
            debug: false,
            verbose: false,
            quiet: false,
        }
    }

    #[test]
    fn test_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");
        fs::write(&temp_conf, sample_toml()).unwrap();

        let cli = sample_cli(&temp_conf);
        let cfg = load_config(&cli).unwrap();
        assert_eq!(cfg, expected_config());
    }

    #[test]
    fn test_save_config() {
        let temp_dir = TempDir::new().unwrap();
        let temp_conf = temp_dir.path().join("aliases.toml");

        let cli = sample_cli(&temp_conf);
        let config = expected_config();
        save_config(&cli, &config).unwrap();

        let saved_content = fs::read_to_string(&temp_conf).unwrap();
        let parsed_spec: ConfigSpec = toml::from_str(&saved_content).unwrap();
        let parsed_config = convert_spec_to_config(parsed_spec);
        assert_eq!(parsed_config, config);
    }

    #[test]
    fn test_convert_spec_to_config() {
        let spec: ConfigSpec = toml::from_str(sample_toml()).unwrap();
        let config = convert_spec_to_config(spec);
        assert_eq!(config, expected_config());
    }

    #[test]
    fn test_convert_config_roundtrip() {
        let config = expected_config();
        let spec = convert_config_to_spec(&config);
        let converted = convert_spec_to_config(spec);
        assert_eq!(converted, config);
    }
}
