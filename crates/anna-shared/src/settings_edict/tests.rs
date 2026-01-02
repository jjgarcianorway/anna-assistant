// v0.0.719: Settings Edict - Tests
// Test suite for settings edict module

#[cfg(test)]
mod tests {
    use crate::settings_edict::*;

    #[test]
    fn test_edict_type_display() {
        assert_eq!(format!("{}", EdictType::Royal), "royal");
        assert_eq!(format!("{}", EdictType::Imperial), "imperial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", EdictStatus::Draft), "draft");
        assert_eq!(format!("{}", EdictStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = EdictConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = EdictConfig::new("test")
            .edict_type(EdictType::Sovereign)
            .default_status(EdictStatus::Proclaimed);
        assert_eq!(c.edict_type, EdictType::Sovereign);
        assert_eq!(c.default_status, EdictStatus::Proclaimed);
    }

    #[test]
    fn test_proclamation_new() {
        let p = EdictProclamation::new("p1", "Title", "Decree");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_proclamation_builder() {
        let p = EdictProclamation::new("p1", "Title", "Decree")
            .seal("ROYAL_SEAL");
        assert_eq!(p.seal, "ROYAL_SEAL");
    }

    #[test]
    fn test_proclamation_lifecycle() {
        let mut p = EdictProclamation::new("p1", "Title", "Decree");
        p.proclaim();
        assert_eq!(p.status, EdictStatus::Proclaimed);
        p.activate();
        assert_eq!(p.status, EdictStatus::Active);
        p.revoke();
        assert_eq!(p.status, EdictStatus::Revoked);
    }

    #[test]
    fn test_annotation_new() {
        let a = EdictAnnotation::new("key", "value", "p1");
        assert_eq!(a.proclamation_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = EdictStats::default();
        let mut proc = EdictProclamation::new("p1", "Title", "Decree");
        proc.activate();
        s.update(&[proc], EdictType::Royal);
        assert_eq!(s.total_edicts, 1);
        assert_eq!(s.active, 1);
    }

    #[test]
    fn test_edict_new() {
        let e = SettingsEdict::new(EdictConfig::default());
        assert_eq!(e.proclamation_count(), 0);
    }

    #[test]
    fn test_edict_add_proclamation() {
        let mut e = SettingsEdict::new(EdictConfig::default());
        e.add_proclamation(EdictProclamation::new("p1", "Title", "Decree"));
        assert_eq!(e.proclamation_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = EdictRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = EdictRegistry::new();
        r.register("e1", SettingsEdict::new(EdictConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_edict_query() {
        assert!(is_edict_query("settings edict"));
        assert!(!is_edict_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = edict_fun_fact();
        assert!(fact.contains("edict"));
    }
}
