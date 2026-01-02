// v0.0.740: Settings Bloc Tests (Phase 316)
// Test suite for bloc system

#[cfg(test)]
mod tests {
    use crate::settings_bloc::*;

    #[test]
    fn test_bloc_type_display() {
        assert_eq!(format!("{}", BlocType::Trading), "trading");
        assert_eq!(format!("{}", BlocType::Voting), "voting");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BlocStatus::Forming), "forming");
        assert_eq!(format!("{}", BlocStatus::Dominant), "dominant");
    }

    #[test]
    fn test_config_new() {
        let c = BlocConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BlocConfig::new("test")
            .bloc_type(BlocType::Power)
            .status(BlocStatus::Active);
        assert_eq!(c.bloc_type, BlocType::Power);
        assert_eq!(c.status, BlocStatus::Active);
    }

    #[test]
    fn test_policy_new() {
        let p = BlocPolicy::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_policy_builder() {
        let p = BlocPolicy::new("p1", "Title", "Content")
            .priority(1);
        assert_eq!(p.priority, 1);
    }

    #[test]
    fn test_policy_coordinated() {
        let mut p = BlocPolicy::new("p1", "Title", "Content");
        p.make_independent();
        assert!(!p.coordinated);
        p.make_coordinated();
        assert!(p.coordinated);
    }

    #[test]
    fn test_member_new() {
        let m = BlocMember::new("key", "name", "p1");
        assert_eq!(m.policy_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BlocStats::default();
        let policy = BlocPolicy::new("p1", "Title", "Content");
        s.update(&[policy], BlocType::Trading);
        assert_eq!(s.total_policies, 1);
        assert_eq!(s.coordinated, 1);
    }

    #[test]
    fn test_bloc_new() {
        let b = SettingsBloc::new(BlocConfig::default());
        assert_eq!(b.policy_count(), 0);
    }

    #[test]
    fn test_bloc_add_policy() {
        let mut b = SettingsBloc::new(BlocConfig::default());
        b.add_policy(BlocPolicy::new("p1", "Title", "Content"));
        assert_eq!(b.policy_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BlocRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BlocRegistry::new();
        r.register("b1", SettingsBloc::new(BlocConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_bloc_query() {
        assert!(is_bloc_query("settings bloc"));
        assert!(!is_bloc_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = bloc_fun_fact();
        assert!(fact.contains("bloc"));
    }
}
