// v0.0.735: Settings Alliance (Phase 311)
// Tests for settings alliance module

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_alliance_type_display() {
        assert_eq!(format!("{}", AllianceType::Military), "military");
        assert_eq!(format!("{}", AllianceType::Economic), "economic");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", AllianceStatus::Forming), "forming");
        assert_eq!(format!("{}", AllianceStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = AllianceConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AllianceConfig::new("test")
            .alliance_type(AllianceType::Economic)
            .status(AllianceStatus::Active);
        assert_eq!(c.alliance_type, AllianceType::Economic);
        assert_eq!(c.status, AllianceStatus::Active);
    }

    #[test]
    fn test_commitment_new() {
        let c = AllianceCommitment::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_commitment_builder() {
        let c = AllianceCommitment::new("c1", "Title", "Content")
            .article(1);
        assert_eq!(c.article, 1);
    }

    #[test]
    fn test_commitment_binding() {
        let mut c = AllianceCommitment::new("c1", "Title", "Content");
        c.make_optional();
        assert!(!c.binding);
        c.make_binding();
        assert!(c.binding);
    }

    #[test]
    fn test_member_new() {
        let m = AllianceMember::new("key", "name", "c1");
        assert_eq!(m.commitment_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = AllianceStats::default();
        let commitment = AllianceCommitment::new("c1", "Title", "Content");
        s.update(&[commitment], AllianceType::Military);
        assert_eq!(s.total_commitments, 1);
        assert_eq!(s.binding, 1);
    }

    #[test]
    fn test_alliance_new() {
        let a = SettingsAlliance::new(AllianceConfig::default());
        assert_eq!(a.commitment_count(), 0);
    }

    #[test]
    fn test_alliance_add_commitment() {
        let mut a = SettingsAlliance::new(AllianceConfig::default());
        a.add_commitment(AllianceCommitment::new("c1", "Title", "Content"));
        assert_eq!(a.commitment_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = AllianceRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = AllianceRegistry::new();
        r.register("a1", SettingsAlliance::new(AllianceConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_alliance_query() {
        assert!(is_alliance_query("settings alliance"));
        assert!(!is_alliance_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = alliance_fun_fact();
        assert!(fact.contains("alliance"));
    }
}
