//! Tests for intake module (v0.0.180).

#[cfg(test)]
mod tests {
    use crate::facts::{FactKey, FactsStore};
    use crate::intake::{
        analyze_intake, check_slot_satisfied, ClarificationQuestion, ClarificationSlot,
        VerificationResult, VerifyPlan,
    };
    use crate::rpc::{QueryIntent, SpecialistDomain};

    #[test]
    fn test_verify_plan_probe_command() {
        let plan = VerifyPlan::BinaryExists {
            binary: "vim".to_string(),
        };
        assert_eq!(plan.probe_command(), Some("command -v vim".to_string()));

        let plan = VerifyPlan::None;
        assert_eq!(plan.probe_command(), None);
    }

    #[test]
    fn test_clarification_question_builder() {
        let q = ClarificationQuestion::new("test", "Test question?", "testing")
            .with_choices(vec!["a", "b"])
            .with_verify(VerifyPlan::BinaryExists {
                binary: "test".to_string(),
            })
            .with_priority(5);

        assert_eq!(q.id, "test");
        assert_eq!(q.choices, vec!["a", "b"]);
        assert_eq!(q.priority, 5);
    }

    #[test]
    fn test_analyze_intake_editor_with_known_fact() {
        let mut facts = FactsStore::new();
        facts.set_verified(
            FactKey::PreferredEditor,
            "vim".to_string(),
            "test".to_string(),
        );
        facts.set_verified(
            FactKey::BinaryAvailable("vim".to_string()),
            "/usr/bin/vim".to_string(),
            "test".to_string(),
        );

        let result = analyze_intake(
            "enable syntax highlighting in my editor",
            QueryIntent::Request,
            SpecialistDomain::System,
            &facts,
            &[],
        );

        assert!(result.can_proceed);
        assert!(result.clarifications_needed.is_empty());
        assert!(result.facts_used.contains(&FactKey::PreferredEditor));
    }

    #[test]
    fn test_analyze_intake_editor_without_fact() {
        let facts = FactsStore::new();

        let result = analyze_intake(
            "enable syntax highlighting in my editor",
            QueryIntent::Request,
            SpecialistDomain::System,
            &facts,
            &[],
        );

        assert!(!result.can_proceed);
        assert!(!result.clarifications_needed.is_empty());
        assert_eq!(result.clarifications_needed[0].id, "editor_selection");
    }

    #[test]
    fn test_analyze_intake_network_clarification() {
        let facts = FactsStore::new();

        let result = analyze_intake(
            "my internet connection is broken",
            QueryIntent::Investigate,
            SpecialistDomain::Network,
            &facts,
            &[],
        );

        assert!(!result.can_proceed);
        assert!(result
            .clarifications_needed
            .iter()
            .any(|c| c.id == "network_interface"));
    }

    #[test]
    fn test_verification_result_success() {
        let result = VerificationResult::success("/usr/bin/vim".to_string(), "probe:which vim");
        assert!(result.verified);
        assert_eq!(result.value, Some("/usr/bin/vim".to_string()));
    }

    #[test]
    fn test_verification_result_failed_with_alternatives() {
        let result = VerificationResult::failed_with_alternatives(
            "vim not found",
            vec!["vi".to_string(), "nvim".to_string()],
            "probe:which vim",
        );
        assert!(!result.verified);
        assert_eq!(result.alternatives, vec!["vi", "nvim"]);
    }

    #[test]
    fn test_check_slot_satisfied() {
        let mut facts = FactsStore::new();
        facts.set_verified(
            FactKey::PreferredEditor,
            "vim".to_string(),
            "test".to_string(),
        );

        let result = check_slot_satisfied(ClarificationSlot::EditorName, &facts);
        assert_eq!(result, Some("vim".to_string()));

        let result = check_slot_satisfied(ClarificationSlot::NetworkInterface, &facts);
        assert_eq!(result, None);
    }
}
