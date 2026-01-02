// v0.0.730: Settings Accord (Phase 306)
// Tests for settings accord

#[cfg(test)]
mod tests {
    use crate::settings_accord::*;

    #[test]
    fn test_accord_type_display() {
        assert_eq!(format!("{}", AccordType::Peace), "peace");
        assert_eq!(format!("{}", AccordType::Trade), "trade");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AccordStatus::Preliminary), "preliminary");
        assert_eq!(format!("{}", AccordStatus::Implemented), "implemented");
    }

    #[test]
    fn test_config_new() {
        let c = AccordConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AccordConfig::new("test")
            .accord_type(AccordType::Trade)
            .status(AccordStatus::Final);
        assert_eq!(c.accord_type, AccordType::Trade);
        assert_eq!(c.status, AccordStatus::Final);
    }

    #[test]
    fn test_provision_new() {
        let p = AccordProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = AccordProvision::new("p1", "Title", "Content")
            .section(1);
        assert_eq!(p.section, 1);
    }

    #[test]
    fn test_provision_agree_disagree() {
        let mut p = AccordProvision::new("p1", "Title", "Content");
        p.agree();
        assert!(p.agreed);
        p.disagree();
        assert!(!p.agreed);
    }

    #[test]
    fn test_signatory_new() {
        let s = AccordSignatory::new("key", "name", "p1");
        assert_eq!(s.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AccordStats::default();
        let mut provision = AccordProvision::new("p1", "Title", "Content");
        provision.agree();
        s.update(&[provision], AccordType::Peace);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.agreed, 1);
    }

    #[test]
    fn test_accord_new() {
        let a = SettingsAccord::new(AccordConfig::default());
        assert_eq!(a.provision_count(), 0);
    }

    #[test]
    fn test_accord_add_provision() {
        let mut a = SettingsAccord::new(AccordConfig::default());
        a.add_provision(AccordProvision::new("p1", "Title", "Content"));
        assert_eq!(a.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AccordRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AccordRegistry::new();
        r.register("a1", SettingsAccord::new(AccordConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_accord_query() {
        assert!(is_accord_query("settings accord"));
        assert!(!is_accord_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = accord_fun_fact();
        assert!(fact.contains("accord"));
    }
}
