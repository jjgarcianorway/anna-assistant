// v0.0.721: Settings Mandate Tests (Phase 297)
// Test suite for mandate system

#[cfg(test)]
mod tests {
    use super::super::{
        mandate::SettingsMandate,
        registry::MandateRegistry,
        types::{MandateCompliance, MandateConfig, MandateEvidence, MandateRequirement, MandateStats, MandateType},
        utils::{is_mandate_query, mandate_fun_fact},
    };

    #[test]
    fn test_mandate_type_display() {
        assert_eq!(format!("{}", MandateType::Legal), "legal");
        assert_eq!(format!("{}", MandateType::Regulatory), "regulatory");
    }

    #[test]
    fn test_compliance_display() {
        assert_eq!(format!("{}", MandateCompliance::Required), "required");
        assert_eq!(format!("{}", MandateCompliance::Exempt), "exempt");
    }

    #[test]
    fn test_config_new() {
        let c = MandateConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MandateConfig::new("test")
            .mandate_type(MandateType::Corporate)
            .default_compliance(MandateCompliance::Recommended);
        assert_eq!(c.mandate_type, MandateType::Corporate);
        assert_eq!(c.default_compliance, MandateCompliance::Recommended);
    }

    #[test]
    fn test_requirement_new() {
        let r = MandateRequirement::new("r1", "Title", "Description");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_requirement_builder() {
        let r = MandateRequirement::new("r1", "Title", "Description")
            .compliance(MandateCompliance::Optional);
        assert_eq!(r.compliance, MandateCompliance::Optional);
    }

    #[test]
    fn test_requirement_fulfill() {
        let mut r = MandateRequirement::new("r1", "Title", "Description");
        r.fulfill();
        assert!(r.fulfilled);
        r.unfulfill();
        assert!(!r.fulfilled);
    }

    #[test]
    fn test_evidence_new() {
        let e = MandateEvidence::new("key", "value", "r1");
        assert_eq!(e.requirement_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MandateStats::default();
        let mut req = MandateRequirement::new("r1", "Title", "Description");
        req.fulfill();
        s.update(&[req], MandateType::Legal);
        assert_eq!(s.total_mandates, 1);
        assert_eq!(s.fulfilled, 1);
        assert_eq!(s.required_count, 1);
    }

    #[test]
    fn test_mandate_new() {
        let m = SettingsMandate::new(MandateConfig::default());
        assert_eq!(m.requirement_count(), 0);
    }

    #[test]
    fn test_mandate_add_requirement() {
        let mut m = SettingsMandate::new(MandateConfig::default());
        m.add_requirement(MandateRequirement::new("r1", "Title", "Description"));
        assert_eq!(m.requirement_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MandateRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MandateRegistry::new();
        r.register("m1", SettingsMandate::new(MandateConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_mandate_query() {
        assert!(is_mandate_query("settings mandate"));
        assert!(!is_mandate_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = mandate_fun_fact();
        assert!(fact.contains("mandate"));
    }
}
