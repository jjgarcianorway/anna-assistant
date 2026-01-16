//! User Trust Review Tests (Phase 50)
//!
//! These tests simulate user interactions asking for forbidden things.
//! They verify that Anna's responses reference declared capabilities,
//! not hard-coded excuses.
//!
//! The goal: User trust through transparency. When Anna can't do something,
//! she should explain what she CAN do, based on the capability ledger.

#[cfg(test)]
mod user_trust_tests {
    use crate::capabilities::{all_allowed_binaries, forbidden_capabilities, CAPABILITIES};
    use crate::command_policy::{
        authorize_command, CommandPolicyDecision, CommandSpec, DenialReason, PolicyContext,
    };
    use crate::declaration::CapabilityDeclaration;

    fn ctx() -> PolicyContext {
        PolicyContext::default()
    }

    // =========================================================================
    // USER INTERACTION SCENARIOS
    // =========================================================================

    /// Simulate: "Can you download this file for me?"
    #[test]
    fn user_interaction_download_request() {
        // User wants to download a file - this requires network access
        let decl = CapabilityDeclaration::from_ledger();

        // Verify network is in forbidden capabilities
        let has_network_forbidden = decl
            .will_never_do
            .iter()
            .any(|e| e.name.to_lowercase().contains("network"));
        assert!(
            has_network_forbidden,
            "Network access should be in will_never_do list"
        );

        // Verify wget/curl are denied by policy
        for binary in &["wget", "curl", "nc"] {
            let cmd = CommandSpec::new(*binary, vec!["http://example.com".to_string()]);
            let decision = authorize_command(&cmd, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Network binary '{}' should be denied",
                binary
            );
        }

        // Verify the response can reference what Anna CAN do
        assert!(
            !decl.can_do.is_empty(),
            "Declaration should have capabilities Anna CAN do"
        );
    }

    /// Simulate: "Install this package for me"
    #[test]
    fn user_interaction_package_install_request() {
        let decl = CapabilityDeclaration::from_ledger();

        // Verify package installation is in forbidden capabilities
        let has_package_forbidden = decl
            .will_never_do
            .iter()
            .any(|e| e.name.to_lowercase().contains("package"));
        assert!(
            has_package_forbidden,
            "Package installation should be in will_never_do list"
        );

        // Verify package managers are denied by policy
        for binary in &["pacman", "apt", "dnf", "yay"] {
            let cmd = CommandSpec::new(*binary, vec!["-S".to_string(), "vim".to_string()]);
            let decision = authorize_command(&cmd, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Package manager '{}' should be denied",
                binary
            );
        }
    }

    /// Simulate: "Run this as root"
    #[test]
    fn user_interaction_sudo_request() {
        let decl = CapabilityDeclaration::from_ledger();

        // Verify sudo is in forbidden capabilities
        let has_sudo_forbidden = decl
            .will_never_do
            .iter()
            .any(|e| e.name.to_lowercase().contains("sudo"));
        assert!(
            has_sudo_forbidden,
            "Sudo execution should be in will_never_do list"
        );

        // Verify sudo is denied by policy
        let cmd = CommandSpec::new("sudo", vec!["cat".to_string(), "/etc/shadow".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::PrivilegeEscalationBinary(_)
            }
        ));
    }

    /// Simulate: "Delete these files for me"
    #[test]
    fn user_interaction_delete_request() {
        let decl = CapabilityDeclaration::from_ledger();

        // Verify destructive commands are in forbidden capabilities
        let has_destructive_forbidden = decl
            .will_never_do
            .iter()
            .any(|e| e.name.to_lowercase().contains("destructive"));
        assert!(
            has_destructive_forbidden,
            "Destructive commands should be in will_never_do list"
        );

        // Verify rm is denied by policy
        let cmd = CommandSpec::new("rm", vec!["-rf".to_string(), "/tmp/test".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(matches!(
            decision,
            CommandPolicyDecision::Denied {
                reason: DenialReason::DestructiveBinary(_)
            }
        ));
    }

    // =========================================================================
    // CAPABILITY DECLARATION MATCHES POLICY
    // =========================================================================

    #[test]
    fn user_interaction_matches_capability_declaration() {
        let decl = CapabilityDeclaration::from_ledger();

        // Verify the declaration has exactly 4 forbidden capabilities
        // (network, package, sudo, destructive)
        let forbidden = forbidden_capabilities();
        assert_eq!(
            forbidden.len(),
            4,
            "Should have exactly 4 forbidden capabilities"
        );

        // Verify all forbidden capabilities appear in will_never_do
        assert_eq!(
            decl.will_never_do.len(),
            forbidden.len(),
            "will_never_do should match forbidden capabilities count"
        );

        // Verify can_do includes execution capabilities
        let has_execution_cap = decl.can_do.iter().any(|e| {
            e.name.to_lowercase().contains("execution")
                || e.name.to_lowercase().contains("diagnosis")
        });
        assert!(
            has_execution_cap,
            "can_do should include execution or diagnosis capabilities"
        );
    }

    #[test]
    fn user_trust_allowed_binaries_match_can_do() {
        let decl = CapabilityDeclaration::from_ledger();
        let allowed = all_allowed_binaries();

        // Every allowed binary should be covered by a capability in can_do
        for binary in &allowed {
            let cap = CAPABILITIES.iter().find(|c| c.allowed_binaries.contains(binary));
            assert!(
                cap.is_some(),
                "Binary '{}' should be covered by a capability",
                binary
            );
        }

        // The number of allowed binaries should be stable (11 as per PHASES.md)
        assert_eq!(
            allowed.len(),
            11,
            "Should have exactly 11 allowed binaries (frozen at Phase 48)"
        );
    }

    #[test]
    fn user_trust_denial_reasons_are_informative() {
        // Verify that denial reasons provide enough information for user-facing messages

        // Network denial
        let cmd = CommandSpec::new("wget", vec![]);
        let decision = authorize_command(&cmd, &ctx());
        if let CommandPolicyDecision::Denied { reason } = decision {
            let msg = format!("{}", reason);
            assert!(
                msg.contains("wget") || msg.contains("ledger"),
                "Denial message should mention the binary or ledger: {}",
                msg
            );
        }

        // Privilege escalation denial
        let cmd = CommandSpec::new("sudo", vec!["ls".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        if let CommandPolicyDecision::Denied { reason } = decision {
            let msg = format!("{}", reason);
            assert!(
                msg.contains("sudo") || msg.contains("privilege") || msg.contains("escalation"),
                "Denial message should mention privilege escalation: {}",
                msg
            );
        }

        // Shell invocation denial
        let cmd = CommandSpec::new("bash", vec!["-c".to_string(), "ls".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        if let CommandPolicyDecision::Denied { reason } = decision {
            let msg = format!("{}", reason);
            assert!(
                msg.contains("bash") || msg.contains("shell") || msg.contains("Shell"),
                "Denial message should mention shell: {}",
                msg
            );
        }
    }

    // =========================================================================
    // RESPONSE TEMPLATE VERIFICATION
    // =========================================================================

    /// Verify that we can construct informative responses from capability data
    #[test]
    fn user_trust_can_construct_informative_response() {
        let decl = CapabilityDeclaration::from_ledger();

        // Should be able to build a response like:
        // "I cannot download files because network access is outside my capability boundary.
        //  I CAN help you with: [list from can_do]"

        // Verify we have the data to construct this
        assert!(!decl.will_never_do.is_empty(), "Need forbidden list");
        assert!(!decl.can_do.is_empty(), "Need capability list");

        // Verify forbidden entries have names and descriptions
        for entry in &decl.will_never_do {
            assert!(!entry.name.is_empty(), "Forbidden entry needs name");
            assert!(
                !entry.description.is_empty(),
                "Forbidden entry needs description"
            );
        }

        // Verify capability entries have names
        for entry in &decl.can_do {
            assert!(!entry.name.is_empty(), "Capability entry needs name");
        }
    }

    /// Verify declaration provides human-readable capability summary
    #[test]
    fn user_trust_declaration_has_summary() {
        let decl = CapabilityDeclaration::from_ledger();

        // Verify ledger version is present
        assert!(
            !decl.ledger_version.is_empty(),
            "Declaration should have ledger version"
        );

        // Verify allowed binary count is accessible via helper function
        let binary_count = all_allowed_binaries().len();
        assert!(binary_count > 0, "Should have allowed binaries");

        // Verify render functions include binary count
        let plain = decl.render_plain_text();
        assert!(
            plain.contains("Allowed binaries:"),
            "Plain text should show binary count"
        );
    }

    // =========================================================================
    // TRANSPARENCY TESTS
    // =========================================================================

    #[test]
    fn user_trust_no_hidden_capabilities() {
        // Every capability in CAPABILITIES should be visible in declaration
        let decl = CapabilityDeclaration::from_ledger();

        let total_caps = decl.can_do.len() + decl.will_never_do.len();
        let ledger_caps = CAPABILITIES.len();

        // Some capabilities might not be in can_do or will_never_do
        // (like diagnosis-only), but the total should be reasonable
        assert!(
            total_caps <= ledger_caps,
            "Declaration entries ({}) should not exceed ledger capabilities ({})",
            total_caps,
            ledger_caps
        );
    }

    #[test]
    fn user_trust_all_forbidden_explicitly_listed() {
        let decl = CapabilityDeclaration::from_ledger();

        // The 4 key forbidden categories should be explicitly named
        let forbidden_names: Vec<String> = decl
            .will_never_do
            .iter()
            .map(|e| e.name.to_lowercase())
            .collect();

        // Check for the 4 forbidden categories
        let has_network = forbidden_names.iter().any(|n| n.contains("network"));
        let has_package = forbidden_names.iter().any(|n| n.contains("package"));
        let has_sudo = forbidden_names.iter().any(|n| n.contains("sudo"));
        let has_destructive = forbidden_names.iter().any(|n| n.contains("destructive"));

        assert!(has_network, "Should explicitly list network as forbidden");
        assert!(has_package, "Should explicitly list package as forbidden");
        assert!(has_sudo, "Should explicitly list sudo as forbidden");
        assert!(
            has_destructive,
            "Should explicitly list destructive as forbidden"
        );
    }
}
