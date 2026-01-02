// v0.0.722: Ordinance Tests (Phase 298)
// Test cases for settings ordinance

#[cfg(test)]
mod tests {
    use crate::settings_ordinance::*;

    #[test]
    fn test_ordinance_type_display() {
        assert_eq!(format!("{}", OrdinanceType::Municipal), "municipal");
        assert_eq!(format!("{}", OrdinanceType::Zoning), "zoning");
    }

    #[test]
    fn test_jurisdiction_display() {
        assert_eq!(format!("{}", OrdinanceJurisdiction::City), "city");
        assert_eq!(format!("{}", OrdinanceJurisdiction::District), "district");
    }

    #[test]
    fn test_config_new() {
        let c = OrdinanceConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = OrdinanceConfig::new("test")
            .ordinance_type(OrdinanceType::Regional)
            .jurisdiction(OrdinanceJurisdiction::County);
        assert_eq!(c.ordinance_type, OrdinanceType::Regional);
        assert_eq!(c.jurisdiction, OrdinanceJurisdiction::County);
    }

    #[test]
    fn test_provision_new() {
        let p = OrdinanceProvision::new("p1", "Title", "Text");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = OrdinanceProvision::new("p1", "Title", "Text")
            .section("1.1");
        assert_eq!(p.section, "1.1");
    }

    #[test]
    fn test_provision_effective() {
        let mut p = OrdinanceProvision::new("p1", "Title", "Text");
        p.make_effective();
        assert!(p.effective);
        p.make_ineffective();
        assert!(!p.effective);
    }

    #[test]
    fn test_amendment_new() {
        let a = OrdinanceAmendment::new("key", "value", "p1");
        assert_eq!(a.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = OrdinanceStats::default();
        let mut prov = OrdinanceProvision::new("p1", "Title", "Text");
        prov.make_effective();
        s.update(&[prov], OrdinanceType::Municipal);
        assert_eq!(s.total_ordinances, 1);
        assert_eq!(s.effective, 1);
        assert_eq!(s.municipal_count, 1);
    }

    #[test]
    fn test_ordinance_new() {
        let o = SettingsOrdinance::new(OrdinanceConfig::default());
        assert_eq!(o.provision_count(), 0);
    }

    #[test]
    fn test_ordinance_add_provision() {
        let mut o = SettingsOrdinance::new(OrdinanceConfig::default());
        o.add_provision(OrdinanceProvision::new("p1", "Title", "Text"));
        assert_eq!(o.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = OrdinanceRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = OrdinanceRegistry::new();
        r.register("o1", SettingsOrdinance::new(OrdinanceConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_ordinance_query() {
        assert!(is_ordinance_query("settings ordinance"));
        assert!(!is_ordinance_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = ordinance_fun_fact();
        assert!(fact.contains("ordinance"));
    }
}
