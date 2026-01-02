// v0.0.728: Settings Protocol - Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_protocol_type_display() {
        assert_eq!(format!("{}", ProtocolType::Amendment), "amendment");
        assert_eq!(format!("{}", ProtocolType::Optional), "optional");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ProtocolStatus::Draft), "draft");
        assert_eq!(format!("{}", ProtocolStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = ProtocolConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ProtocolConfig::new("test")
            .protocol_type(ProtocolType::Optional)
            .status(ProtocolStatus::Open);
        assert_eq!(c.protocol_type, ProtocolType::Optional);
        assert_eq!(c.status, ProtocolStatus::Open);
    }

    #[test]
    fn test_clause_new() {
        let c = ProtocolClause::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_clause_builder() {
        let c = ProtocolClause::new("c1", "Title", "Content")
            .section(1);
        assert_eq!(c.section, 1);
    }

    #[test]
    fn test_clause_adopt_reject() {
        let mut c = ProtocolClause::new("c1", "Title", "Content");
        c.adopt();
        assert!(c.adopted);
        c.reject();
        assert!(!c.adopted);
    }

    #[test]
    fn test_party_new() {
        let p = ProtocolParty::new("key", "name", "c1");
        assert_eq!(p.clause_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ProtocolStats::default();
        let mut clause = ProtocolClause::new("c1", "Title", "Content");
        clause.adopt();
        s.update(&[clause], ProtocolType::Amendment);
        assert_eq!(s.total_clauses, 1);
        assert_eq!(s.adopted, 1);
    }

    #[test]
    fn test_protocol_new() {
        let p = SettingsProtocol::new(ProtocolConfig::default());
        assert_eq!(p.clause_count(), 0);
    }

    #[test]
    fn test_protocol_add_clause() {
        let mut p = SettingsProtocol::new(ProtocolConfig::default());
        p.add_clause(ProtocolClause::new("c1", "Title", "Content"));
        assert_eq!(p.clause_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ProtocolRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ProtocolRegistry::new();
        r.register("p1", SettingsProtocol::new(ProtocolConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_protocol_query() {
        assert!(is_protocol_query("settings protocol"));
        assert!(!is_protocol_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = protocol_fun_fact();
        assert!(fact.contains("protocol"));
    }
}
