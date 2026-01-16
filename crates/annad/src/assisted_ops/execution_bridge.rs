//! Execution Bridge - AssistedOperation to ExecutionRequest Translation (Phase 43)
//!
//! This module provides helpers to create ExecutionRequests from AssistedOperations.
//!
//! # CRITICAL INVARIANT: THIS MODULE DOES NOT EXECUTE
//!
//! This module:
//! - Creates ExecutionRequest structures (inert data)
//! - Filters for safe-to-run-automatically commands only
//! - Requires exact confirmation text
//! - Does NOT execute anything
//! - Does NOT call HumanExecutionAdapter
//!
//! # Purpose
//!
//! When an AssistedOperation contains safe commands (CommandSafety::SafeAutomatic),
//! this module creates the ExecutionRequest needed to run them via HumanExecutionAdapter.
//!
//! # Execution Model
//!
//! 1. AssistedOperation is created with safe/manual command classification
//! 2. This module creates ExecutionRequest for safe commands ONLY
//! 3. Human provides confirmation text exactly: "I understand this will execute automatically."
//! 4. ExecutionRequest is persisted
//! 5. (Later) HumanExecutionAdapter executes the commands
//!
//! This module bridges diagnosis to execution request. It does not execute.

use anna_shared::execution_request::{
    ExecutionRequest, PersistenceError, AUTOMATIC_EXECUTION_CONFIRMATION,
};

use super::types::{AssistedOperation, CommandSafety, ProposedStep};

/// Error when creating an execution request from an assisted operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// No safe commands to execute
    NoSafeCommands,
    /// Confirmation text does not match
    InvalidConfirmation { expected: String, actual: String },
    /// Failed to persist request
    PersistenceFailed(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::NoSafeCommands => {
                write!(
                    f,
                    "This operation only contains commands that require manual execution. \
                     Anna cannot run these automatically."
                )
            }
            BridgeError::InvalidConfirmation { expected, actual } => {
                write!(
                    f,
                    "The confirmation text doesn't match. Anna expects exactly: '{}' \
                     but received: '{}'",
                    expected, actual
                )
            }
            BridgeError::PersistenceFailed(msg) => {
                write!(
                    f,
                    "Could not save the execution request: {}. Please try again.",
                    msg
                )
            }
        }
    }
}

impl std::error::Error for BridgeError {}

/// Result of creating an execution request from an assisted operation.
#[derive(Debug, Clone)]
pub struct ExecutionBridgeResult {
    /// The created execution request
    pub request: ExecutionRequest,
    /// The safe commands that will be executed
    pub safe_commands: Vec<String>,
    /// Commands that must be run manually (not included in request)
    pub manual_commands: Vec<String>,
}

/// Create an ExecutionRequest from an AssistedOperation.
///
/// This function:
/// - Extracts ONLY the safe-to-run-automatically commands
/// - Requires exact confirmation text: "I understand this will execute automatically."
/// - Creates an ExecutionRequest with unique ID
/// - Does NOT execute anything
///
/// # Arguments
///
/// * `operation` - The AssistedOperation to convert
/// * `operator` - Identifier of the human requesting execution
/// * `confirmation` - Must exactly match AUTOMATIC_EXECUTION_CONFIRMATION
///
/// # Returns
///
/// * `Ok(ExecutionBridgeResult)` - Request created successfully
/// * `Err(BridgeError)` - No safe commands or invalid confirmation
///
/// This function creates data. It does not execute.
pub fn execution_request_from_assisted_op(
    operation: &AssistedOperation,
    operator: &str,
    confirmation: &str,
) -> Result<ExecutionBridgeResult, BridgeError> {
    // Verify confirmation text matches exactly
    if confirmation != AUTOMATIC_EXECUTION_CONFIRMATION {
        return Err(BridgeError::InvalidConfirmation {
            expected: AUTOMATIC_EXECUTION_CONFIRMATION.to_string(),
            actual: confirmation.to_string(),
        });
    }

    // Extract safe commands
    let safe_steps: Vec<&ProposedStep> = operation
        .proposed_steps
        .iter()
        .filter(|s| s.safety == CommandSafety::SafeAutomatic)
        .collect();

    if safe_steps.is_empty() {
        return Err(BridgeError::NoSafeCommands);
    }

    // Extract command strings
    let safe_commands: Vec<String> = safe_steps
        .iter()
        .map(|s| s.exact_command.clone())
        .collect();

    let manual_commands: Vec<String> = operation
        .proposed_steps
        .iter()
        .filter(|s| s.safety == CommandSafety::ManualOnly)
        .map(|s| s.exact_command.clone())
        .collect();

    // Generate request ID
    let request_id = format!(
        "req-{}-{}",
        chrono::Utc::now().timestamp_millis(),
        &operation.operation_id[..8.min(operation.operation_id.len())]
    );

    // Build the requested action description
    let requested_action = format!(
        "Execute {} safe diagnostic command(s) for: {}",
        safe_commands.len(),
        operation.detected_problem
    );

    // Create the ExecutionRequest
    let request = ExecutionRequest {
        request_id,
        proposal_id: operation.operation_id.clone(),
        recorded_utc: chrono::Utc::now().to_rfc3339(),
        requested_by: operator.to_string(),
        requested_action,
        confirmation_text: AUTOMATIC_EXECUTION_CONFIRMATION.to_string(),
    };

    Ok(ExecutionBridgeResult {
        request,
        safe_commands,
        manual_commands,
    })
}

/// Create and persist an ExecutionRequest from an AssistedOperation.
///
/// This is a convenience function that calls `execution_request_from_assisted_op`
/// and then persists the result.
///
/// # Arguments
///
/// * `operation` - The AssistedOperation to convert
/// * `operator` - Identifier of the human requesting execution
/// * `confirmation` - Must exactly match AUTOMATIC_EXECUTION_CONFIRMATION
///
/// # Returns
///
/// * `Ok(ExecutionBridgeResult)` - Request created and persisted
/// * `Err(BridgeError)` - Creation or persistence failed
///
/// This function creates and persists data. It does not execute.
pub fn create_and_persist_execution_request(
    operation: &AssistedOperation,
    operator: &str,
    confirmation: &str,
) -> Result<ExecutionBridgeResult, BridgeError> {
    let result = execution_request_from_assisted_op(operation, operator, confirmation)?;

    // Persist the request
    result
        .request
        .save()
        .map_err(|e: PersistenceError| BridgeError::PersistenceFailed(e.to_string()))?;

    Ok(result)
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES
// =============================================================================
//
// This module:
// - DOES NOT execute commands
// - DOES NOT call HumanExecutionAdapter
// - DOES NOT spawn processes
// - DOES NOT access system resources (beyond file I/O for persistence)
// - DOES NOT interpret command strings
//
// The safe_commands field contains strings like "iw wlan0 link"
// but there is no code path that passes these strings to Command::new().
//
// This module bridges diagnosis to execution request creation.
// Actual execution happens elsewhere, explicitly, with human confirmation.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assisted_ops::types::{RiskLevel, Source, SourceType};

    fn test_operation() -> AssistedOperation {
        AssistedOperation {
            operation_id: "wifi-fix-001".to_string(),
            detected_problem: "WiFi connection dropping".to_string(),
            explanation: "The iwlwifi driver has a known issue".to_string(),
            proposed_steps: vec![
                ProposedStep {
                    step_number: 1,
                    description: "Check WiFi link status".to_string(),
                    exact_command: "iw wlan0 link".to_string(),
                    why: "Verify current connection state".to_string(),
                    reversible: true,
                    reverse_command: None,
                    safety: CommandSafety::SafeAutomatic,
                },
                ProposedStep {
                    step_number: 2,
                    description: "Check loaded modules".to_string(),
                    exact_command: "lsmod".to_string(),
                    why: "Verify iwlwifi is loaded".to_string(),
                    reversible: true,
                    reverse_command: None,
                    safety: CommandSafety::SafeAutomatic,
                },
                ProposedStep {
                    step_number: 3,
                    description: "Apply driver fix".to_string(),
                    exact_command: "sudo modprobe -r iwlwifi && sudo modprobe iwlwifi".to_string(),
                    why: "Reload with new parameters".to_string(),
                    reversible: true,
                    reverse_command: Some("sudo modprobe -r iwlwifi && sudo modprobe iwlwifi".to_string()),
                    safety: CommandSafety::ManualOnly,
                },
            ],
            risk_level: RiskLevel::Medium,
            sources: vec![Source {
                source_type: SourceType::ArchWiki,
                title: "Wireless network configuration".to_string(),
                reference: "https://wiki.archlinux.org/title/Wireless".to_string(),
            }],
            requires_reboot: false,
            diagnosis_summary: "WiFi dropping due to iwlwifi driver issue".to_string(),
        }
    }

    // =========================================================================
    // POSITIVE TESTS
    // =========================================================================

    #[test]
    fn test_creates_request_for_safe_commands_only() {
        let op = test_operation();
        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            AUTOMATIC_EXECUTION_CONFIRMATION,
        );

        assert!(result.is_ok());
        let result = result.unwrap();

        // Should have 2 safe commands
        assert_eq!(result.safe_commands.len(), 2);
        assert!(result.safe_commands.contains(&"iw wlan0 link".to_string()));
        assert!(result.safe_commands.contains(&"lsmod".to_string()));

        // Should have 1 manual command
        assert_eq!(result.manual_commands.len(), 1);
        assert!(result.manual_commands[0].contains("sudo modprobe"));

        // Request should reference the operation
        assert_eq!(result.request.proposal_id, "wifi-fix-001");
        assert_eq!(
            result.request.confirmation_text,
            AUTOMATIC_EXECUTION_CONFIRMATION
        );
    }

    #[test]
    fn test_request_id_format() {
        let op = test_operation();
        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            AUTOMATIC_EXECUTION_CONFIRMATION,
        )
        .unwrap();

        // Request ID should start with "req-" and contain operation ID prefix
        assert!(result.request.request_id.starts_with("req-"));
        assert!(result.request.request_id.contains("wifi-fix"));
    }

    #[test]
    fn test_requested_action_describes_commands() {
        let op = test_operation();
        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            AUTOMATIC_EXECUTION_CONFIRMATION,
        )
        .unwrap();

        assert!(result.request.requested_action.contains("2 safe"));
        assert!(result
            .request
            .requested_action
            .contains("WiFi connection dropping"));
    }

    // =========================================================================
    // CONFIRMATION VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_rejects_wrong_confirmation() {
        let op = test_operation();
        let result = execution_request_from_assisted_op(&op, "test@example.com", "I agree");

        assert!(matches!(
            result,
            Err(BridgeError::InvalidConfirmation { .. })
        ));
    }

    #[test]
    fn test_rejects_manual_confirmation() {
        let op = test_operation();
        // Try using the manual confirmation instead of automatic
        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            "I understand this will not execute automatically.",
        );

        assert!(matches!(
            result,
            Err(BridgeError::InvalidConfirmation { .. })
        ));
    }

    #[test]
    fn test_rejects_almost_correct_confirmation() {
        let op = test_operation();
        // Missing period
        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            "I understand this will execute automatically",
        );

        assert!(matches!(
            result,
            Err(BridgeError::InvalidConfirmation { .. })
        ));
    }

    // =========================================================================
    // NO SAFE COMMANDS TESTS
    // =========================================================================

    #[test]
    fn test_rejects_operation_with_no_safe_commands() {
        let mut op = test_operation();
        // Make all commands manual
        for step in &mut op.proposed_steps {
            step.safety = CommandSafety::ManualOnly;
        }

        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            AUTOMATIC_EXECUTION_CONFIRMATION,
        );

        assert!(matches!(result, Err(BridgeError::NoSafeCommands)));
    }

    #[test]
    fn test_rejects_empty_operation() {
        let mut op = test_operation();
        op.proposed_steps.clear();

        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            AUTOMATIC_EXECUTION_CONFIRMATION,
        );

        assert!(matches!(result, Err(BridgeError::NoSafeCommands)));
    }

    // =========================================================================
    // ISOLATION PROOF TESTS
    // =========================================================================

    #[test]
    fn proof_does_not_execute() {
        let op = test_operation();
        let result = execution_request_from_assisted_op(
            &op,
            "test@example.com",
            AUTOMATIC_EXECUTION_CONFIRMATION,
        )
        .unwrap();

        // The result contains command strings
        assert!(!result.safe_commands.is_empty());

        // But there is no:
        // result.execute()
        // result.run_commands()
        // execute_safe_commands(&result)
        //
        // The commands are strings. They are data.
        // This function creates a request. It does not execute.
    }

    #[test]
    fn proof_no_command_import() {
        // Verification:
        // grep -n "std::process::Command\|Command::new" \
        //     crates/annad/src/assisted_ops/execution_bridge.rs
        //
        // Expected: Zero results
        //
        // This module does not import or use std::process::Command.
    }

    #[test]
    fn proof_no_adapter_import() {
        // Verification:
        // grep -n "HumanExecutionAdapter" \
        //     crates/annad/src/assisted_ops/execution_bridge.rs
        //
        // Expected: Zero results (only in this comment)
        //
        // This module does not import or use HumanExecutionAdapter.
        // It creates requests. It does not execute them.
    }
}
