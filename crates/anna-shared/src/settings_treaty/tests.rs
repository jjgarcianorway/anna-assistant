// v0.0.727: Settings Treaty (Phase 303)
// International agreement for settings governance - Tests

#[cfg(test)]
mod tests {
    use crate::settings_treaty::*;

    #[test]
    fn test_treaty_type_display() {
        assert_eq!(format!("{}", TreatyType::Bilateral), "bilateral");
        assert_eq!(format!("{}", TreatyType::Multilateral), "multilateral");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TreatyStatus::Negotiating), "negotiating");
        assert_eq!(format!("{}", TreatyStatus::Ratified), "ratified");
    }

    #[test]
    fn test_config_new() {
        let c = TreatyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TreatyConfig::new("test")
            .treaty_type(TreatyType::Multilateral)
            .status(TreatyStatus::Signed);
        assert_eq!(c.treaty_type, TreatyType::Multilateral);
        assert_eq!(c.status, TreatyStatus::Signed);
    }

    #[test]
    fn test_provision_new() {
        let p = TreatyProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = TreatyProvision::new("p1", "Title", "Content")
            .article(1);
        assert_eq!(p.article, 1);
    }

    #[test]
    fn test_provision_force_terminate() {
        let mut p = TreatyProvision::new("p1", "Title", "Content");
        p.enter_force();
        assert!(p.in_force);
        p.terminate();
        assert!(!p.in_force);
    }

    #[test]
    fn test_signatory_new() {
        let s = TreatySignatory::new("key", "name", "p1");
        assert_eq!(s.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = TreatyStats::default();
        let mut provision = TreatyProvision::new("p1", "Title", "Content");
        provision.enter_force();
        s.update(&[provision], TreatyType::Bilateral);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.in_force, 1);
    }

    #[test]
    fn test_treaty_new() {
        let t = SettingsTreaty::new(TreatyConfig::default());
        assert_eq!(t.provision_count(), 0);
    }

    #[test]
    fn test_treaty_add_provision() {
        let mut t = SettingsTreaty::new(TreatyConfig::default());
        t.add_provision(TreatyProvision::new("p1", "Title", "Content"));
        assert_eq!(t.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = TreatyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TreatyRegistry::new();
        r.register("t1", SettingsTreaty::new(TreatyConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_treaty_query() {
        assert!(is_treaty_query("settings treaty"));
        assert!(!is_treaty_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = treaty_fun_fact();
        assert!(fact.contains("treaty"));
    }
}
