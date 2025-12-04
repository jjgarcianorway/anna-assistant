//! Integration Tests for Mutation Engine v0.0.80
//!
//! Tests for:
//! - Blocked privilege behavior
//! - Config edit preview
//! - Rollback path
//! - Validation of confirmation phrases

#[cfg(test)]
mod tests {
    use crate::config_mutation::{preview_config_edit, get_config_edit_risk};
    use crate::mutation_engine_v1::{
        is_config_allowed, is_service_allowed, ConfigEditOp, MutationCategory, MutationDetail,
        MutationPlanState, MutationPlanV1, MutationRiskLevel, MutationStats, MutationStep,
        PackageAction, RollbackStep, ServiceAction, StepPreview, VerificationCheck,
    };
    use crate::mutation_orchestrator::{
        plan_config_mutation, plan_package_mutation, plan_service_mutation, validate_confirmation,
    };
    use crate::package_mutation::{
        create_install_rollback, create_remove_rollback, get_package_action_risk,
        preview_package_install, preview_package_remove,
    };
    use crate::privilege::{check_privilege, PrivilegeMethod};
    use crate::service_mutation::{
        create_service_rollback, create_service_verification, get_service_action_risk,
        preview_service_action,
    };

    // =========================================================================
    // Privilege Tests
    // =========================================================================

    #[test]
    fn test_privilege_check_runs() {
        // This test just verifies privilege check doesn't hang
        let status = check_privilege();
        // Result depends on environment, but should be one of the valid methods
        assert!(matches!(
            status.method,
            PrivilegeMethod::Root | PrivilegeMethod::SudoNoPassword | PrivilegeMethod::None
        ));
    }

    #[test]
    fn test_privilege_blocked_generates_manual_commands() {
        let commands = crate::privilege::generate_manual_commands("systemctl", &["restart", "docker"]);
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("sudo"));
        assert!(commands[0].contains("systemctl"));
        assert!(commands[0].contains("restart"));
        assert!(commands[0].contains("docker"));
    }

    // =========================================================================
    // Config Edit Preview Tests
    // =========================================================================

    #[test]
    fn test_config_edit_preview_allowlist() {
        // Allowed config file should preview successfully (if file exists)
        // This test checks allowlist, not actual file existence
        assert!(is_config_allowed("/etc/pacman.conf"));
        assert!(is_config_allowed("/etc/ssh/sshd_config"));
        assert!(is_config_allowed("/etc/NetworkManager/NetworkManager.conf"));

        // Non-allowed files should be rejected
        assert!(!is_config_allowed("/etc/passwd"));
        assert!(!is_config_allowed("/etc/shadow"));
        assert!(!is_config_allowed("/etc/sudoers"));
    }

    #[test]
    fn test_config_edit_preview_rejects_non_allowed() {
        let op = ConfigEditOp::AddLine {
            line: "test".to_string(),
        };
        let result = preview_config_edit("/etc/passwd", &op);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the allowed whitelist"));
    }

    #[test]
    fn test_config_edit_risk_levels() {
        // SSH config should be high risk
        let op = ConfigEditOp::AddLine {
            line: "test".to_string(),
        };
        assert_eq!(
            get_config_edit_risk("/etc/ssh/sshd_config", &op),
            MutationRiskLevel::High
        );

        // pacman.conf should be medium risk
        assert_eq!(
            get_config_edit_risk("/etc/pacman.conf", &op),
            MutationRiskLevel::Medium
        );
    }

    // =========================================================================
    // Service Mutation Tests
    // =========================================================================

    #[test]
    fn test_service_allowlist() {
        assert!(is_service_allowed("NetworkManager"));
        assert!(is_service_allowed("sshd"));
        assert!(is_service_allowed("docker"));
        assert!(is_service_allowed("bluetooth"));

        // Non-allowed services
        assert!(!is_service_allowed("httpd"));
        assert!(!is_service_allowed("nginx"));
        assert!(!is_service_allowed("random-service"));
    }

    #[test]
    fn test_service_preview_rejects_non_allowed() {
        let result = preview_service_action("httpd", &ServiceAction::Start);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the allowed whitelist"));
    }

    #[test]
    fn test_service_risk_levels() {
        // NetworkManager stop is high risk
        assert_eq!(
            get_service_action_risk("NetworkManager", &ServiceAction::Stop),
            MutationRiskLevel::High
        );
        // NetworkManager start is medium risk
        assert_eq!(
            get_service_action_risk("NetworkManager", &ServiceAction::Start),
            MutationRiskLevel::Medium
        );
        // docker restart is low risk
        assert_eq!(
            get_service_action_risk("docker", &ServiceAction::Restart),
            MutationRiskLevel::Low
        );
    }

    #[test]
    fn test_service_rollback_creation() {
        let rollback = create_service_rollback("docker", &ServiceAction::Start);
        assert_eq!(rollback.len(), 1);
        assert!(rollback[0].undo_action.contains("stop"));

        let rollback = create_service_rollback("docker", &ServiceAction::Stop);
        assert!(rollback[0].undo_action.contains("start"));

        let rollback = create_service_rollback("docker", &ServiceAction::Enable);
        assert!(rollback[0].undo_action.contains("disable"));

        let rollback = create_service_rollback("docker", &ServiceAction::Disable);
        assert!(rollback[0].undo_action.contains("enable"));
    }

    #[test]
    fn test_service_verification_creation() {
        let checks = create_service_verification("docker", &ServiceAction::Start);
        assert_eq!(checks.len(), 1);
        assert!(checks[0].expected.contains("active"));

        let checks = create_service_verification("docker", &ServiceAction::Stop);
        assert!(checks[0].expected.contains("inactive"));

        let checks = create_service_verification("docker", &ServiceAction::Enable);
        assert!(checks[0].expected.contains("enabled"));

        let checks = create_service_verification("docker", &ServiceAction::Disable);
        assert!(checks[0].expected.contains("disabled"));
    }

    // =========================================================================
    // Package Mutation Tests
    // =========================================================================

    #[test]
    fn test_package_risk_levels() {
        let pkgs = vec!["vim".to_string()];

        // Install is low risk for small package count
        assert_eq!(
            get_package_action_risk(&PackageAction::Install, &pkgs),
            MutationRiskLevel::Low
        );

        // Remove is medium risk
        assert_eq!(
            get_package_action_risk(&PackageAction::Remove, &pkgs),
            MutationRiskLevel::Medium
        );

        // Many packages increases risk
        let many_pkgs: Vec<String> = (0..10).map(|i| format!("pkg{}", i)).collect();
        assert_eq!(
            get_package_action_risk(&PackageAction::Install, &many_pkgs),
            MutationRiskLevel::Medium
        );
        assert_eq!(
            get_package_action_risk(&PackageAction::Remove, &many_pkgs),
            MutationRiskLevel::High
        );
    }

    #[test]
    fn test_package_rollback_creation() {
        let pkgs = vec!["vim".to_string(), "git".to_string()];

        let install_rollback = create_install_rollback(&pkgs);
        assert_eq!(install_rollback.len(), 1);
        assert!(install_rollback[0].undo_action.contains("pacman -R"));
        assert!(install_rollback[0].undo_action.contains("vim"));

        let remove_rollback = create_remove_rollback(&pkgs);
        assert!(remove_rollback[0].undo_action.contains("pacman -S"));
    }

    // =========================================================================
    // Confirmation Validation Tests
    // =========================================================================

    #[test]
    fn test_confirmation_phrases_match_risk() {
        assert_eq!(
            MutationRiskLevel::Low.confirmation_phrase(),
            "I CONFIRM (low risk)"
        );
        assert_eq!(
            MutationRiskLevel::Medium.confirmation_phrase(),
            "I CONFIRM (medium risk)"
        );
        assert_eq!(
            MutationRiskLevel::High.confirmation_phrase(),
            "I CONFIRM (high risk)"
        );
    }

    #[test]
    fn test_confirmation_validation_with_plan() {
        // Create a low-risk plan
        let plan = plan_service_mutation("docker", ServiceAction::Restart, None).unwrap();

        // Correct phrase should pass
        assert!(validate_confirmation(&plan, "I CONFIRM (low risk)"));

        // Case insensitive
        assert!(validate_confirmation(&plan, "i confirm (low risk)"));

        // Wrong phrase should fail
        assert!(!validate_confirmation(&plan, "I CONFIRM (high risk)"));
        assert!(!validate_confirmation(&plan, "yes"));
        assert!(!validate_confirmation(&plan, "confirm"));
    }

    #[test]
    fn test_rollback_confirmation() {
        assert_eq!(MutationRiskLevel::rollback_phrase(), "I CONFIRM ROLLBACK");
    }

    // =========================================================================
    // Plan Creation Tests
    // =========================================================================

    #[test]
    fn test_service_plan_creation() {
        let plan = plan_service_mutation("docker", ServiceAction::Restart, Some("case-001".to_string()));
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert!(matches!(
            plan.steps[0].category,
            MutationCategory::ServiceControl
        ));
        assert_eq!(plan.risk, MutationRiskLevel::Low); // docker restart is low risk
        assert!(!plan.verification_checks.is_empty());
        assert!(!plan.rollback_steps.is_empty());
    }

    #[test]
    fn test_service_plan_rejects_non_allowed() {
        let plan = plan_service_mutation("httpd", ServiceAction::Start, None);
        assert!(plan.is_err());
        assert!(plan.unwrap_err().contains("not in the allowed"));
    }

    #[test]
    fn test_package_plan_creation() {
        let pkgs = vec!["vim".to_string()];
        let plan = plan_package_mutation(PackageAction::Install, pkgs, Some("case-002".to_string()));
        assert!(plan.is_ok());

        let plan = plan.unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert!(matches!(
            plan.steps[0].category,
            MutationCategory::PackageManagement
        ));
    }

    #[test]
    fn test_package_plan_empty_rejected() {
        let plan = plan_package_mutation(PackageAction::Install, vec![], None);
        assert!(plan.is_err());
        assert!(plan.unwrap_err().contains("No packages"));
    }

    // =========================================================================
    // Mutation Stats Tests
    // =========================================================================

    #[test]
    fn test_mutation_stats_defaults() {
        let stats = MutationStats::default();
        assert_eq!(stats.successful_count, 0);
        assert_eq!(stats.rollback_count, 0);
        assert_eq!(stats.blocked_privilege_count, 0);
        assert!(stats.last_mutation_outcome.is_none());
    }

    #[test]
    fn test_mutation_stats_recording() {
        let mut stats = MutationStats::default();

        stats.record_success();
        assert_eq!(stats.successful_count, 1);
        assert_eq!(stats.last_mutation_outcome, Some("success".to_string()));

        stats.record_rollback();
        assert_eq!(stats.rollback_count, 1);
        assert_eq!(stats.last_mutation_outcome, Some("rolled_back".to_string()));

        stats.record_privilege_block();
        assert_eq!(stats.blocked_privilege_count, 1);
        assert_eq!(
            stats.last_mutation_outcome,
            Some("blocked_privilege".to_string())
        );
    }

    // =========================================================================
    // Plan State Transitions
    // =========================================================================

    #[test]
    fn test_plan_state_display() {
        assert_eq!(format!("{}", MutationPlanState::Created), "created");
        assert_eq!(format!("{}", MutationPlanState::Previewed), "previewed");
        assert_eq!(
            format!("{}", MutationPlanState::AwaitingConfirmation),
            "awaiting_confirmation"
        );
        assert_eq!(
            format!("{}", MutationPlanState::BlockedPrivilege),
            "blocked_privilege"
        );
        assert_eq!(format!("{}", MutationPlanState::Confirmed), "confirmed");
        assert_eq!(format!("{}", MutationPlanState::Executing), "executing");
        assert_eq!(
            format!("{}", MutationPlanState::VerifiedSuccess),
            "verified_success"
        );
        assert_eq!(format!("{}", MutationPlanState::RolledBack), "rolled_back");
    }

    // =========================================================================
    // Risk Level Display
    // =========================================================================

    #[test]
    fn test_risk_level_display() {
        assert_eq!(format!("{}", MutationRiskLevel::Low), "low");
        assert_eq!(format!("{}", MutationRiskLevel::Medium), "medium");
        assert_eq!(format!("{}", MutationRiskLevel::High), "high");
    }
}
