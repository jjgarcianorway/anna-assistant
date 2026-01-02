// v0.0.743: Settings Domain - Tests (Phase 319)
// Test suite for all domain components

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_domain_type_display() {
        assert_eq!(format!("{}", DomainType::Public), "public");
        assert_eq!(format!("{}", DomainType::Private), "private");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DomainStatus::Claimed), "claimed");
        assert_eq!(format!("{}", DomainStatus::Consolidated), "consolidated");
    }

    #[test]
    fn test_config_new() {
        let c = DomainConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DomainConfig::new("test")
            .domain_type(DomainType::Royal)
            .status(DomainStatus::Recognized);
        assert_eq!(c.domain_type, DomainType::Royal);
        assert_eq!(c.status, DomainStatus::Recognized);
    }

    #[test]
    fn test_right_new() {
        let r = DomainRight::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_right_builder() {
        let r = DomainRight::new("r1", "Title", "Content")
            .priority(1);
        assert_eq!(r.priority, 1);
    }

    #[test]
    fn test_right_exclusive() {
        let mut r = DomainRight::new("r1", "Title", "Content");
        r.make_shared();
        assert!(!r.exclusive);
        r.make_exclusive();
        assert!(r.exclusive);
    }

    #[test]
    fn test_holder_new() {
        let h = DomainHolder::new("key", "name", "r1");
        assert_eq!(h.right_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DomainStats::default();
        let right = DomainRight::new("r1", "Title", "Content");
        s.update(&[right], DomainType::Public);
        assert_eq!(s.total_rights, 1);
        assert_eq!(s.exclusive, 1);
    }

    #[test]
    fn test_domain_new() {
        let d = SettingsDomain::new(DomainConfig::default());
        assert_eq!(d.right_count(), 0);
    }

    #[test]
    fn test_domain_add_right() {
        let mut d = SettingsDomain::new(DomainConfig::default());
        d.add_right(DomainRight::new("r1", "Title", "Content"));
        assert_eq!(d.right_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DomainRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DomainRegistry::new();
        r.register("d1", SettingsDomain::new(DomainConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_domain_query() {
        assert!(is_domain_query("settings domain"));
        assert!(!is_domain_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = domain_fun_fact();
        assert!(fact.contains("domain"));
    }
}
