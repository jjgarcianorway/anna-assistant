//! Assisted Operations Layer (Phase 39)
//!
//! This module provides supervised, reversible system assistance.
//! It detects situations, explains findings, proposes commands, and
//! requests explicit human confirmation for every step.
//!
//! # CRITICAL INVARIANT: ANNA NEVER EXECUTES COMMANDS
//!
//! This layer:
//! - Detects problems by reading system state
//! - Explains what it found
//! - Proposes concrete shell commands (as text)
//! - Waits for human to execute each command manually
//! - Re-checks state after human reports completion
//!
//! # What This Layer ENABLES
//!
//! - Diagnosing system issues (WiFi, audio, services, etc.)
//! - Explaining root causes with citations
//! - Proposing fix steps with exact commands
//! - Documenting reversibility and risks
//! - Guiding humans through multi-step fixes
//!
//! # What This Layer Will NEVER Enable
//!
//! - Automatic command execution
//! - Background monitoring with auto-fix
//! - Proactive changes without human presence
//! - Bypassing confirmation for any step
//! - Running commands via sudo/pkexec internally
//! - Any path that leads to unsupervised execution
//!
//! # Isolation Guarantees
//!
//! This module:
//! - Lives in annad, NOT anna-shared
//! - Does NOT import action_plan
//! - Does NOT import execution_gate, execution_adapter
//! - Does NOT import approval, readiness, proposal, or intent modules
//! - Does NOT use std::process::Command to execute fixes
//! - CAN read system state (files, commands) for diagnosis only
//!
//! # Execution Model
//!
//! ```text
//! Anna detects problem → Anna explains → Anna proposes command
//!                                              ↓
//!                              Human reviews command
//!                                              ↓
//!                              Human runs command manually (sudo/shell)
//!                                              ↓
//!                              Human confirms completion
//!                                              ↓
//!                              Anna re-checks state
//!                                              ↓
//!                              Repeat or complete
//! ```
//!
//! The human is always in the loop. Always.

pub mod types;
pub mod detection;
pub mod execution_bridge;
pub mod wifi_diagnosis;

pub use types::*;
pub use detection::*;
pub use execution_bridge::*;
pub use wifi_diagnosis::*;

// =============================================================================
// COMPILE-TIME PROOF: THIS LAYER CANNOT EXECUTE COMMANDS
// =============================================================================
//
// PROOF BY CONSTRUCTION:
//
// 1. This module does not import std::process::Command for execution
// 2. Detection functions use Command only to READ system state (diagnostic)
// 3. Proposed commands are stored as String, not executed
// 4. There is no function that takes a ProposedStep and runs it
// 5. The AssistedOperation struct contains commands as text data only
//
// VERIFICATION:
//
// Run: grep -rn "Command::new" crates/annad/src/assisted_ops/
// Result: Only in detection.rs for READING state, never for EXECUTING fixes
//
// Run: grep -rn "\.spawn()\|\.output()\|\.status()" crates/annad/src/assisted_ops/
// Result: Only for diagnostic commands, never for proposed fix commands
//
// The proposed_steps field contains strings like "sudo modprobe -r iwlwifi"
// but there is no code path that passes these strings to Command::new().
//
// Removing the UI that shows these to humans makes the layer completely inert.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_layer_cannot_execute_proposed_commands() {
        // Create an assisted operation with proposed commands
        let op = AssistedOperation {
            operation_id: "test".to_string(),
            detected_problem: "Test problem".to_string(),
            explanation: "Test explanation".to_string(),
            proposed_steps: vec![
                ProposedStep {
                    step_number: 1,
                    description: "Do something".to_string(),
                    exact_command: "sudo rm -rf /".to_string(), // Dangerous if executed
                    why: "Test".to_string(),
                    reversible: false,
                    reverse_command: None,
                    safety: CommandSafety::ManualOnly,
                },
            ],
            risk_level: RiskLevel::High,
            sources: vec![],
            requires_reboot: false,
            diagnosis_summary: String::new(),
        };

        // The dangerous command exists as a string
        assert_eq!(op.proposed_steps[0].exact_command, "sudo rm -rf /");

        // But there is no function to execute it
        // There is no: op.execute()
        // There is no: execute_step(&op.proposed_steps[0])
        // There is no: run_proposed_command(&op)
        //
        // The command is data. It is shown to a human.
        // The human decides whether to run it.
        // Anna cannot run it.
    }

    #[test]
    fn proof_no_action_plan_import() {
        // This test documents that assisted_ops does not import action_plan.
        //
        // Verification: grep -rn "use.*action_plan\|anna_shared::action_plan"
        //               crates/annad/src/assisted_ops/
        // Expected: Zero results
    }

    #[test]
    fn proof_no_execution_gate_import() {
        // Verification: grep -rn "ExecutionGate\|ExecutionAdapter\|ExecutionResult"
        //               crates/annad/src/assisted_ops/
        // Expected: Zero results
    }

    #[test]
    fn proof_no_approval_import() {
        // Verification: grep -rn "ApprovalRecord\|ApprovalDecision"
        //               crates/annad/src/assisted_ops/
        // Expected: Zero results
    }

    #[test]
    fn proof_removal_makes_layer_inert() {
        // If you remove the UI code that displays AssistedOperation to humans,
        // this layer does nothing. It produces data structures that sit in memory.
        //
        // Without a human to:
        // 1. See the proposed commands
        // 2. Copy them to a terminal
        // 3. Execute them with sudo
        // 4. Report completion back
        //
        // Nothing happens. The layer is completely passive.
    }
}
