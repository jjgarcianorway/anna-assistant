// v0.0.778: Settings Aviary (Phase 354)
// Tests for settings aviary

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_aviary_type_display() {
        assert_eq!(format!("{}", AviaryType::Flight), "flight");
        assert_eq!(format!("{}", AviaryType::Breeding), "breeding");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AviaryStatus::Active), "active");
        assert_eq!(format!("{}", AviaryStatus::Nesting), "nesting");
    }

    #[test]
    fn test_config_new() {
        let c = AviaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AviaryConfig::new("test")
            .aviary_type(AviaryType::Display)
            .status(AviaryStatus::Molting);
        assert_eq!(c.aviary_type, AviaryType::Display);
        assert_eq!(c.status, AviaryStatus::Molting);
    }

    #[test]
    fn test_bird_new() {
        let b = AviaryBird::new("b1", "Title", "Content");
        assert_eq!(b.id, "b1");
    }

    #[test]
    fn test_bird_builder() {
        let b = AviaryBird::new("b1", "Title", "Content")
            .perch(1);
        assert_eq!(b.perch, 1);
    }

    #[test]
    fn test_bird_flying() {
        let mut b = AviaryBird::new("b1", "Title", "Content");
        b.make_grounded();
        assert!(!b.flying);
        b.make_flying();
        assert!(b.flying);
    }

    #[test]
    fn test_keeper_new() {
        let k = AviaryKeeper::new("key", "name", "b1");
        assert_eq!(k.bird_id, "b1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AviaryStats::default();
        let bird = AviaryBird::new("b1", "Title", "Content");
        s.update(&[bird], AviaryType::Flight);
        assert_eq!(s.total_birds, 1);
        assert_eq!(s.flying, 1);
    }

    #[test]
    fn test_aviary_new() {
        let a = SettingsAviary::new(AviaryConfig::default());
        assert_eq!(a.bird_count(), 0);
    }

    #[test]
    fn test_aviary_add_bird() {
        let mut a = SettingsAviary::new(AviaryConfig::default());
        a.add_bird(AviaryBird::new("b1", "Title", "Content"));
        assert_eq!(a.bird_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AviaryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AviaryRegistry::new();
        r.register("a1", SettingsAviary::new(AviaryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_aviary_query() {
        assert!(is_aviary_query("settings aviary"));
        assert!(!is_aviary_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = aviary_fun_fact();
        assert!(fact.contains("aviary"));
    }
}
