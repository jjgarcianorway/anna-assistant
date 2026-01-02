// v0.0.724: Settings Charter - Tests module
// Unit tests for charter functionality

#[cfg(test)]
mod tests {
    use super::super::types::{CharterType, CharterStatus};
    use super::super::config::CharterConfig;
    use super::super::provision::{CharterProvision, CharterAmendment};
    use super::super::stats::CharterStats;
    use super::super::charter::SettingsCharter;
    use super::super::registry::{CharterRegistry, is_charter_query, charter_fun_fact};

    #[test]
    fn test_charter_type_display() {
        assert_eq!(format!("{}", CharterType::Founding), "founding");
        assert_eq!(format!("{}", CharterType::Royal), "royal");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", CharterStatus::Draft), "draft");
        assert_eq!(format!("{}", CharterStatus::Ratified), "ratified");
    }

    #[test]
    fn test_config_new() {
        let c = CharterConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CharterConfig::new("test")
            .charter_type(CharterType::Municipal)
            .status(CharterStatus::Amended);
        assert_eq!(c.charter_type, CharterType::Municipal);
        assert_eq!(c.status, CharterStatus::Amended);
    }

    #[test]
    fn test_provision_new() {
        let p = CharterProvision::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_provision_builder() {
        let p = CharterProvision::new("p1", "Title", "Content")
            .section("Section 1");
        assert_eq!(p.section, "Section 1");
    }

    #[test]
    fn test_provision_activate_deactivate() {
        let mut p = CharterProvision::new("p1", "Title", "Content");
        p.deactivate();
        assert!(!p.active);
        p.activate();
        assert!(p.active);
    }

    #[test]
    fn test_amendment_new() {
        let a = CharterAmendment::new("key", "value", "p1");
        assert_eq!(a.provision_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CharterStats::default();
        let provision = CharterProvision::new("p1", "Title", "Content");
        s.update(&[provision], CharterType::Founding);
        assert_eq!(s.total_provisions, 1);
        assert_eq!(s.active, 1);
    }

    #[test]
    fn test_charter_new() {
        let c = SettingsCharter::new(CharterConfig::default());
        assert_eq!(c.provision_count(), 0);
    }

    #[test]
    fn test_charter_add_provision() {
        let mut c = SettingsCharter::new(CharterConfig::default());
        c.add_provision(CharterProvision::new("p1", "Title", "Content"));
        assert_eq!(c.provision_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CharterRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CharterRegistry::new();
        r.register("c1", SettingsCharter::new(CharterConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_charter_query() {
        assert!(is_charter_query("settings charter"));
        assert!(!is_charter_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = charter_fun_fact();
        assert!(fact.contains("charter"));
    }
}
