// v0.0.761: Settings Hectare (Phase 337)
// Unit tests

#[cfg(test)]
mod tests {
    use crate::settings_hectare::*;

    #[test]
    fn test_hectare_type_display() {
        assert_eq!(format!("{}", HectareType::Standard), "standard");
        assert_eq!(format!("{}", HectareType::Cadastral), "cadastral");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HectareStatus::Surveyed), "surveyed");
        assert_eq!(format!("{}", HectareStatus::Confirmed), "confirmed");
    }

    #[test]
    fn test_config_new() {
        let c = HectareConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HectareConfig::new("test")
            .hectare_type(HectareType::Forest)
            .status(HectareStatus::Contested);
        assert_eq!(c.hectare_type, HectareType::Forest);
        assert_eq!(c.status, HectareStatus::Contested);
    }

    #[test]
    fn test_record_new() {
        let r = HectareRecord::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_record_builder() {
        let r = HectareRecord::new("r1", "Title", "Content")
            .grid(1);
        assert_eq!(r.grid, 1);
    }

    #[test]
    fn test_record_confirmed() {
        let mut r = HectareRecord::new("r1", "Title", "Content");
        r.make_unconfirmed();
        assert!(!r.confirmed);
        r.make_confirmed();
        assert!(r.confirmed);
    }

    #[test]
    fn test_inspector_new() {
        let i = HectareInspector::new("key", "name", "r1");
        assert_eq!(i.record_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HectareStats::default();
        let record = HectareRecord::new("r1", "Title", "Content");
        s.update(&[record], HectareType::Standard);
        assert_eq!(s.total_records, 1);
        assert_eq!(s.confirmed, 1);
    }

    #[test]
    fn test_hectare_new() {
        let h = SettingsHectare::new(HectareConfig::default());
        assert_eq!(h.record_count(), 0);
    }

    #[test]
    fn test_hectare_add_record() {
        let mut h = SettingsHectare::new(HectareConfig::default());
        h.add_record(HectareRecord::new("r1", "Title", "Content"));
        assert_eq!(h.record_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HectareRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HectareRegistry::new();
        r.register("h1", SettingsHectare::new(HectareConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_hectare_query() {
        assert!(is_hectare_query("settings hectare"));
        assert!(!is_hectare_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = hectare_fun_fact();
        assert!(fact.contains("hectare"));
    }
}
