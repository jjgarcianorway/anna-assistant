// v0.0.734: Settings Entente (Phase 310)
// Unit tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_entente_type_display() {
        assert_eq!(format!("{}", EntenteType::Cordiale), "cordiale");
        assert_eq!(format!("{}", EntenteType::Strategic), "strategic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", EntenteStatus::Informal), "informal");
        assert_eq!(format!("{}", EntenteStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = EntenteConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = EntenteConfig::new("test")
            .entente_type(EntenteType::Strategic)
            .status(EntenteStatus::Formalized);
        assert_eq!(c.entente_type, EntenteType::Strategic);
        assert_eq!(c.status, EntenteStatus::Formalized);
    }

    #[test]
    fn test_understanding_new() {
        let u = EntenteUnderstanding::new("u1", "Title", "Content");
        assert_eq!(u.id, "u1");
    }

    #[test]
    fn test_understanding_builder() {
        let u = EntenteUnderstanding::new("u1", "Title", "Content")
            .point(1);
        assert_eq!(u.point, 1);
    }

    #[test]
    fn test_understanding_tacit() {
        let mut u = EntenteUnderstanding::new("u1", "Title", "Content");
        u.make_explicit();
        assert!(!u.tacit);
        u.make_tacit();
        assert!(u.tacit);
    }

    #[test]
    fn test_partner_new() {
        let p = EntentePartner::new("key", "name", "u1");
        assert_eq!(p.understanding_id, "u1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = EntenteStats::default();
        let understanding = EntenteUnderstanding::new("u1", "Title", "Content");
        s.update(&[understanding], EntenteType::Cordiale);
        assert_eq!(s.total_understandings, 1);
        assert_eq!(s.tacit, 1);
    }

    #[test]
    fn test_entente_new() {
        let e = SettingsEntente::new(EntenteConfig::default());
        assert_eq!(e.understanding_count(), 0);
    }

    #[test]
    fn test_entente_add_understanding() {
        let mut e = SettingsEntente::new(EntenteConfig::default());
        e.add_understanding(EntenteUnderstanding::new("u1", "Title", "Content"));
        assert_eq!(e.understanding_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = EntenteRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = EntenteRegistry::new();
        r.register("e1", SettingsEntente::new(EntenteConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_entente_query() {
        assert!(is_entente_query("settings entente"));
        assert!(!is_entente_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = entente_fun_fact();
        assert!(fact.contains("entente"));
    }
}
