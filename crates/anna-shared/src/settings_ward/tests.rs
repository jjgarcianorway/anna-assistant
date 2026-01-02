// v0.0.752: Settings Ward Tests (Phase 328)
// Test module for ward system

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_ward_type_display() {
        assert_eq!(format!("{}", WardType::Electoral), "electoral");
        assert_eq!(format!("{}", WardType::Hospital), "hospital");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", WardStatus::Created), "created");
        assert_eq!(format!("{}", WardStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = WardConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = WardConfig::new("test")
            .ward_type(WardType::Administrative)
            .status(WardStatus::Redrawn);
        assert_eq!(c.ward_type, WardType::Administrative);
        assert_eq!(c.status, WardStatus::Redrawn);
    }

    #[test]
    fn test_motion_new() {
        let m = WardMotion::new("m1", "Title", "Content");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_motion_builder() {
        let m = WardMotion::new("m1", "Title", "Content")
            .precinct(1);
        assert_eq!(m.precinct, 1);
    }

    #[test]
    fn test_motion_passed() {
        let mut m = WardMotion::new("m1", "Title", "Content");
        m.make_failed();
        assert!(!m.passed);
        m.make_passed();
        assert!(m.passed);
    }

    #[test]
    fn test_delegate_new() {
        let d = WardDelegate::new("key", "name", "m1");
        assert_eq!(d.motion_id, "m1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = WardStats::default();
        let motion = WardMotion::new("m1", "Title", "Content");
        s.update(&[motion], WardType::Electoral);
        assert_eq!(s.total_motions, 1);
        assert_eq!(s.passed, 1);
    }

    #[test]
    fn test_ward_new() {
        let w = SettingsWard::new(WardConfig::default());
        assert_eq!(w.motion_count(), 0);
    }

    #[test]
    fn test_ward_add_motion() {
        let mut w = SettingsWard::new(WardConfig::default());
        w.add_motion(WardMotion::new("m1", "Title", "Content"));
        assert_eq!(w.motion_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = WardRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = WardRegistry::new();
        r.register("w1", SettingsWard::new(WardConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_ward_query() {
        assert!(is_ward_query("settings ward"));
        assert!(!is_ward_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = ward_fun_fact();
        assert!(fact.contains("ward"));
    }
}
