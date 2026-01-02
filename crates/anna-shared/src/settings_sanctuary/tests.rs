// v0.0.781: Settings Sanctuary (Phase 357)
// Wildlife sanctuary for settings conservation - Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_sanctuary_type_display() {
        assert_eq!(format!("{}", SanctuaryType::Wildlife), "wildlife");
        assert_eq!(format!("{}", SanctuaryType::Marine), "marine");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", SanctuaryStatus::Protected), "protected");
        assert_eq!(format!("{}", SanctuaryStatus::Expanding), "expanding");
    }

    #[test]
    fn test_config_new() {
        let c = SanctuaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = SanctuaryConfig::new("test")
            .sanctuary_type(SanctuaryType::Bird)
            .status(SanctuaryStatus::Monitored);
        assert_eq!(c.sanctuary_type, SanctuaryType::Bird);
        assert_eq!(c.status, SanctuaryStatus::Monitored);
    }

    #[test]
    fn test_resident_new() {
        let r = SanctuaryResident::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_resident_builder() {
        let r = SanctuaryResident::new("r1", "Title", "Content")
            .habitat(1);
        assert_eq!(r.habitat, 1);
    }

    #[test]
    fn test_resident_thriving() {
        let mut r = SanctuaryResident::new("r1", "Title", "Content");
        r.make_recovering();
        assert!(!r.thriving);
        r.make_thriving();
        assert!(r.thriving);
    }

    #[test]
    fn test_warden_new() {
        let w = SanctuaryWarden::new("key", "name", "r1");
        assert_eq!(w.resident_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = SanctuaryStats::default();
        let resident = SanctuaryResident::new("r1", "Title", "Content");
        s.update(&[resident], SanctuaryType::Wildlife);
        assert_eq!(s.total_residents, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_sanctuary_new() {
        let s = SettingsSanctuary::new(SanctuaryConfig::default());
        assert_eq!(s.resident_count(), 0);
    }

    #[test]
    fn test_sanctuary_add_resident() {
        let mut s = SettingsSanctuary::new(SanctuaryConfig::default());
        s.add_resident(SanctuaryResident::new("r1", "Title", "Content"));
        assert_eq!(s.resident_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SanctuaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SanctuaryRegistry::new();
        r.register("s1", SettingsSanctuary::new(SanctuaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_sanctuary_query() {
        assert!(is_sanctuary_query("settings sanctuary"));
        assert!(!is_sanctuary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = sanctuary_fun_fact();
        assert!(fact.contains("sanctuary"));
    }
}
