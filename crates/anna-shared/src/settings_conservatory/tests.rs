// v0.0.771: Conservatory Tests
// Unit tests for all conservatory components

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_conservatory_type_display() {
        assert_eq!(format!("{}", ConservatoryType::Victorian), "victorian");
        assert_eq!(format!("{}", ConservatoryType::Modern), "modern");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ConservatoryStatus::Open), "open");
        assert_eq!(format!("{}", ConservatoryStatus::Renovation), "renovation");
    }

    #[test]
    fn test_config_new() {
        let c = ConservatoryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConservatoryConfig::new("test")
            .conservatory_type(ConservatoryType::Edwardian)
            .status(ConservatoryStatus::Ventilating);
        assert_eq!(c.conservatory_type, ConservatoryType::Edwardian);
        assert_eq!(c.status, ConservatoryStatus::Ventilating);
    }

    #[test]
    fn test_specimen_new() {
        let s = ConservatorySpecimen::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_specimen_builder() {
        let s = ConservatorySpecimen::new("s1", "Title", "Content")
            .section(1);
        assert_eq!(s.section, 1);
    }

    #[test]
    fn test_specimen_preserved() {
        let mut s = ConservatorySpecimen::new("s1", "Title", "Content");
        s.make_damaged();
        assert!(!s.preserved);
        s.make_preserved();
        assert!(s.preserved);
    }

    #[test]
    fn test_curator_new() {
        let c = ConservatoryCurator::new("key", "name", "s1");
        assert_eq!(c.specimen_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ConservatoryStats::default();
        let specimen = ConservatorySpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], ConservatoryType::Victorian);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.preserved, 1);
    }

    #[test]
    fn test_conservatory_new() {
        let c = SettingsConservatory::new(ConservatoryConfig::default());
        assert_eq!(c.specimen_count(), 0);
    }

    #[test]
    fn test_conservatory_add_specimen() {
        let mut c = SettingsConservatory::new(ConservatoryConfig::default());
        c.add_specimen(ConservatorySpecimen::new("s1", "Title", "Content"));
        assert_eq!(c.specimen_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ConservatoryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConservatoryRegistry::new();
        r.register("c1", SettingsConservatory::new(ConservatoryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_conservatory_query() {
        assert!(is_conservatory_query("settings conservatory"));
        assert!(!is_conservatory_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = conservatory_fun_fact();
        assert!(fact.contains("conservatory"));
    }
}
