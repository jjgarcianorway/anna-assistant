// v0.0.701: Settings Anthology (Phase 277)
// Tests for anthology module

#[cfg(test)]
mod tests {
    use crate::settings_anthology::*;

    #[test]
    fn test_anthology_type_display() {
        assert_eq!(format!("{}", AnthologyType::BestOf), "best_of");
        assert_eq!(format!("{}", AnthologyType::Complete), "complete");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AnthologyStatus::Curating), "curating");
        assert_eq!(format!("{}", AnthologyStatus::Published), "published");
    }

    #[test]
    fn test_config_new() {
        let c = AnthologyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AnthologyConfig::new("test")
            .anthology_type(AnthologyType::Themed)
            .theme("Linux configs");
        assert_eq!(c.anthology_type, AnthologyType::Themed);
        assert_eq!(c.theme, "Linux configs");
    }

    #[test]
    fn test_work_new() {
        let w = AnthologyWork::new("w1", "Work 1", "Author");
        assert_eq!(w.id, "w1");
    }

    #[test]
    fn test_work_builder() {
        let w = AnthologyWork::new("w1", "Work 1", "Author")
            .source("config.toml")
            .featured(true);
        assert_eq!(w.source, "config.toml");
        assert!(w.featured);
    }

    #[test]
    fn test_piece_new() {
        let p = AnthologyPiece::new("key", "value", "w1", 1);
        assert_eq!(p.work_id, "w1");
        assert_eq!(p.order, 1);
    }

    #[test]
    fn test_stats_update() {
        let mut s = AnthologyStats::default();
        let works = vec![AnthologyWork::new("w1", "Work", "Author").featured(true)];
        s.update(&works);
        assert_eq!(s.total_works, 1);
        assert_eq!(s.featured_works, 1);
    }

    #[test]
    fn test_anthology_new() {
        let a = SettingsAnthology::new(AnthologyConfig::default());
        assert_eq!(a.work_count(), 0);
    }

    #[test]
    fn test_anthology_add_work() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.add_work(AnthologyWork::new("w1", "Work 1", "Author"));
        assert_eq!(a.work_count(), 1);
    }

    #[test]
    fn test_anthology_add_piece() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.add_piece(AnthologyPiece::new("key", "value", "w1", 1));
        assert_eq!(a.stats().total_pieces, 1);
    }

    #[test]
    fn test_anthology_complete() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.complete();
        assert_eq!(a.status(), AnthologyStatus::Complete);
    }

    #[test]
    fn test_anthology_publish() {
        let mut a = SettingsAnthology::new(AnthologyConfig::default());
        a.publish();
        assert_eq!(a.status(), AnthologyStatus::Published);
    }

    #[test]
    fn test_registry_new() {
        let r = AnthologyRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AnthologyRegistry::new();
        r.register("a1", SettingsAnthology::new(AnthologyConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_anthology_query() {
        assert!(is_anthology_query("settings anthology"));
        assert!(!is_anthology_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = anthology_fun_fact();
        assert!(fact.contains("anthology"));
    }
}
