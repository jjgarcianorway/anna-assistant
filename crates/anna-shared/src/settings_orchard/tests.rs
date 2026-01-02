// v0.0.766: Settings Orchard Tests
// Test suite for settings orchard module

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_orchard_type_display() {
        assert_eq!(format!("{}", OrchardType::Apple), "apple");
        assert_eq!(format!("{}", OrchardType::Cherry), "cherry");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", OrchardStatus::Dormant), "dormant");
        assert_eq!(format!("{}", OrchardStatus::Blooming), "blooming");
    }

    #[test]
    fn test_config_new() {
        let c = OrchardConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = OrchardConfig::new("test")
            .orchard_type(OrchardType::Peach)
            .status(OrchardStatus::Fruiting);
        assert_eq!(c.orchard_type, OrchardType::Peach);
        assert_eq!(c.status, OrchardStatus::Fruiting);
    }

    #[test]
    fn test_fruit_new() {
        let f = OrchardFruit::new("f1", "Title", "Content");
        assert_eq!(f.id, "f1");
    }

    #[test]
    fn test_fruit_builder() {
        let f = OrchardFruit::new("f1", "Title", "Content")
            .branch(1);
        assert_eq!(f.branch, 1);
    }

    #[test]
    fn test_fruit_ripe() {
        let mut f = OrchardFruit::new("f1", "Title", "Content");
        f.make_unripe();
        assert!(!f.ripe);
        f.make_ripe();
        assert!(f.ripe);
    }

    #[test]
    fn test_picker_new() {
        let p = OrchardPicker::new("key", "name", "f1");
        assert_eq!(p.fruit_id, "f1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = OrchardStats::default();
        let fruit = OrchardFruit::new("f1", "Title", "Content");
        s.update(&[fruit], OrchardType::Apple);
        assert_eq!(s.total_fruits, 1);
        assert_eq!(s.ripe, 1);
    }

    #[test]
    fn test_orchard_new() {
        let o = SettingsOrchard::new(OrchardConfig::default());
        assert_eq!(o.fruit_count(), 0);
    }

    #[test]
    fn test_orchard_add_fruit() {
        let mut o = SettingsOrchard::new(OrchardConfig::default());
        o.add_fruit(OrchardFruit::new("f1", "Title", "Content"));
        assert_eq!(o.fruit_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = OrchardRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = OrchardRegistry::new();
        r.register("o1", SettingsOrchard::new(OrchardConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_orchard_query() {
        assert!(is_orchard_query("settings orchard"));
        assert!(!is_orchard_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = orchard_fun_fact();
        assert!(fact.contains("orchard"));
    }
}
