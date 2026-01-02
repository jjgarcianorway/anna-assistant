// v0.0.545: Escalation Policy Config - Tests (Phase 121)
// Test cases for escalation policy configuration

#[cfg(test)]
mod tests {
    use super::super::config::EscalationPolicyConfig;
    use super::super::formatting::{escalation_policy_fun_fact, is_escalation_policy_query};
    use super::super::types::{EscalationMode, EscalationPriority, EscalationTrigger};

    #[test]
    fn test_trigger_display() {
        assert_eq!(
            format!("{}", EscalationTrigger::LowConfidence),
            "Low Confidence"
        );
        assert_eq!(
            format!("{}", EscalationTrigger::SecurityRelated),
            "Security Related"
        );
    }

    #[test]
    fn test_default_config() {
        let config = EscalationPolicyConfig::default();
        assert_eq!(config.mode, EscalationMode::Automatic);
        assert_eq!(config.confidence_threshold, 70);
    }

    #[test]
    fn test_lenient_preset() {
        let config = EscalationPolicyConfig::lenient();
        assert_eq!(config.mode, EscalationMode::SemiAutomatic);
        assert_eq!(config.confidence_threshold, 50);
        assert!(!config.auto_escalate_high_risk);
    }

    #[test]
    fn test_strict_preset() {
        let config = EscalationPolicyConfig::strict();
        assert_eq!(config.confidence_threshold, 85);
        assert!(config.auto_escalate_high_risk);
    }

    #[test]
    fn test_should_escalate_confidence() {
        let config = EscalationPolicyConfig::default();
        assert!(config.should_escalate_confidence(50));
        assert!(!config.should_escalate_confidence(80));
    }

    #[test]
    fn test_disabled_mode() {
        let mut config = EscalationPolicyConfig::default();
        config.mode = EscalationMode::Disabled;
        assert!(!config.should_escalate_confidence(10));
        assert!(!config.should_escalate_security());
    }

    #[test]
    fn test_priority_for_trigger() {
        let config = EscalationPolicyConfig::default();
        assert_eq!(
            config.priority_for(EscalationTrigger::SecurityRelated),
            EscalationPriority::Critical
        );
        assert_eq!(
            config.priority_for(EscalationTrigger::LowConfidence),
            EscalationPriority::Normal
        );
    }

    #[test]
    fn test_should_notify() {
        let mut config = EscalationPolicyConfig::default();
        assert!(config.should_notify(EscalationPriority::Normal));

        config.notify = super::super::types::EscalationNotify::OnlyHighPriority;
        assert!(!config.should_notify(EscalationPriority::Normal));
        assert!(config.should_notify(EscalationPriority::High));
    }

    #[test]
    fn test_apply_automatic() {
        let mut config = EscalationPolicyConfig::manual();
        let result = config.apply_change("use automatic escalation");
        assert!(result.is_some());
        assert_eq!(config.mode, EscalationMode::Automatic);
    }

    #[test]
    fn test_apply_strict() {
        let mut config = EscalationPolicyConfig::default();
        config.apply_change("use strict escalation policy");
        assert_eq!(config.confidence_threshold, 85);
    }

    #[test]
    fn test_is_policy_query() {
        assert!(is_escalation_policy_query("Show escalation policy"));
        assert!(is_escalation_policy_query("When to escalate?"));
        assert!(!is_escalation_policy_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = escalation_policy_fun_fact();
        assert!(fact.contains("30%"));
    }
}
