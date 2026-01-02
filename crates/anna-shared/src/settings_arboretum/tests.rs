// v0.0.772: Settings Arboretum Tests (Phase 348)
// Test suite for arboretum functionality

#[cfg(test)]
mod tests {
    use crate::settings_arboretum::*;

    #[test]
    fn test_arboretum_type_display() {
        assert_eq!(format!("{}", ArboretumType::Public), "public");
        assert_eq!(format!("{}", ArboretumType::University), "university");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ArboretumStatus::Open), "open");
        assert_eq!(format!("{}", ArboretumStatus::Closed), "closed");
    }

    #[test]
    fn test_config_new() {
        let c = ArboretumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ArboretumConfig::new("test")
            .arboretum_type(ArboretumType::Memorial)
            .status(ArboretumStatus::Planting);
        assert_eq!(c.arboretum_type, ArboretumType::Memorial);
        assert_eq!(c.status, ArboretumStatus::Planting);
    }

    #[test]
    fn test_specimen_new() {
        let s = ArboretumSpecimen::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_specimen_builder() {
        let s = ArboretumSpecimen::new("s1", "Title", "Content")
            .plot(1);
        assert_eq!(s.plot, 1);
    }

    #[test]
    fn test_specimen_cataloged() {
        let mut s = ArboretumSpecimen::new("s1", "Title", "Content");
        s.make_uncataloged();
        assert!(!s.cataloged);
        s.make_cataloged();
        assert!(s.cataloged);
    }

    #[test]
    fn test_dendrologist_new() {
        let d = ArboretumDendrologist::new("key", "name", "s1");
        assert_eq!(d.specimen_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ArboretumStats::default();
        let specimen = ArboretumSpecimen::new("s1", "Title", "Content");
        s.update(&[specimen], ArboretumType::Public);
        assert_eq!(s.total_specimens, 1);
        assert_eq!(s.cataloged, 1);
    }

    #[test]
    fn test_arboretum_new() {
        let a = SettingsArboretum::new(ArboretumConfig::default());
        assert_eq!(a.specimen_count(), 0);
    }

    #[test]
    fn test_arboretum_add_specimen() {
        let mut a = SettingsArboretum::new(ArboretumConfig::default());
        a.add_specimen(ArboretumSpecimen::new("s1", "Title", "Content"));
        assert_eq!(a.specimen_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ArboretumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ArboretumRegistry::new();
        r.register("a1", SettingsArboretum::new(ArboretumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_arboretum_query() {
        assert!(is_arboretum_query("settings arboretum"));
        assert!(!is_arboretum_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = arboretum_fun_fact();
        assert!(fact.contains("arboretum"));
    }
}
