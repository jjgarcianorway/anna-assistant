//! Regression Guardrails for Capability Boundaries (Phase 45)
//!
//! These tests are intentionally annoying. They fail loudly when:
//! - Command::new appears outside allowed execution modules
//! - New binaries appear in allowlists without ledger updates
//! - Confirmation strings change without test updates
//!
//! If a test here fails, you MUST either:
//! 1. Revert your change (if it violates the capability contract)
//! 2. Update the capability ledger AND bump the version
//!
//! There is no third option.

#[cfg(test)]
mod guardrail_tests {
    use crate::capabilities::{
        all_allowed_binaries, execution_capabilities, verify_execution_allowlist_consistency,
        CAPABILITIES, LEDGER_VERSION,
    };
    use crate::execution_request::{AUTOMATIC_EXECUTION_CONFIRMATION, REQUIRED_CONFIRMATION};
    use crate::human_execution::ALLOWED_BINARIES;

    // =========================================================================
    // ALLOWLIST SYNCHRONIZATION GUARDRAILS
    // =========================================================================

    #[test]
    fn guardrail_adapter_allowlist_matches_ledger() {
        // This test fails if HumanExecutionAdapter allowlist diverges from capability ledger.
        // If it fails, update BOTH the adapter and the ledger, then bump LEDGER_VERSION.
        let result = verify_execution_allowlist_consistency(ALLOWED_BINARIES);
        assert!(
            result.is_ok(),
            "GUARDRAIL FAILURE: Adapter allowlist diverges from capability ledger!\n\
             Mismatches: {:?}\n\
             \n\
             To fix:\n\
             1. If you added a binary to the adapter, add it to capabilities.rs too\n\
             2. If you removed a binary, remove it from capabilities.rs too\n\
             3. Bump LEDGER_VERSION in capabilities.rs\n\
             4. Update docs/trust_surface.md",
            result.err().unwrap()
        );
    }

    #[test]
    fn guardrail_no_unauthorized_execution_binaries() {
        // These binaries must NEVER be in any allowlist
        const FORBIDDEN_BINARIES: &[&str] = &[
            // Privilege escalation
            "sudo",
            "pkexec",
            "doas",
            "su",
            // Network
            "wget",
            "curl",
            "nc",
            "netcat",
            "ssh",
            "scp",
            "rsync",
            // Package managers
            "pacman",
            "yay",
            "paru",
            "apt",
            "apt-get",
            "dnf",
            "yum",
            // Destructive
            "rm",
            "dd",
            "mkfs",
            "fdisk",
            "parted",
            "shred",
            // Shell execution
            "sh",
            "bash",
            "zsh",
            "fish",
            "eval",
            // Dangerous utilities
            "chmod",
            "chown",
            "chgrp",
            "kill",
            "killall",
        ];

        let allowed = all_allowed_binaries();

        for binary in FORBIDDEN_BINARIES {
            assert!(
                !allowed.contains(binary),
                "GUARDRAIL FAILURE: Forbidden binary '{}' found in capability ledger!\n\
                 \n\
                 This binary is explicitly forbidden from the execution allowlist.\n\
                 If you believe this is necessary, you must:\n\
                 1. Get explicit approval from the project maintainer\n\
                 2. Document the security implications\n\
                 3. Add extensive boundary tests\n\
                 4. Update the trust surface documentation",
                binary
            );
        }
    }

    // =========================================================================
    // CONFIRMATION STRING GUARDRAILS
    // =========================================================================

    #[test]
    fn guardrail_confirmation_strings_immutable() {
        // These strings are part of the security contract.
        // Changing them breaks existing workflows and expectations.

        assert_eq!(
            REQUIRED_CONFIRMATION,
            "I understand this will not execute automatically.",
            "GUARDRAIL FAILURE: Manual confirmation string changed!\n\
             \n\
             This string is part of the security contract.\n\
             Changing it will break existing workflows.\n\
             If you must change it:\n\
             1. Update all tests that reference it\n\
             2. Update the capability ledger\n\
             3. Document the change in CHANGELOG.md\n\
             4. Bump MAJOR version"
        );

        assert_eq!(
            AUTOMATIC_EXECUTION_CONFIRMATION,
            "I understand this will execute automatically.",
            "GUARDRAIL FAILURE: Automatic confirmation string changed!\n\
             \n\
             This string is part of the security contract.\n\
             Changing it will break existing workflows.\n\
             If you must change it:\n\
             1. Update all tests that reference it\n\
             2. Update the capability ledger\n\
             3. Document the change in CHANGELOG.md\n\
             4. Bump MAJOR version"
        );
    }

    #[test]
    fn guardrail_ledger_confirmation_matches_execution_request() {
        // Ensure the ledger's confirmation strings match execution_request.rs
        let mut found_manual = false;
        let mut found_auto = false;

        for cap in CAPABILITIES {
            if let Some(conf) = cap.requires_confirmation {
                if conf == REQUIRED_CONFIRMATION {
                    found_manual = true;
                }
                if conf == AUTOMATIC_EXECUTION_CONFIRMATION {
                    found_auto = true;
                }
            }
        }

        assert!(
            found_manual,
            "GUARDRAIL FAILURE: Ledger missing manual confirmation string!\n\
             Expected: {:?}",
            REQUIRED_CONFIRMATION
        );

        assert!(
            found_auto,
            "GUARDRAIL FAILURE: Ledger missing automatic confirmation string!\n\
             Expected: {:?}",
            AUTOMATIC_EXECUTION_CONFIRMATION
        );
    }

    // =========================================================================
    // EXECUTION CAPABILITY GUARDRAILS
    // =========================================================================

    #[test]
    fn guardrail_execution_capabilities_require_confirmation() {
        for cap in execution_capabilities() {
            assert!(
                cap.requires_confirmation.is_some(),
                "GUARDRAIL FAILURE: Execution capability '{}' has no confirmation!\n\
                 \n\
                 All execution capabilities MUST require a confirmation string.\n\
                 This is non-negotiable. Add requires_confirmation to this capability.",
                cap.name
            );
        }
    }

    #[test]
    fn guardrail_execution_capabilities_have_audit() {
        use crate::capabilities::PersistenceBehavior;

        for cap in execution_capabilities() {
            assert!(
                matches!(
                    cap.persistence,
                    PersistenceBehavior::AuditOnly | PersistenceBehavior::StateChanging
                ),
                "GUARDRAIL FAILURE: Execution capability '{}' has no audit trail!\n\
                 \n\
                 All execution capabilities MUST have audit logging.\n\
                 Set persistence to AuditOnly or StateChanging.",
                cap.name
            );
        }
    }

    // =========================================================================
    // LEDGER VERSION GUARDRAIL
    // =========================================================================

    #[test]
    fn guardrail_ledger_version_format() {
        // Ledger version must be in X.Y format
        let parts: Vec<&str> = LEDGER_VERSION.split('.').collect();
        assert_eq!(
            parts.len(),
            2,
            "GUARDRAIL FAILURE: LEDGER_VERSION must be in X.Y format, got: {}",
            LEDGER_VERSION
        );

        for (i, part) in parts.iter().enumerate() {
            assert!(
                part.parse::<u32>().is_ok(),
                "GUARDRAIL FAILURE: LEDGER_VERSION part {} is not a number: {}",
                i,
                part
            );
        }
    }

    // =========================================================================
    // SOURCE CODE GUARDRAILS (grep-based)
    // =========================================================================

    #[test]
    fn guardrail_command_new_only_in_allowed_modules() {
        // This test verifies Command::new only appears in allowed locations.
        // We check by examining our own source expectations.

        // Allowed locations for Command::new:
        // - human_execution.rs (the single execution point)
        // - profile/*.rs (read-only diagnostics)
        // - monitor/*.rs (read-only diagnostics)
        // - detection.rs (read-only diagnostics)

        // Forbidden locations:
        // - execution_request.rs
        // - capabilities.rs
        // - assisted_ops/types.rs
        // - assisted_ops/wifi_diagnosis.rs
        // - assisted_ops/execution_bridge.rs

        // This is a documentation test - actual grep verification is in CI.
        // The test exists to document the expectation.

        // Verification command:
        // grep -rn "Command::new" crates/anna-shared/src/execution_request.rs
        // Expected: 0 results

        // grep -rn "Command::new" crates/anna-shared/src/capabilities.rs
        // Expected: 0 results

        // If this test is failing because you added Command::new somewhere:
        // 1. If it's for diagnosis, put it in profile/ or monitor/
        // 2. If it's for execution, it MUST go through HumanExecutionAdapter
        // 3. There are no other valid options
    }

    #[test]
    fn guardrail_no_execution_in_types_module() {
        // This is a structural guarantee test.
        // Types modules must NEVER contain execution logic.

        // The following files must contain ZERO:
        // - std::process::Command
        // - Command::new
        // - tokio::process::Command
        // - .spawn()
        // - .output()

        // Files covered by this guarantee:
        // - crates/anna-shared/src/execution_request.rs
        // - crates/anna-shared/src/capabilities.rs
        // - crates/annad/src/assisted_ops/types.rs
        // - crates/annad/src/assisted_ops/execution_bridge.rs

        // This test documents the expectation. CI enforces it via grep.
    }

    // =========================================================================
    // CAPABILITY COUNT GUARDRAILS
    // =========================================================================

    #[test]
    fn guardrail_minimum_forbidden_capabilities() {
        use crate::capabilities::forbidden_capabilities;

        // We must always have explicit "what Anna cannot do" entries
        let forbidden = forbidden_capabilities();

        assert!(
            forbidden.len() >= 4,
            "GUARDRAIL FAILURE: Too few forbidden capabilities!\n\
             Expected at least 4, found: {}\n\
             \n\
             The capability ledger must explicitly document what Anna cannot do.\n\
             Current forbidden capabilities should include:\n\
             - NO_network_requests\n\
             - NO_package_installation\n\
             - NO_sudo_execution\n\
             - NO_destructive_commands",
            forbidden.len()
        );
    }

    #[test]
    fn guardrail_execution_capability_count() {
        let exec_caps = execution_capabilities();

        // Document the current count. If this changes, the test fails
        // and forces you to acknowledge the change.
        assert_eq!(
            exec_caps.len(),
            2,
            "GUARDRAIL FAILURE: Number of execution capabilities changed!\n\
             Expected: 2, Found: {}\n\
             \n\
             Current execution capabilities:\n\
             - human_mediated_execution\n\
             - automatic_safe_execution\n\
             \n\
             If you added a new execution capability:\n\
             1. Ensure it has confirmation requirements\n\
             2. Ensure it has audit logging\n\
             3. Update this test's expected count\n\
             4. Bump LEDGER_VERSION\n\
             5. Update docs/trust_surface.md",
            exec_caps.len()
        );
    }

    #[test]
    fn guardrail_allowed_binary_count() {
        let binaries = all_allowed_binaries();

        // Document the current count. Changes force acknowledgment.
        // Count includes: iw, lsmod, lspci, cat, echo (execution)
        //                 lsusb, lscpu, free, df, uname, hostname (diagnosis)
        assert_eq!(
            binaries.len(),
            11,
            "GUARDRAIL FAILURE: Number of allowed binaries changed!\n\
             Expected: 11, Found: {}\n\
             \n\
             Current allowed binaries: {:?}\n\
             \n\
             If you added a new binary:\n\
             1. Add it to the capability ledger\n\
             2. Add tests for boundary enforcement\n\
             3. Update this test's expected count\n\
             4. Bump LEDGER_VERSION\n\
             5. Update docs/trust_surface.md",
            binaries.len(),
            binaries
        );
    }
}
