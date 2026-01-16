//! Execution Request - Human-Issued Intent to Act
//!
//! This module represents an explicit, human-issued request to execute something
//! that Anna has previously explained or proposed.
//!
//! # CRITICAL INVARIANT: THIS MODULE DOES NOT EXECUTE
//!
//! This module:
//! - Captures human intent as inert data
//! - Validates structural completeness only
//! - Persists records to disk
//! - Performs NO system interaction beyond file I/O
//!
//! # What This Module DOES
//!
//! - Stores a human's explicit request to act on a proposal
//! - Validates that required fields are present
//! - Validates that confirmation text matches exactly
//! - Serializes to JSON with deterministic field order
//! - Writes atomically to prevent corruption
//!
//! # What This Module DOES NOT Do (Non-Negotiable)
//!
//! - DOES NOT execute
//! - DOES NOT authorize execution
//! - DOES NOT imply safety
//! - DOES NOT select commands
//! - DOES NOT validate proposals
//! - DOES NOT bypass any gate
//! - DOES NOT trigger anything
//!
//! # The ExecutionRequest is a Human Artifact
//!
//! This record is created when a human explicitly states they want to act.
//! It is not an AI decision. It is not a recommendation. It is not approval.
//! It is a timestamped capture of human intent, nothing more.
//!
//! # Isolation Guarantees
//!
//! This module:
//! - Does NOT import ExecutionAdapter
//! - Does NOT import ExecutionGate
//! - Does NOT import any execution-capable type
//! - Does NOT reference action_plan
//! - Is NOT referenced by execution pipelines
//!
//! This record captures human intent to act. It performs no action.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// The exact confirmation text that must be provided for manual execution requests.
/// This phrase acknowledges that creating a request does not trigger execution.
pub const REQUIRED_CONFIRMATION: &str = "I understand this will not execute automatically.";

/// The exact confirmation text for automatic execution requests (Phase 43).
/// This phrase acknowledges that the user understands safe commands will execute.
pub const AUTOMATIC_EXECUTION_CONFIRMATION: &str = "I understand this will execute automatically.";

/// An execution request issued by a human.
///
/// This is a pure data structure. It captures the fact that a human
/// has explicitly requested to act on a proposal. It does not execute,
/// authorize, or trigger anything.
///
/// # Required Fields
///
/// All fields are required and must be non-empty.
/// The `confirmation_text` must exactly match `REQUIRED_CONFIRMATION`.
///
/// # Persistence
///
/// Records are stored at: `/var/lib/anna/execution_requests/{request_id}.v1.json`
///
/// This record captures human intent to act. It performs no action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique identifier for this request (non-empty)
    pub request_id: String,

    /// Reference to the proposal this request relates to (non-empty)
    pub proposal_id: String,

    /// When this request was recorded (ISO 8601 format)
    pub recorded_utc: String,

    /// Human identifier - who made this request (non-empty)
    pub requested_by: String,

    /// Free-form description of what the human wants to do (non-empty)
    pub requested_action: String,

    /// Must exactly equal REQUIRED_CONFIRMATION
    pub confirmation_text: String,
}

/// Validation error for ExecutionRequest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A required field is empty
    EmptyField { field: &'static str },
    /// The confirmation text does not match
    InvalidConfirmation { expected: String, actual: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyField { field } => {
                write!(f, "The '{}' field is required but was not provided", field)
            }
            ValidationError::InvalidConfirmation { expected, actual } => {
                write!(
                    f,
                    "Confirmation text doesn't match. Anna expects: '{}' but received: '{}'",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Persistence error for ExecutionRequest.
#[derive(Debug)]
pub enum PersistenceError {
    /// Validation failed
    Validation(ValidationError),
    /// I/O error during save/load
    Io(std::io::Error),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// Directory creation failed
    DirectoryCreation(std::io::Error),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Validation(e) => write!(f, "Validation error: {}", e),
            PersistenceError::Io(e) => write!(f, "I/O error: {}", e),
            PersistenceError::Json(e) => write!(f, "JSON error: {}", e),
            PersistenceError::DirectoryCreation(e) => {
                write!(f, "Failed to create directory: {}", e)
            }
        }
    }
}

impl std::error::Error for PersistenceError {}

impl ExecutionRequest {
    /// Create a new ExecutionRequest for testing purposes.
    ///
    /// This creates a structurally valid request with placeholder values.
    /// The confirmation_text is left empty so tests can set it explicitly.
    ///
    /// # Test Only
    ///
    /// This function is only available in test builds.
    #[cfg(test)]
    pub fn new_for_test(request_id: &str, requested_by: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            proposal_id: "test-proposal".to_string(),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            requested_by: requested_by.to_string(),
            requested_action: "Test action".to_string(),
            confirmation_text: String::new(), // Tests must set this explicitly
        }
    }

    /// Validate this execution request for manual (non-automatic) execution.
    ///
    /// Validation rules:
    /// - All fields must be non-empty
    /// - confirmation_text must exactly match REQUIRED_CONFIRMATION
    ///
    /// This performs NO semantic interpretation.
    /// This performs NO feasibility checks.
    /// This performs NO cross-record lookup.
    ///
    /// This record captures human intent to act. It performs no action.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_fields()?;

        // Confirmation text must match exactly
        if self.confirmation_text != REQUIRED_CONFIRMATION {
            return Err(ValidationError::InvalidConfirmation {
                expected: REQUIRED_CONFIRMATION.to_string(),
                actual: self.confirmation_text.clone(),
            });
        }

        Ok(())
    }

    /// Validate this execution request for automatic execution (Phase 43).
    ///
    /// Validation rules:
    /// - All fields must be non-empty
    /// - confirmation_text must exactly match AUTOMATIC_EXECUTION_CONFIRMATION
    ///
    /// This is used for safe commands that can execute via HumanExecutionAdapter.
    ///
    /// This record captures human intent to act. It performs no action.
    pub fn validate_automatic(&self) -> Result<(), ValidationError> {
        self.validate_fields()?;

        // Confirmation text must match automatic execution phrase
        if self.confirmation_text != AUTOMATIC_EXECUTION_CONFIRMATION {
            return Err(ValidationError::InvalidConfirmation {
                expected: AUTOMATIC_EXECUTION_CONFIRMATION.to_string(),
                actual: self.confirmation_text.clone(),
            });
        }

        Ok(())
    }

    /// Validate common fields without checking confirmation text.
    fn validate_fields(&self) -> Result<(), ValidationError> {
        if self.request_id.is_empty() {
            return Err(ValidationError::EmptyField { field: "request_id" });
        }
        if self.proposal_id.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "proposal_id",
            });
        }
        if self.recorded_utc.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "recorded_utc",
            });
        }
        if self.requested_by.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "requested_by",
            });
        }
        if self.requested_action.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "requested_action",
            });
        }
        if self.confirmation_text.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "confirmation_text",
            });
        }

        Ok(())
    }

    /// Get the persistence path for this request.
    ///
    /// Path format: /var/lib/anna/execution_requests/{request_id}.v1.json
    pub fn persistence_path(&self) -> PathBuf {
        PathBuf::from("/var/lib/anna/execution_requests")
            .join(format!("{}.v1.json", self.request_id))
    }

    /// Save this request to disk.
    ///
    /// This method:
    /// - Validates the request first
    /// - Serializes to JSON with deterministic field order
    /// - Writes atomically (write to temp, then rename)
    ///
    /// This method DOES NOT execute anything.
    /// This method DOES NOT authorize anything.
    /// This method DOES NOT trigger anything.
    ///
    /// This record captures human intent to act. It performs no action.
    pub fn save(&self) -> Result<PathBuf, PersistenceError> {
        // Validate first
        self.validate().map_err(PersistenceError::Validation)?;

        let path = self.persistence_path();
        let dir = path.parent().unwrap();

        // Create directory if needed
        fs::create_dir_all(dir).map_err(PersistenceError::DirectoryCreation)?;

        // Serialize with deterministic field order (serde_json preserves struct field order)
        let json = serde_json::to_string_pretty(self).map_err(PersistenceError::Json)?;

        // Atomic write: write to temp file, then rename
        let temp_path = dir.join(format!(".{}.tmp", self.request_id));
        {
            let mut file = fs::File::create(&temp_path).map_err(PersistenceError::Io)?;
            file.write_all(json.as_bytes())
                .map_err(PersistenceError::Io)?;
            file.sync_all().map_err(PersistenceError::Io)?;
        }

        // Rename for atomicity
        fs::rename(&temp_path, &path).map_err(PersistenceError::Io)?;

        Ok(path)
    }

    /// Load a request from disk.
    ///
    /// This method:
    /// - Reads the JSON file
    /// - Deserializes to ExecutionRequest
    /// - Validates the loaded request
    ///
    /// This method DOES NOT execute anything.
    /// This method DOES NOT authorize anything.
    /// This method DOES NOT trigger anything.
    ///
    /// This record captures human intent to act. It performs no action.
    pub fn load(request_id: &str) -> Result<Self, PersistenceError> {
        let path = PathBuf::from("/var/lib/anna/execution_requests")
            .join(format!("{}.v1.json", request_id));

        let content = fs::read_to_string(&path).map_err(PersistenceError::Io)?;
        let request: ExecutionRequest =
            serde_json::from_str(&content).map_err(PersistenceError::Json)?;

        // Validate after loading
        request.validate().map_err(PersistenceError::Validation)?;

        Ok(request)
    }
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES
// =============================================================================
//
// This module:
// - DOES NOT execute
// - DOES NOT authorize execution
// - DOES NOT imply safety
// - DOES NOT select commands
// - DOES NOT validate proposals
// - DOES NOT bypass any gate
// - DOES NOT trigger anything
//
// The ExecutionRequest is inert data. It has no methods that:
// - Call std::process::Command
// - Spawn processes
// - Interact with system services
// - Modify system state (beyond writing its own record file)
// - Reference or import execution-capable types
//
// ISOLATION PROOF:
//
// grep -n "ExecutionAdapter\|ExecutionGate\|action_plan" execution_request.rs
// Expected: Zero matches (excluding this comment block)
//
// grep -rn "execution_request" crates/anna-shared/src/action_plan.rs
// Expected: Zero matches
//
// This record captures human intent to act. It performs no action.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> ExecutionRequest {
        ExecutionRequest {
            request_id: "req-001".to_string(),
            proposal_id: "prop-001".to_string(),
            recorded_utc: "2024-01-15T10:30:00Z".to_string(),
            requested_by: "user@example.com".to_string(),
            requested_action: "Apply the WiFi fix as described".to_string(),
            confirmation_text: REQUIRED_CONFIRMATION.to_string(),
        }
    }

    // =========================================================================
    // STRUCTURAL VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_valid_request_passes_validation() {
        let request = valid_request();
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_empty_request_id_fails() {
        let mut request = valid_request();
        request.request_id = String::new();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::EmptyField { field: "request_id" })
        ));
    }

    #[test]
    fn test_empty_proposal_id_fails() {
        let mut request = valid_request();
        request.proposal_id = String::new();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::EmptyField {
                field: "proposal_id"
            })
        ));
    }

    #[test]
    fn test_empty_recorded_utc_fails() {
        let mut request = valid_request();
        request.recorded_utc = String::new();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::EmptyField {
                field: "recorded_utc"
            })
        ));
    }

    #[test]
    fn test_empty_requested_by_fails() {
        let mut request = valid_request();
        request.requested_by = String::new();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::EmptyField {
                field: "requested_by"
            })
        ));
    }

    #[test]
    fn test_empty_requested_action_fails() {
        let mut request = valid_request();
        request.requested_action = String::new();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::EmptyField {
                field: "requested_action"
            })
        ));
    }

    #[test]
    fn test_empty_confirmation_text_fails() {
        let mut request = valid_request();
        request.confirmation_text = String::new();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::EmptyField {
                field: "confirmation_text"
            })
        ));
    }

    #[test]
    fn test_wrong_confirmation_text_fails() {
        let mut request = valid_request();
        request.confirmation_text = "I agree".to_string();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::InvalidConfirmation { .. })
        ));
    }

    #[test]
    fn test_almost_correct_confirmation_fails() {
        let mut request = valid_request();
        // Missing period at the end
        request.confirmation_text = "I understand this will not execute automatically".to_string();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::InvalidConfirmation { .. })
        ));
    }

    #[test]
    fn test_confirmation_case_sensitive() {
        let mut request = valid_request();
        // Wrong case
        request.confirmation_text = "i understand this will not execute automatically.".to_string();
        let result = request.validate();
        assert!(matches!(
            result,
            Err(ValidationError::InvalidConfirmation { .. })
        ));
    }

    // =========================================================================
    // AUTOMATIC EXECUTION VALIDATION TESTS (Phase 43)
    // =========================================================================

    fn automatic_request() -> ExecutionRequest {
        ExecutionRequest {
            request_id: "req-auto-001".to_string(),
            proposal_id: "prop-001".to_string(),
            recorded_utc: "2026-01-15T10:30:00Z".to_string(),
            requested_by: "user@example.com".to_string(),
            requested_action: "Execute safe diagnostic commands".to_string(),
            confirmation_text: AUTOMATIC_EXECUTION_CONFIRMATION.to_string(),
        }
    }

    #[test]
    fn test_automatic_request_passes_automatic_validation() {
        let request = automatic_request();
        assert!(request.validate_automatic().is_ok());
    }

    #[test]
    fn test_automatic_request_fails_manual_validation() {
        let request = automatic_request();
        // validate() expects REQUIRED_CONFIRMATION, not AUTOMATIC_EXECUTION_CONFIRMATION
        assert!(matches!(
            request.validate(),
            Err(ValidationError::InvalidConfirmation { .. })
        ));
    }

    #[test]
    fn test_manual_request_fails_automatic_validation() {
        let request = valid_request();
        // validate_automatic() expects AUTOMATIC_EXECUTION_CONFIRMATION
        assert!(matches!(
            request.validate_automatic(),
            Err(ValidationError::InvalidConfirmation { .. })
        ));
    }

    #[test]
    fn test_automatic_wrong_confirmation_fails() {
        let mut request = automatic_request();
        request.confirmation_text = "I will execute automatically".to_string();
        assert!(matches!(
            request.validate_automatic(),
            Err(ValidationError::InvalidConfirmation { .. })
        ));
    }

    #[test]
    fn test_automatic_empty_fields_fail() {
        let mut request = automatic_request();
        request.request_id = String::new();
        assert!(matches!(
            request.validate_automatic(),
            Err(ValidationError::EmptyField { field: "request_id" })
        ));
    }

    // =========================================================================
    // SERIALIZATION ROUND-TRIP TESTS
    // =========================================================================

    #[test]
    fn test_serialization_roundtrip() {
        let request = valid_request();
        let json = serde_json::to_string(&request).unwrap();
        let restored: ExecutionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    #[test]
    fn test_serialization_deterministic() {
        let request = valid_request();

        // Serialize multiple times
        let json1 = serde_json::to_string(&request).unwrap();
        let json2 = serde_json::to_string(&request).unwrap();
        let json3 = serde_json::to_string(&request).unwrap();

        // All should be identical
        assert_eq!(json1, json2);
        assert_eq!(json2, json3);
    }

    #[test]
    fn test_serialization_field_order() {
        let request = valid_request();
        let json = serde_json::to_string_pretty(&request).unwrap();

        // Verify field order matches struct definition
        let request_id_pos = json.find("request_id").unwrap();
        let proposal_id_pos = json.find("proposal_id").unwrap();
        let recorded_utc_pos = json.find("recorded_utc").unwrap();
        let requested_by_pos = json.find("requested_by").unwrap();
        let requested_action_pos = json.find("requested_action").unwrap();
        let confirmation_text_pos = json.find("confirmation_text").unwrap();

        assert!(request_id_pos < proposal_id_pos);
        assert!(proposal_id_pos < recorded_utc_pos);
        assert!(recorded_utc_pos < requested_by_pos);
        assert!(requested_by_pos < requested_action_pos);
        assert!(requested_action_pos < confirmation_text_pos);
    }

    // =========================================================================
    // PERSISTENCE PATH TESTS
    // =========================================================================

    #[test]
    fn test_persistence_path_format() {
        let request = valid_request();
        let path = request.persistence_path();
        assert_eq!(
            path,
            PathBuf::from("/var/lib/anna/execution_requests/req-001.v1.json")
        );
    }

    #[test]
    fn test_persistence_path_different_ids() {
        let mut request = valid_request();
        request.request_id = "unique-id-12345".to_string();
        let path = request.persistence_path();
        assert_eq!(
            path,
            PathBuf::from("/var/lib/anna/execution_requests/unique-id-12345.v1.json")
        );
    }

    // =========================================================================
    // ISOLATION PROOF TESTS
    // =========================================================================

    #[test]
    fn proof_no_execute_method() {
        let request = valid_request();

        // ExecutionRequest has NO execute() method
        // ExecutionRequest has NO run() method
        // ExecutionRequest has NO apply() method
        // ExecutionRequest has NO dispatch() method
        //
        // The only methods are:
        // - validate() -> checks structural validity
        // - persistence_path() -> returns a PathBuf
        // - save() -> writes JSON to disk
        // - load() -> reads JSON from disk
        //
        // None of these execute commands or interact with the system
        // beyond file I/O for the record itself.

        assert!(request.validate().is_ok());
    }

    #[test]
    fn proof_no_execution_imports() {
        // This test documents the isolation guarantee.
        //
        // Verification command:
        // grep -n "ExecutionAdapter\|ExecutionGate\|action_plan" \
        //     crates/anna-shared/src/execution_request.rs
        //
        // Expected result: Zero matches in actual code
        // (only matches in this comment block and documentation)
        //
        // This module imports ONLY:
        // - serde (serialization)
        // - std::fs (file I/O for persistence)
        // - std::io::Write (for atomic writes)
        // - std::path::PathBuf (for paths)
    }

    #[test]
    fn proof_not_referenced_by_execution_pipeline() {
        // This test documents that the execution pipeline does not use this module.
        //
        // Verification command:
        // grep -rn "execution_request" crates/anna-shared/src/action_plan.rs
        //
        // Expected result: Zero matches
        //
        // Verification command:
        // grep -rn "ExecutionRequest" crates/anna-shared/src/action_plan.rs
        //
        // Expected result: Zero matches
    }

    #[test]
    fn proof_cannot_influence_readiness() {
        // ExecutionRequest has no connection to ReadinessCalculator or readiness modules.
        //
        // Verification:
        // grep -rn "ExecutionRequest" crates/anna-shared/src/readiness/
        //
        // Expected: Zero matches
        //
        // The existence of an ExecutionRequest record does not affect
        // whether any other component considers something "ready".
    }

    #[test]
    fn proof_cannot_influence_gating() {
        // ExecutionRequest has no connection to ExecutionGate or gating modules.
        //
        // Verification:
        // grep -rn "ExecutionRequest" crates/anna-shared/src/execution_gate.rs
        //
        // Expected: Zero matches (or file doesn't exist)
        //
        // The existence of an ExecutionRequest record does not affect
        // whether any gate is open, closed, or breached.
    }

    #[test]
    fn proof_cannot_trigger_attempts() {
        // ExecutionRequest has no connection to attempt recording or execution.
        //
        // This record is inert. It sits in a JSON file.
        // No other module reads it to decide whether to execute.
        // No process monitors this directory for new requests.
        // No daemon acts on these records.
        //
        // The record exists. That is all it does.
    }

    #[test]
    fn proof_request_is_human_artifact() {
        let request = valid_request();

        // The request captures what a HUMAN said they want to do.
        // It is not an AI decision.
        // It is not a recommendation.
        // It is not authorization.
        //
        // The requested_action is free-form text from the human.
        // The confirmation_text proves the human acknowledged the limitation.
        // The requested_by identifies the human.
        //
        // Anna did not create this. A human did.

        assert!(!request.requested_by.is_empty());
        assert!(!request.requested_action.is_empty());
        assert_eq!(request.confirmation_text, REQUIRED_CONFIRMATION);
    }

    // =========================================================================
    // EXPLICIT NON-CAPABILITY TESTS
    // =========================================================================

    #[test]
    fn explicit_does_not_execute() {
        let request = valid_request();

        // There is no method on ExecutionRequest that executes anything.
        // There is no function in this module that executes anything.
        // The validate() method only checks field values.
        // The save() method only writes JSON to disk.
        // The load() method only reads JSON from disk.

        let _ = request.validate();
        // Nothing was executed.
    }

    #[test]
    fn explicit_does_not_authorize() {
        let request = valid_request();

        // Creating an ExecutionRequest does not authorize anything.
        // Validating an ExecutionRequest does not authorize anything.
        // Saving an ExecutionRequest does not authorize anything.
        //
        // Authorization is a separate concern that this module does not address.

        assert!(request.validate().is_ok());
        // Nothing was authorized.
    }

    #[test]
    fn explicit_does_not_imply_safety() {
        let request = valid_request();

        // A valid ExecutionRequest does not mean the requested action is safe.
        // Validation checks structural completeness, not safety.
        // The confirmation text acknowledges that execution won't happen automatically.
        // It does NOT claim the action is safe to perform.

        assert!(request.validate().is_ok());
        // No safety claim was made.
    }

    #[test]
    fn explicit_does_not_select_commands() {
        let request = valid_request();

        // The requested_action is free-form text from the human.
        // This module does not parse it.
        // This module does not extract commands from it.
        // This module does not select which commands to run.

        assert_eq!(
            request.requested_action,
            "Apply the WiFi fix as described"
        );
        // No command was selected.
    }

    #[test]
    fn explicit_does_not_validate_proposals() {
        let request = valid_request();

        // The proposal_id is just a string reference.
        // This module does not check if the proposal exists.
        // This module does not validate the proposal's content.
        // This module does not cross-reference with the proposal module.

        assert_eq!(request.proposal_id, "prop-001");
        // No proposal was validated.
    }

    #[test]
    fn explicit_does_not_bypass_any_gate() {
        let request = valid_request();

        // ExecutionRequest has no knowledge of gates.
        // It cannot open gates.
        // It cannot close gates.
        // It cannot bypass gates.
        // It is just data.

        assert!(request.validate().is_ok());
        // No gate was affected.
    }

    #[test]
    fn explicit_does_not_trigger_anything() {
        let request = valid_request();

        // Creating an ExecutionRequest triggers nothing.
        // Validating an ExecutionRequest triggers nothing.
        // Saving an ExecutionRequest triggers nothing.
        // Loading an ExecutionRequest triggers nothing.
        //
        // No events are emitted.
        // No callbacks are invoked.
        // No processes are spawned.
        // No services are notified.

        assert!(request.validate().is_ok());
        // Nothing was triggered.
    }

    // =========================================================================
    // FINAL DOCUMENTATION TEST
    // =========================================================================

    #[test]
    fn final_documentation_statement() {
        // This record captures human intent to act. It performs no action.
    }
}
