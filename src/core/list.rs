use crate::app::shell::ShellType;
use crate::catalog::types::AliasCatalog;

pub fn visible_aliases<'a>(
    catalog: &'a AliasCatalog,
    shell: &ShellType,
) -> impl Iterator<Item = (&'a String, &'a crate::catalog::types::Alias)> {
    catalog
        .aliases
        .iter()
        .filter(move |(_, alias)| !alias.global || *shell == ShellType::Zsh)
}
