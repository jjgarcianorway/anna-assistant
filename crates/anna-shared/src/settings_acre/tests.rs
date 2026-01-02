// v0.0.760: Settings Acre Tests
// Unit tests for settings acre

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_acre_type_display() {
        assert_eq!(format!("{}", AcreType::Survey), "survey");
        assert_eq!(format!("{}", AcreType::Statute), "statute");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AcreStatus::Measured), "measured");
        assert_eq!(format!("{}", AcreStatus::Certified), "certified");
    }

    #[test]
    fn test_config_new() {
        let c = AcreConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AcreConfig::new("test")
            .acre_type(AcreType::Irish)
            .status(AcreStatus::Disputed);
        assert_eq!(c.acre_type, AcreType::Irish);
        assert_eq!(c.status, AcreStatus::Disputed);
    }

    #[test]
    fn test_measurement_new() {
        let m = AcreMeasurement::new("m1", "Title", "Content");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_measurement_builder() {
        let m = AcreMeasurement::new("m1", "Title", "Content")
            .chain(1);
        assert_eq!(m.chain, 1);
    }

    #[test]
    fn test_measurement_certified() {
        let mut m = AcreMeasurement::new("m1", "Title", "Content");
        m.make_uncertified();
        assert!(!m.certified);
        m.make_certified();
        assert!(m.certified);
    }

    #[test]
    fn test_surveyor_new() {
        let s = AcreSurveyor::new("key", "name", "m1");
        assert_eq!(s.measurement_id, "m1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AcreStats::default();
        let measurement = AcreMeasurement::new("m1", "Title", "Content");
        s.update(&[measurement], AcreType::Survey);
        assert_eq!(s.total_measurements, 1);
        assert_eq!(s.certified, 1);
    }

    #[test]
    fn test_acre_new() {
        let a = SettingsAcre::new(AcreConfig::default());
        assert_eq!(a.measurement_count(), 0);
    }

    #[test]
    fn test_acre_add_measurement() {
        let mut a = SettingsAcre::new(AcreConfig::default());
        a.add_measurement(AcreMeasurement::new("m1", "Title", "Content"));
        assert_eq!(a.measurement_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AcreRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AcreRegistry::new();
        r.register("a1", SettingsAcre::new(AcreConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_acre_query() {
        assert!(is_acre_query("settings acre"));
        assert!(!is_acre_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = acre_fun_fact();
        assert!(fact.contains("acre"));
    }
}
