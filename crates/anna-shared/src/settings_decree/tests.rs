// v0.0.720: Settings Decree - Tests (Phase 296)
// Test module

#[cfg(test)]
mod tests {
    use crate::settings_decree::*;

    #[test]
    fn test_decree_type_display() {
        assert_eq!(format!("{}", DecreeType::Executive), "executive");
        assert_eq!(format!("{}", DecreeType::Emergency), "emergency");
    }

    #[test]
    fn test_binding_display() {
        assert_eq!(format!("{}", DecreeBinding::Mandatory), "mandatory");
        assert_eq!(format!("{}", DecreeBinding::Advisory), "advisory");
    }

    #[test]
    fn test_config_new() {
        let c = DecreeConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DecreeConfig::new("test")
            .decree_type(DecreeType::Legislative)
            .binding(DecreeBinding::Recommended);
        assert_eq!(c.decree_type, DecreeType::Legislative);
        assert_eq!(c.binding, DecreeBinding::Recommended);
    }

    #[test]
    fn test_ruling_new() {
        let r = DecreeRuling::new("r1", "Title", "Text");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_ruling_builder() {
        let r = DecreeRuling::new("r1", "Title", "Text")
            .binding(DecreeBinding::Voluntary);
        assert_eq!(r.binding, DecreeBinding::Voluntary);
    }

    #[test]
    fn test_ruling_enact_repeal() {
        let mut r = DecreeRuling::new("r1", "Title", "Text");
        r.enact();
        assert!(r.in_force);
        r.repeal();
        assert!(!r.in_force);
    }

    #[test]
    fn test_clause_new() {
        let c = DecreeClause::new("key", "value", "r1");
        assert_eq!(c.ruling_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DecreeStats::default();
        let mut ruling = DecreeRuling::new("r1", "Title", "Text");
        ruling.enact();
        s.update(&[ruling], DecreeType::Executive);
        assert_eq!(s.total_decrees, 1);
        assert_eq!(s.in_force, 1);
    }

    #[test]
    fn test_decree_new() {
        let d = SettingsDecree::new(DecreeConfig::default());
        assert_eq!(d.ruling_count(), 0);
    }

    #[test]
    fn test_decree_add_ruling() {
        let mut d = SettingsDecree::new(DecreeConfig::default());
        d.add_ruling(DecreeRuling::new("r1", "Title", "Text"));
        assert_eq!(d.ruling_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DecreeRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DecreeRegistry::new();
        r.register("d1", SettingsDecree::new(DecreeConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_decree_query() {
        assert!(is_decree_query("settings decree"));
        assert!(!is_decree_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = decree_fun_fact();
        assert!(fact.contains("decree"));
    }
}
