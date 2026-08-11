//! Parse individual Bash and Zsh alias declarations.

use crate::catalog::types::Alias;
use crate::core::validation::is_valid_alias_name;

#[derive(Debug, PartialEq, Eq)]
pub enum ParsedLine {
    Ignored,
    Unsupported,
    Alias {
        name: String,
        command: String,
        global: bool,
    },
}

pub fn parse_alias_line(line: &str) -> ParsedLine {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return ParsedLine::Ignored;
    }

    let Some(mut declaration) = line.strip_prefix("alias") else {
        return ParsedLine::Unsupported;
    };
    if !declaration.starts_with(char::is_whitespace) {
        return ParsedLine::Unsupported;
    }
    declaration = declaration.trim_start();

    let global = if let Some(rest) = declaration.strip_prefix("-g") {
        if !rest.starts_with(char::is_whitespace) {
            return ParsedLine::Unsupported;
        }
        declaration = rest.trim_start();
        true
    } else {
        false
    };

    let Some((name, value)) = declaration.split_once('=') else {
        return ParsedLine::Unsupported;
    };
    if !is_valid_alias_name(name) {
        return ParsedLine::Unsupported;
    }

    let Some((command, remainder)) = parse_shell_word(value) else {
        return ParsedLine::Unsupported;
    };
    let remainder = remainder.trim_start();
    if !remainder.is_empty() && !remainder.starts_with('#') {
        return ParsedLine::Unsupported;
    }

    ParsedLine::Alias {
        name: name.into(),
        command,
        global,
    }
}

pub fn is_identical(existing: &Alias, command: &str, global: bool) -> bool {
    existing.command == command && existing.global == global
}

fn parse_shell_word(input: &str) -> Option<(String, &str)> {
    #[derive(Clone, Copy)]
    enum Quote {
        Single,
        Double,
    }

    let mut command = String::new();
    let mut chars = input.char_indices().peekable();
    let mut quote = None;
    let mut consumed = 0;
    let mut saw_content = false;

    while let Some((index, character)) = chars.next() {
        consumed = index + character.len_utf8();
        match quote {
            Some(Quote::Single) => {
                if character == '\'' {
                    quote = None;
                } else {
                    command.push(character);
                }
                saw_content = true;
            }
            Some(Quote::Double) => {
                if character == '"' {
                    quote = None;
                    saw_content = true;
                } else if matches!(character, '$' | '`') {
                    return None;
                } else if character == '\\' {
                    let (_, escaped) = chars.next()?;
                    consumed += escaped.len_utf8();
                    command.push(escaped);
                    saw_content = true;
                } else {
                    command.push(character);
                    saw_content = true;
                }
            }
            None => match character {
                '\'' | '"' => {
                    quote = Some(if character == '\'' {
                        Quote::Single
                    } else {
                        Quote::Double
                    });
                    saw_content = true;
                }
                '\\' => {
                    let (_, escaped) = chars.next()?;
                    consumed += escaped.len_utf8();
                    command.push(escaped);
                    saw_content = true;
                }
                character if character.is_whitespace() => {
                    consumed = index;
                    break;
                }
                '$' | '`' | '*' | '?' | '[' | '{' | '~' | ';' | '&' | '|' | '<' | '>' | '('
                | ')' => return None,
                _ => {
                    command.push(character);
                    saw_content = true;
                }
            },
        }
    }

    if quote.is_some() || !saw_content {
        return None;
    }
    Some((command, &input[consumed..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bash_and_zsh_aliases() {
        assert_eq!(
            parse_alias_line("alias ll='ls -la'"),
            ParsedLine::Alias {
                name: "ll".into(),
                command: "ls -la".into(),
                global: false,
            }
        );
        assert_eq!(
            parse_alias_line("  alias -g G=\"| grep\" # zsh global"),
            ParsedLine::Alias {
                name: "G".into(),
                command: "| grep".into(),
                global: true,
            }
        );
        assert_eq!(
            parse_alias_line(r"alias escaped=ls\ -la"),
            ParsedLine::Alias {
                name: "escaped".into(),
                command: "ls -la".into(),
                global: false,
            }
        );
        assert_eq!(
            parse_alias_line(r#"alias quoted="echo \"hello\"""#),
            ParsedLine::Alias {
                name: "quoted".into(),
                command: "echo \"hello\"".into(),
                global: false,
            }
        );
    }

    #[test]
    fn ignores_blank_and_comment_lines() {
        assert_eq!(parse_alias_line(""), ParsedLine::Ignored);
        assert_eq!(parse_alias_line("  # aliases"), ParsedLine::Ignored);
    }

    #[test]
    fn skips_unsupported_constructs() {
        for line in [
            "function ll() { ls -la; }",
            "alias ll='unterminated",
            "alias two='echo two'; echo extra",
            "alias a='one' b='two'",
            "alias invalid name='cmd'",
            "alias variable=\"echo $HOME\"",
            "alias command='safe'$(computed)",
            "alias glob=*.rs",
            "aliasfoo=bar",
            "alias -global value=bar",
            "alias ll",
        ] {
            assert_eq!(parse_alias_line(line), ParsedLine::Unsupported, "{line}");
        }
    }

    #[test]
    fn identical_aliases_ignore_catalog_only_metadata() {
        let mut alias = Alias::new("ls -la".into(), false, false);
        alias.description = Some("Files".into());
        alias.tags.insert("shell".into());
        assert!(is_identical(&alias, "ls -la", false));
        assert!(!is_identical(&alias, "ls", false));
        assert!(!is_identical(&alias, "ls -la", true));
    }
}
