// v0.0.750: Settings Municipality Tests (Phase 326)
// Unit tests for municipality module

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_municipality_type_display() {
        assert_eq!(format!("{}", MunicipalityType::City), "city");
        assert_eq!(format!("{}", MunicipalityType::Town), "town");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", MunicipalityStatus::Incorporated), "incorporated");
        assert_eq!(format!("{}", MunicipalityStatus::Chartered), "chartered");
    }

    #[test]
    fn test_config_new() {
        let c = MunicipalityConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MunicipalityConfig::new("test")
            .municipality_type(MunicipalityType::Village)
            .status(MunicipalityStatus::Chartered);
        assert_eq!(c.municipality_type, MunicipalityType::Village);
        assert_eq!(c.status, MunicipalityStatus::Chartered);
    }

    #[test]
    fn test_code_new() {
        let c = MunicipalityCode::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_code_builder() {
        let c = MunicipalityCode::new("c1", "Title", "Content")
            .chapter(1);
        assert_eq!(c.chapter, 1);
    }

    #[test]
    fn test_code_in_force() {
        let mut c = MunicipalityCode::new("c1", "Title", "Content");
        c.make_suspended();
        assert!(!c.in_force);
        c.make_in_force();
        assert!(c.in_force);
    }

    #[test]
    fn test_councilor_new() {
        let c = MunicipalityCouncilor::new("key", "name", "c1");
        assert_eq!(c.code_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MunicipalityStats::default();
        let code = MunicipalityCode::new("c1", "Title", "Content");
        s.update(&[code], MunicipalityType::City);
        assert_eq!(s.total_codes, 1);
        assert_eq!(s.in_force, 1);
    }

    #[test]
    fn test_municipality_new() {
        let m = SettingsMunicipality::new(MunicipalityConfig::default());
        assert_eq!(m.code_count(), 0);
    }

    #[test]
    fn test_municipality_add_code() {
        let mut m = SettingsMunicipality::new(MunicipalityConfig::default());
        m.add_code(MunicipalityCode::new("c1", "Title", "Content"));
        assert_eq!(m.code_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MunicipalityRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MunicipalityRegistry::new();
        r.register("m1", SettingsMunicipality::new(MunicipalityConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_municipality_query() {
        assert!(is_municipality_query("settings municipality"));
        assert!(!is_municipality_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = municipality_fun_fact();
        assert!(fact.contains("municipality"));
    }
}
