// v0.0.774: Settings Herbarium - Tests
// Unit tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_herbarium_type_display() {
        assert_eq!(format!("{}", HerbariumType::University), "university");
        assert_eq!(format!("{}", HerbariumType::Museum), "museum");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HerbariumStatus::Active), "active");
        assert_eq!(format!("{}", HerbariumStatus::Archiving), "archiving");
    }

    #[test]
    fn test_config_new() {
        let c = HerbariumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HerbariumConfig::new("test")
            .herbarium_type(HerbariumType::National)
            .status(HerbariumStatus::Digitizing);
        assert_eq!(c.herbarium_type, HerbariumType::National);
        assert_eq!(c.status, HerbariumStatus::Digitizing);
    }

    #[test]
    fn test_specimen_new() {
        let s = HerbariumSpecimen::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_specimen_builder() {
        let s = HerbariumSpecimen::new("s1", "Title", "Content")
            .cabinet(1);
        assert_eq!(s.cabinet, 1);
    }

    #[test]
    fn test_specimen_mounted() {
        let mut s = HerbariumSpecimen::new("s1", "Title", "Content");
        s.make_unmounted();
        assert!(!s.mounted);
        s.make_mounted();
        assert!(s.mounted);
    }

    #[test]
    fn test_taxonomist_new() {
        let t = HerbariumTaxonomist::new("key", "name", "s1");
        assert_eq!(t.specimen_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HerbariumStats::default();
        let specimen = HerbariumSpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], HerbariumType::University);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.mounted, 1);
    }

    #[test]
    fn test_herbarium_new() {
        let h = SettingsHerbarium::new(HerbariumConfig::default());
        assert_eq!(h.specimen_count(), 0);
    }

    #[test]
    fn test_herbarium_add_specimen() {
        let mut h = SettingsHerbarium::new(HerbariumConfig::default());
        h.add_specimen(HerbariumSpecimen::new("s1", "Title", "Content"));
        assert_eq!(h.specimen_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HerbariumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HerbariumRegistry::new();
        r.register("h1", SettingsHerbarium::new(HerbariumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_herbarium_query() {
        assert!(is_herbarium_query("settings herbarium"));
        assert!(!is_herbarium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = herbarium_fun_fact();
        assert!(fact.contains("herbarium"));
    }
}
