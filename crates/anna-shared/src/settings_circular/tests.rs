// v0.0.717: Settings Circular - Tests (Phase 293)
// Unit tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_circular_type_display() {
        assert_eq!(format!("{}", CircularType::Policy), "policy");
        assert_eq!(format!("{}", CircularType::Advisory), "advisory");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", CircularScope::All), "all");
        assert_eq!(format!("{}", CircularScope::Team), "team");
    }

    #[test]
    fn test_config_new() {
        let c = CircularConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CircularConfig::new("test")
            .circular_type(CircularType::Directive)
            .scope(CircularScope::Department);
        assert_eq!(c.circular_type, CircularType::Directive);
        assert_eq!(c.scope, CircularScope::Department);
    }

    #[test]
    fn test_notice_new() {
        let n = CircularNotice::new("n1", "Title", "Content");
        assert_eq!(n.id, "n1");
    }

    #[test]
    fn test_notice_builder() {
        let n = CircularNotice::new("n1", "Title", "Content")
            .reference("REF-001")
            .effective_date("2025-01-01");
        assert_eq!(n.reference, "REF-001");
        assert_eq!(n.effective_date, "2025-01-01");
    }

    #[test]
    fn test_notice_deactivate() {
        let mut n = CircularNotice::new("n1", "Title", "Content");
        n.deactivate();
        assert!(!n.active);
    }

    #[test]
    fn test_attachment_new() {
        let a = CircularAttachment::new("key", "value", "n1");
        assert_eq!(a.notice_id, "n1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CircularStats::default();
        let notice = CircularNotice::new("n1", "Title", "Content");
        s.update(&[notice], CircularType::Policy);
        assert_eq!(s.total_circulars, 1);
        assert_eq!(s.active, 1);
        assert_eq!(s.policy_count, 1);
    }

    #[test]
    fn test_circular_new() {
        let c = SettingsCircular::new(CircularConfig::default());
        assert_eq!(c.notice_count(), 0);
    }

    #[test]
    fn test_circular_add_notice() {
        let mut c = SettingsCircular::new(CircularConfig::default());
        c.add_notice(CircularNotice::new("n1", "Title", "Content"));
        assert_eq!(c.notice_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CircularRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CircularRegistry::new();
        r.register("c1", SettingsCircular::new(CircularConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_circular_query() {
        assert!(is_circular_query("settings circular"));
        assert!(!is_circular_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = circular_fun_fact();
        assert!(fact.contains("circular"));
    }
}
