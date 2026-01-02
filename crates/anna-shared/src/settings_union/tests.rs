// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_union_type_display() {
        assert_eq!(format!("{}", UnionType::Full), "full");
        assert_eq!(format!("{}", UnionType::Customs), "customs");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", UnionStatus::Proposed), "proposed");
        assert_eq!(format!("{}", UnionStatus::Integrated), "integrated");
    }

    #[test]
    fn test_config_new() {
        let c = UnionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = UnionConfig::new("test")
            .union_type(UnionType::Monetary)
            .status(UnionStatus::Ratified);
        assert_eq!(c.union_type, UnionType::Monetary);
        assert_eq!(c.status, UnionStatus::Ratified);
    }

    #[test]
    fn test_provision_new() {
        let p = UnionProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = UnionProvision::new("p1", "Title", "Content")
            .section(1);
        assert_eq!(p.section, 1);
    }

    #[test]
    fn test_provision_binding() {
        let mut p = UnionProvision::new("p1", "Title", "Content");
        p.make_advisory();
        assert!(!p.binding);
        p.make_binding();
        assert!(p.binding);
    }

    #[test]
    fn test_member_new() {
        let m = UnionMember::new("key", "name", "p1");
        assert_eq!(m.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = UnionStats::default();
        let provision = UnionProvision::new("p1", "Title", "Content");
        s.update(&[provision], UnionType::Full);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.binding, 1);
    }

    #[test]
    fn test_union_new() {
        let u = SettingsUnion::new(UnionConfig::default());
        assert_eq!(u.provision_count(), 0);
    }

    #[test]
    fn test_union_add_provision() {
        let mut u = SettingsUnion::new(UnionConfig::default());
        u.add_provision(UnionProvision::new("p1", "Title", "Content"));
        assert_eq!(u.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = UnionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = UnionRegistry::new();
        r.register("u1", SettingsUnion::new(UnionConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_union_query() {
        assert!(is_union_query("settings union"));
        assert!(!is_union_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = union_fun_fact();
        assert!(fact.contains("union"));
    }
}
