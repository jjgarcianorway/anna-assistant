// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Tests module

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_haven_type_display() {
        assert_eq!(format!("{}", HavenType::Safe), "safe");
        assert_eq!(format!("{}", HavenType::Secure), "secure");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HavenStatus::Open), "open");
        assert_eq!(format!("{}", HavenStatus::Welcoming), "welcoming");
    }

    #[test]
    fn test_config_new() {
        let c = HavenConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HavenConfig::new("test")
            .haven_type(HavenType::Secure)
            .status(HavenStatus::Guarding);
        assert_eq!(c.haven_type, HavenType::Secure);
        assert_eq!(c.status, HavenStatus::Guarding);
    }

    #[test]
    fn test_guest_new() {
        let g = HavenGuest::new("g1", "Title", "Content");
        assert_eq!(g.id, "g1");
    }

    #[test]
    fn test_guest_builder() {
        let g = HavenGuest::new("g1", "Title", "Content")
            .room(1);
        assert_eq!(g.room, 1);
    }

    #[test]
    fn test_guest_comfort() {
        let mut g = HavenGuest::new("g1", "Title", "Content");
        g.make_restless();
        assert!(!g.comfortable);
        g.make_comfortable();
        assert!(g.comfortable);
    }

    #[test]
    fn test_keeper_new() {
        let k = HavenKeeper::new("key", "name", "g1");
        assert_eq!(k.guest_id, "g1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HavenStats::default();
        let guest = HavenGuest::new("g1", "Title", "Content");
        s.update(&[guest], HavenType::Safe);
        assert_eq!(s.total_guests, 1);
        assert_eq!(s.comfortable, 1);
    }

    #[test]
    fn test_haven_new() {
        let h = SettingsHaven::new(HavenConfig::default());
        assert_eq!(h.guest_count(), 0);
    }

    #[test]
    fn test_haven_add_guest() {
        let mut h = SettingsHaven::new(HavenConfig::default());
        h.add_guest(HavenGuest::new("g1", "Title", "Content"));
        assert_eq!(h.guest_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HavenRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HavenRegistry::new();
        r.register("h1", SettingsHaven::new(HavenConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_haven_query() {
        assert!(is_haven_query("settings haven"));
        assert!(!is_haven_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = haven_fun_fact();
        assert!(fact.contains("haven"));
    }
}
