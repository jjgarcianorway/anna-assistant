//! Command Policy Guardrails - Regression Tests for Single Authorization Path (Phase 47)
//!
//! These tests prove that:
//! 1. Policy decisions match what `annactl capabilities` shows
//! 2. No second authorization mechanism exists outside policy.rs and adapter
//! 3. New commands cannot be authorized without updating the ledger

#[cfg(test)]
mod guardrail_tests {
    use crate::capabilities::{all_allowed_binaries, execution_capabilities, CAPABILITIES};
    use crate::command_policy::{
        authorize_command, verify_policy_ledger_consistency, CommandPolicyDecision, CommandSpec,
        PolicyContext,
    };
    use crate::declaration::CapabilityDeclaration;
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;

    fn ctx() -> PolicyContext {
        PolicyContext::default()
    }

    // =========================================================================
    // INVARIANT 1: Policy matches annactl capabilities
    // =========================================================================

    #[test]
    fn guardrail_policy_matches_declaration() {
        // Get what declaration shows
        let decl = CapabilityDeclaration::from_ledger();

        // Count capabilities that allow execution
        let can_do_with_confirmation = decl
            .can_do
            .iter()
            .filter(|e| {
                e.details
                    .as_ref()
                    .map_or(false, |d| d.contains("confirmation"))
            })
            .count();

        // Must have at least 2 execution capabilities (human_mediated, automatic_safe)
        assert!(
            can_do_with_confirmation >= 2,
            "Declaration should show at least 2 execution capabilities"
        );

        // Verify forbidden capabilities match
        assert!(
            !decl.will_never_do.is_empty(),
            "Declaration must show forbidden capabilities"
        );
    }

    #[test]
    fn guardrail_policy_authorizes_exactly_ledger_binaries() {
        let ledger_binaries = all_allowed_binaries();

        // Every ledger binary must be authorizable
        for binary in &ledger_binaries {
            let cmd = CommandSpec::new(*binary, vec![]);
            let decision = authorize_command(&cmd, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Allowed { .. }),
                "Ledger binary '{}' must be authorizable",
                binary
            );
        }

        // Non-ledger binaries must NOT be authorizable
        let forbidden = ["wget", "curl", "pacman", "apt", "sudo", "rm", "dd"];
        for binary in &forbidden {
            let cmd = CommandSpec::new(*binary, vec![]);
            let decision = authorize_command(&cmd, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Non-ledger binary '{}' must be denied",
                binary
            );
        }
    }

    #[test]
    fn guardrail_policy_ledger_consistency() {
        // This is the key invariant: policy must be consistent with ledger
        let result = verify_policy_ledger_consistency();
        assert!(
            result.is_ok(),
            "Policy/ledger inconsistency: {:?}",
            result.err()
        );
    }

    // =========================================================================
    // INVARIANT 2: No second authorization mechanism
    // =========================================================================

    #[test]
    fn guardrail_no_second_authorization_outside_policy() {
        // Check key files for unauthorized authorization patterns
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

        // Files that should NOT have authorization logic
        let files_to_check = [
            "execution_request.rs",
            "capabilities.rs",
            "declaration.rs",
            "action_plan.rs",
        ];

        let forbidden_patterns = [
            // No module should have its own binary validation logic
            "fn is_binary_allowed",
            "fn check_binary_allowlist",
            // No module should bypass policy
            "validate_without_policy",
            "skip_policy_check",
        ];

        for filename in &files_to_check {
            let path = src_dir.join(filename);
            if !path.exists() {
                continue;
            }

            let content = fs::read_to_string(&path).expect("Failed to read file");

            // Filter out comments
            let lines: Vec<&str> = content
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.starts_with("//") && !trimmed.starts_with("*")
                })
                .collect();
            let code = lines.join("\n");

            for pattern in &forbidden_patterns {
                assert!(
                    !code.contains(pattern),
                    "File {} contains forbidden pattern '{}' - only command_policy.rs should authorize commands",
                    filename,
                    pattern
                );
            }
        }
    }

    #[test]
    fn guardrail_human_execution_uses_policy() {
        // Verify that human_execution.rs imports and uses command_policy
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/human_execution.rs");
        let content = fs::read_to_string(&path).expect("Failed to read human_execution.rs");

        // Must import from command_policy
        assert!(
            content.contains("use crate::command_policy::"),
            "human_execution.rs must import from command_policy"
        );

        // Must call authorize_command
        assert!(
            content.contains("authorize_command"),
            "human_execution.rs must use authorize_command"
        );
    }

    // =========================================================================
    // INVARIANT 3: New commands require ledger update
    // =========================================================================

    #[test]
    fn guardrail_adding_binary_requires_ledger_update() {
        // This test documents the contract: to add a new binary, you must:
        // 1. Add it to CAPABILITIES in capabilities.rs
        // 2. The policy will then automatically authorize it
        // 3. Tests will verify consistency

        // Prove that a binary not in ledger is denied
        let hypothetical_new_binary = "some_new_tool_not_in_ledger";
        let cmd = CommandSpec::new(hypothetical_new_binary, vec![]);
        let decision = authorize_command(&cmd, &ctx());

        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "New binary must be denied until added to ledger"
        );

        // Prove that ledger binaries are authorized
        let ledger_binaries = all_allowed_binaries();
        assert!(
            !ledger_binaries.is_empty(),
            "Ledger must have some allowed binaries"
        );

        for binary in &ledger_binaries {
            let cmd = CommandSpec::new(*binary, vec![]);
            let decision = authorize_command(&cmd, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Allowed { .. }),
                "Ledger binary '{}' must be authorized",
                binary
            );
        }
    }

    #[test]
    fn guardrail_execution_capabilities_count_stable() {
        // Track the number of execution capabilities
        // This must be explicitly updated when adding new execution capabilities
        let exec_caps = execution_capabilities();

        // Currently we have exactly 2 execution capabilities
        assert_eq!(
            exec_caps.len(),
            2,
            "Execution capability count changed! Update this test and ensure the change is intentional."
        );
    }

    #[test]
    fn guardrail_allowed_binary_count_stable() {
        // Track the total number of allowed binaries
        let all_binaries = all_allowed_binaries();

        // Currently we have exactly 11 allowed binaries
        assert_eq!(
            all_binaries.len(),
            11,
            "Allowed binary count changed! Update this test and ensure the change is intentional. Found: {:?}",
            all_binaries
        );
    }

    // =========================================================================
    // INVARIANT 4: Hard bans cannot be bypassed
    // =========================================================================

    #[test]
    fn guardrail_hard_bans_comprehensive() {
        // Privilege escalation - all forms
        let priv_escalation = ["sudo", "su", "pkexec", "doas", "runuser", "dzdo"];
        for cmd in &priv_escalation {
            let spec = CommandSpec::new(*cmd, vec!["anything".to_string()]);
            let decision = authorize_command(&spec, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Privilege escalation '{}' must be denied",
                cmd
            );
        }

        // Shells - all forms
        let shells = ["sh", "bash", "zsh", "fish", "dash", "csh", "tcsh", "ksh"];
        for shell in &shells {
            let spec = CommandSpec::new(*shell, vec!["-c".to_string(), "ls".to_string()]);
            let decision = authorize_command(&spec, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Shell '{}' must be denied",
                shell
            );
        }

        // Destructive commands
        let destructive = ["rm", "dd", "mkfs", "fdisk", "parted", "shred", "wipefs"];
        for cmd in &destructive {
            let spec = CommandSpec::new(*cmd, vec![]);
            let decision = authorize_command(&spec, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Destructive command '{}' must be denied",
                cmd
            );
        }

        // Network commands
        let network = ["wget", "curl", "nc", "netcat", "ssh", "scp", "rsync", "ftp"];
        for cmd in &network {
            let spec = CommandSpec::new(*cmd, vec![]);
            let decision = authorize_command(&spec, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Network command '{}' must be denied",
                cmd
            );
        }

        // Package managers
        let pkg_managers = ["pacman", "yay", "paru", "apt", "apt-get", "dnf", "yum"];
        for cmd in &pkg_managers {
            let spec = CommandSpec::new(*cmd, vec![]);
            let decision = authorize_command(&spec, &ctx());
            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "Package manager '{}' must be denied",
                cmd
            );
        }
    }

    #[test]
    fn guardrail_dangerous_patterns_in_args_denied() {
        // Even if the primary binary is allowed, dangerous patterns in args are denied
        let allowed_binary = "echo"; // This IS in the ledger

        // Sudo in args
        let cmd = CommandSpec::new(allowed_binary, vec!["sudo".to_string(), "ls".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "sudo in args must be denied"
        );

        // rm in args
        let cmd = CommandSpec::new(allowed_binary, vec!["rm".to_string(), "-rf".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "rm in args must be denied"
        );

        // Pipes
        let cmd = CommandSpec::new(allowed_binary, vec!["test".to_string(), "|".to_string(), "bash".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "pipe in args must be denied"
        );

        // Redirects
        let cmd = CommandSpec::new(allowed_binary, vec!["test".to_string(), ">".to_string(), "/tmp/file".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "redirect in args must be denied"
        );

        // Command substitution
        let cmd = CommandSpec::new(allowed_binary, vec!["$(whoami)".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "command substitution in args must be denied"
        );

        // Chaining
        let cmd = CommandSpec::new(allowed_binary, vec!["test".to_string(), "&&".to_string(), "rm".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "chaining in args must be denied"
        );

        // /dev/ access
        let cmd = CommandSpec::new("cat", vec!["/dev/sda".to_string()]);
        let decision = authorize_command(&cmd, &ctx());
        assert!(
            matches!(decision, CommandPolicyDecision::Denied { .. }),
            "/dev/ access must be denied"
        );
    }
}
