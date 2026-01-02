// v0.0.663: Settings Graph - Tests
// Test suite for settings graph functionality

#[cfg(test)]
mod tests {
    use super::super::link_types::{LinkDirection, LinkType};
    use super::super::linker::{SettingsLinker, SettingsLinkerRegistry};
    use super::super::linker_config::LinkerConfig;
    use super::super::settings_link::{LinkResult, LinkerStats, SettingsLink};
    use super::super::utils::{graph_fun_fact, is_graph_query};
    use std::collections::HashMap;

    #[test]
    fn test_link_type_display() {
        assert_eq!(format!("{}", LinkType::Reference), "reference");
        assert_eq!(format!("{}", LinkType::Alias), "alias");
    }

    #[test]
    fn test_link_direction_display() {
        assert_eq!(format!("{}", LinkDirection::Unidirectional), "unidirectional");
        assert_eq!(format!("{}", LinkDirection::Bidirectional), "bidirectional");
    }

    #[test]
    fn test_config_new() {
        let c = LinkerConfig::new(LinkType::Reference);
        assert!(!c.allow_circular);
    }

    #[test]
    fn test_config_builder() {
        let c = LinkerConfig::new(LinkType::Dependency)
            .direction(LinkDirection::Bidirectional)
            .allow_circular(true);
        assert_eq!(c.default_direction, LinkDirection::Bidirectional);
        assert!(c.allow_circular);
    }

    #[test]
    fn test_link_new() {
        let l = SettingsLink::new("link_1", "source", "target");
        assert_eq!(l.source, "source");
        assert_eq!(l.target, "target");
    }

    #[test]
    fn test_link_with_type() {
        let l = SettingsLink::new("link_1", "s", "t").with_type(LinkType::Alias);
        assert_eq!(l.link_type, LinkType::Alias);
    }

    #[test]
    fn test_result_new() {
        let r = LinkResult::new();
        assert_eq!(r.total_links, 0);
    }

    #[test]
    fn test_result_add_created() {
        let mut r = LinkResult::new();
        r.add_created("link_1".to_string());
        assert_eq!(r.total_links, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = LinkerStats::default();
        s.record(LinkType::Reference);
        assert_eq!(s.total_links, 1);
    }

    #[test]
    fn test_linker_new() {
        let l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        assert_eq!(l.link_count(), 0);
    }

    #[test]
    fn test_linker_link() {
        let mut l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        let r = l.link("source", "target");
        assert!(r.success());
        assert_eq!(l.link_count(), 1);
    }

    #[test]
    fn test_linker_resolve() {
        let mut l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        l.link("alias", "actual");

        let mut settings = HashMap::new();
        settings.insert("actual".to_string(), "value".to_string());

        let resolved = l.resolve("alias", &settings);
        assert_eq!(resolved, Some("value".to_string()));
    }

    #[test]
    fn test_linker_circular_prevention() {
        let mut l = SettingsLinker::new(LinkerConfig::new(LinkType::Reference));
        l.link("a", "b");
        l.link("b", "c");
        let r = l.link("c", "a"); // Would create circular
        assert!(r.has_failures());
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsLinkerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsLinkerRegistry::new();
        r.register("l1", SettingsLinker::new(LinkerConfig::new(LinkType::Reference)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_graph_query() {
        assert!(is_graph_query("settings graph"));
        assert!(!is_graph_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = graph_fun_fact();
        assert!(fact.contains("graph"));
    }
}
