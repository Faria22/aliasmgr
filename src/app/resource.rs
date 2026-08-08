use crate::catalog::types::AliasCatalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Alias,
    Group,
}

pub fn resolve_resource_type(
    catalog: &AliasCatalog,
    name: &str,
    choose_alias: impl FnOnce(&str) -> bool,
) -> ResourceType {
    match (
        catalog.aliases.contains_key(name),
        catalog.groups.contains_key(name),
    ) {
        (true, true) if choose_alias(name) => ResourceType::Alias,
        (true, true) | (false, true) => ResourceType::Group,
        (true, false) | (false, false) => ResourceType::Alias,
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;

    fn choose_alias(_: &str) -> bool {
        true
    }

    fn choose_group(_: &str) -> bool {
        false
    }

    fn catalog_with_collision() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog.aliases.insert(
            "tools".to_string(),
            Alias::new("echo tools".to_string(), None, true, false),
        );
        catalog.groups.insert("tools".to_string(), true);
        catalog
    }

    #[test]
    fn defaults_to_alias_when_no_resource_exists() {
        assert_eq!(
            resolve_resource_type(&AliasCatalog::new(), "tools", choose_alias),
            ResourceType::Alias
        );
    }

    #[test]
    fn resolves_single_existing_resource() {
        let mut catalog = catalog_with_collision();
        catalog.groups.clear();
        assert_eq!(
            resolve_resource_type(&catalog, "tools", choose_alias),
            ResourceType::Alias
        );

        catalog.aliases.clear();
        catalog.groups.insert("tools".to_string(), true);
        assert_eq!(
            resolve_resource_type(&catalog, "tools", choose_alias),
            ResourceType::Group
        );
    }

    #[test]
    fn collision_honors_selected_resource() {
        let catalog = catalog_with_collision();
        assert_eq!(
            resolve_resource_type(&catalog, "tools", choose_alias),
            ResourceType::Alias
        );
        assert_eq!(
            resolve_resource_type(&catalog, "tools", choose_group),
            ResourceType::Group
        );
    }
}
