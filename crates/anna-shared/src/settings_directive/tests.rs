// v0.0.718: Settings Directive Tests (Phase 294)
// Test suite for directive systems

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_directive_type_display() {
        assert_eq!(format!("{}", DirectiveType::Mandatory), "mandatory");
        assert_eq!(format!("{}", DirectiveType::Optional), "optional");
    }

    #[test]
    fn test_authority_display() {
        assert_eq!(format!("{}", DirectiveAuthority::System), "system");
        assert_eq!(format!("{}", DirectiveAuthority::Executive), "executive");
    }

    #[test]
    fn test_config_new() {
        let c = DirectiveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DirectiveConfig::new("test")
            .directive_type(DirectiveType::Recommended)
            .authority(DirectiveAuthority::Admin);
        assert_eq!(c.directive_type, DirectiveType::Recommended);
        assert_eq!(c.authority, DirectiveAuthority::Admin);
    }

    #[test]
    fn test_order_new() {
        let o = DirectiveOrder::new("o1", "Title", "Instructions");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_order_builder() {
        let o = DirectiveOrder::new("o1", "Title", "Instructions")
            .authority(DirectiveAuthority::Policy);
        assert_eq!(o.authority, DirectiveAuthority::Policy);
    }

    #[test]
    fn test_order_enforce() {
        let mut o = DirectiveOrder::new("o1", "Title", "Instructions");
        o.enforce();
        assert!(o.enforced);
    }

    #[test]
    fn test_supplement_new() {
        let s = DirectiveSupplement::new("key", "value", "o1");
        assert_eq!(s.order_id, "o1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DirectiveStats::default();
        let mut order = DirectiveOrder::new("o1", "Title", "Instructions");
        order.enforce();
        s.update(&[order], DirectiveType::Mandatory);
        assert_eq!(s.total_directives, 1);
        assert_eq!(s.enforced, 1);
        assert_eq!(s.mandatory_count, 1);
    }

    #[test]
    fn test_directive_new() {
        let d = SettingsDirective::new(DirectiveConfig::default());
        assert_eq!(d.order_count(), 0);
    }

    #[test]
    fn test_directive_add_order() {
        let mut d = SettingsDirective::new(DirectiveConfig::default());
        d.add_order(DirectiveOrder::new("o1", "Title", "Instructions"));
        assert_eq!(d.order_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DirectiveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DirectiveRegistry::new();
        r.register("d1", SettingsDirective::new(DirectiveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_directive_query() {
        assert!(is_directive_query("settings directive"));
        assert!(!is_directive_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = directive_fun_fact();
        assert!(fact.contains("directive"));
    }
}
