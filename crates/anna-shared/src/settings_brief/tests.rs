// v0.0.710: Settings Brief - Tests (Phase 286)
// Unit tests for settings brief module

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_brief_type_display() {
        assert_eq!(format!("{}", BriefType::Executive), "executive");
        assert_eq!(format!("{}", BriefType::Technical), "technical");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", BriefScope::Department), "department");
        assert_eq!(format!("{}", BriefScope::Organization), "organization");
    }

    #[test]
    fn test_config_new() {
        let c = BriefConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BriefConfig::new("test")
            .brief_type(BriefType::Strategic)
            .scope(BriefScope::Organization);
        assert_eq!(c.brief_type, BriefType::Strategic);
        assert_eq!(c.scope, BriefScope::Organization);
    }

    #[test]
    fn test_point_new() {
        let p = BriefPoint::new("p1", "Point 1");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_point_builder() {
        let p = BriefPoint::new("p1", "Point 1")
            .priority(3)
            .action_required(true);
        assert_eq!(p.priority, 3);
        assert!(p.action_required);
    }

    #[test]
    fn test_attachment_new() {
        let a = BriefAttachment::new("key", "value", "p1");
        assert_eq!(a.point_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BriefStats::default();
        let point = BriefPoint::new("p1", "Point").action_required(true).priority(3);
        s.update(&[point], BriefType::Executive);
        assert_eq!(s.total_points, 1);
        assert_eq!(s.action_items, 1);
        assert_eq!(s.high_priority, 1);
    }

    #[test]
    fn test_brief_new() {
        let b = SettingsBrief::new(BriefConfig::default());
        assert_eq!(b.point_count(), 0);
    }

    #[test]
    fn test_brief_add_point() {
        let mut b = SettingsBrief::new(BriefConfig::default());
        b.add_point(BriefPoint::new("p1", "Point 1"));
        assert_eq!(b.point_count(), 1);
    }

    #[test]
    fn test_brief_add_attachment() {
        let mut b = SettingsBrief::new(BriefConfig::default());
        b.add_attachment(BriefAttachment::new("key", "value", "p1"));
        assert_eq!(b.attachment_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BriefRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BriefRegistry::new();
        r.register("b1", SettingsBrief::new(BriefConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_brief_query() {
        assert!(is_brief_query("settings brief"));
        assert!(!is_brief_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = brief_fun_fact();
        assert!(fact.contains("brief"));
    }
}
