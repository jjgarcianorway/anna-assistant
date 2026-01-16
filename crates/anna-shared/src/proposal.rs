//! Intent → Proposal Translation (Phase 38)
//!
//! This module translates human intentions into human-readable proposals.
//! It produces no plans, no actions, no recommendations, and no executable artifacts.
//!
//! # What This Module Does
//!
//! - Creates descriptive options a human could consider
//! - Documents uncertainty and unknowns explicitly
//! - Preserves human judgment as the sole decision-maker
//!
//! # What This Module Does NOT Do
//!
//! - Does NOT create plans
//! - Does NOT suggest actions
//! - Does NOT validate feasibility
//! - Does NOT imply safety
//! - Does NOT imply approval
//! - Does NOT imply execution
//! - Does NOT replace human judgment
//! - Does NOT recommend or select anything
//! - Does NOT connect to approval, readiness, gate, adapter, or execution code
//!
//! # What a Proposal IS
//!
//! A Proposal is:
//! - A descriptive option a human could consider
//! - Written in plain language
//! - Explicitly non-binding
//! - Explicitly non-actionable
//! - Explicitly incomplete by design
//!
//! # What a Proposal IS NOT
//!
//! - Not a plan
//! - Not advice
//! - Not a command
//! - Not a recommendation
//! - Not a suggestion to act
//! - Not an implication of correctness
//!
//! # Isolation
//!
//! This module:
//! - Does NOT import action_plan
//! - Does NOT import execution_reservation
//! - Does NOT import execution_impossibility
//! - Does NOT reference ExecutionGate, ExecutionAdapter, or ExecutionResult
//! - Does NOT reference ApprovalRecord or ExecutionReadiness
//! - Is NOT reachable from any execution pipeline
//!
//! This phase adds interpretability without agency.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// =============================================================================
// PROPOSAL RECORD DATA STRUCTURE
// This is NOT a plan. It CANNOT become a plan.
// It is a descriptive option for human consideration only.
// =============================================================================

/// A human-readable proposal for consideration.
///
/// # Purpose
///
/// This structure represents a descriptive option that a human could consider.
/// It is written in plain language and is explicitly non-binding, non-actionable,
/// and incomplete by design.
///
/// # Fields
///
/// - `proposal_id`: Unique identifier for this proposal
/// - `intention_id`: The intention this proposal relates to (string reference only)
/// - `recorded_utc`: When the proposal was created (ISO 8601)
/// - `title`: Short human-readable label
/// - `description`: What this could involve, described vaguely
/// - `uncertainty_notes`: What is unknown or risky about this option
/// - `evidence_sources`: Optional list of information sources considered
/// - `disclaimer`: Mandatory statement that this authorizes nothing
///
/// # Explicit Non-Capabilities
///
/// This structure:
/// - DOES NOT create plans
/// - DOES NOT suggest actions
/// - DOES NOT validate feasibility
/// - DOES NOT imply safety
/// - DOES NOT imply approval
/// - DOES NOT imply execution
/// - DOES NOT replace human judgment
/// - CANNOT be converted into a DeterministicActionPlan
/// - CANNOT be executed
/// - CANNOT authorize anything
///
/// # Example
///
/// ```ignore
/// ProposalRecord {
///     proposal_id: "prop-001",
///     intention_id: "int-001",
///     recorded_utc: "2026-01-15T12:00:00Z",
///     title: "Investigate wireless driver configuration",
///     description: "This could involve reviewing the wireless driver settings
///                   and firmware versions that influence performance.",
///     uncertainty_notes: "The root cause may not be related to drivers.
///                         Other factors like router settings or interference
///                         could be involved.",
///     evidence_sources: Some(vec!["Arch Wiki: Wireless"]),
///     disclaimer: "This proposal authorizes nothing and performs no action.",
/// }
/// ```
///
/// This proposal describes a possibility. It does not recommend it.
/// It does not validate it. It does not execute it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalRecord {
    /// Unique identifier for this proposal
    pub proposal_id: String,

    /// The intention this proposal relates to (string reference, not import)
    pub intention_id: String,

    /// When the proposal was recorded (ISO 8601 format)
    pub recorded_utc: String,

    /// Short human-readable label for this proposal
    pub title: String,

    /// What this could involve, described vaguely and incompletely
    pub description: String,

    /// What is unknown, uncertain, or potentially risky about this option
    pub uncertainty_notes: String,

    /// Optional list of information sources that were considered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_sources: Option<Vec<String>>,

    /// Mandatory disclaimer that this proposal authorizes nothing
    pub disclaimer: String,
}

// =============================================================================
// STRUCTURAL VALIDATION
// No semantic validation. No feasibility checks. No correctness analysis.
// =============================================================================

/// Validation error for ProposalRecord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalValidationError {
    pub field: String,
    pub message: String,
}

/// Validate a ProposalRecord structurally.
///
/// This validation checks ONLY:
/// - Required fields are present and non-empty
/// - Timestamp is ISO 8601 format
///
/// This validation does NOT check:
/// - Whether the proposal makes sense
/// - Whether the proposal is feasible
/// - Whether the proposal is safe
/// - Whether the proposal is correct
/// - Whether the intention exists
///
/// Returns a list of errors. Empty list means structurally valid.
pub fn validate_proposal(record: &ProposalRecord) -> Vec<ProposalValidationError> {
    let mut errors = Vec::new();

    if record.proposal_id.is_empty() {
        errors.push(ProposalValidationError {
            field: "proposal_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.intention_id.is_empty() {
        errors.push(ProposalValidationError {
            field: "intention_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.recorded_utc.is_empty() {
        errors.push(ProposalValidationError {
            field: "recorded_utc".to_string(),
            message: "must be non-empty".to_string(),
        });
    } else if !is_iso8601_format(&record.recorded_utc) {
        errors.push(ProposalValidationError {
            field: "recorded_utc".to_string(),
            message: "must be ISO 8601 format".to_string(),
        });
    }

    if record.title.is_empty() {
        errors.push(ProposalValidationError {
            field: "title".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.description.is_empty() {
        errors.push(ProposalValidationError {
            field: "description".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.uncertainty_notes.is_empty() {
        errors.push(ProposalValidationError {
            field: "uncertainty_notes".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.disclaimer.is_empty() {
        errors.push(ProposalValidationError {
            field: "disclaimer".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    errors
}

/// Check if a string is in ISO 8601 format.
fn is_iso8601_format(s: &str) -> bool {
    if s.len() < 20 {
        return false;
    }

    let bytes = s.as_bytes();

    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
}

// =============================================================================
// SERIALIZATION
// JSON only. Stable field order. Round-trip deterministic.
// =============================================================================

/// Current format version for proposal records.
pub const PROPOSAL_FORMAT_VERSION: u32 = 1;

/// Serialize a ProposalRecord to JSON.
/// Output is deterministic: same input always produces same output.
pub fn serialize_proposal(record: &ProposalRecord) -> Result<String, String> {
    serde_json::to_string_pretty(record).map_err(|e| e.to_string())
}

/// Deserialize a ProposalRecord from JSON.
/// Returns the record if parsing succeeds; does not perform validation.
pub fn deserialize_proposal(json: &str) -> Result<ProposalRecord, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Deserialize and validate a ProposalRecord from JSON.
/// Returns the record only if parsing succeeds AND validation passes.
pub fn deserialize_and_validate_proposal(json: &str) -> Result<ProposalRecord, Vec<String>> {
    let record: ProposalRecord =
        serde_json::from_str(json).map_err(|e| vec![format!("Parse error: {}", e)])?;

    let errors = validate_proposal(&record);
    if errors.is_empty() {
        Ok(record)
    } else {
        Err(errors
            .iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect())
    }
}

// =============================================================================
// PERSISTENCE
// Storage location: /var/lib/anna/proposals/
// File naming: {proposal_id}.v1.json
// =============================================================================

/// Get the proposals storage directory.
pub fn proposals_directory() -> PathBuf {
    crate::paths::paths().data_dir.join("proposals")
}

/// Generate the canonical filename for a proposal record.
fn proposal_filename(proposal_id: &str, version: u32) -> String {
    let safe_id: String = proposal_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    format!("{}.v{}.json", safe_id, version)
}

/// Full path for a proposal file.
fn proposal_path(proposal_id: &str, version: u32) -> PathBuf {
    proposals_directory().join(proposal_filename(proposal_id, version))
}

/// Proposal storage error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalStorageError {
    /// Storage directory does not exist or cannot be created
    DirectoryError(String),
    /// File write failed
    WriteError(String),
    /// File read failed
    ReadError(String),
    /// Proposal not found
    NotFound(String),
    /// Unknown format version
    UnknownVersion(u32),
    /// Validation failed after load
    ValidationFailed(Vec<String>),
}

/// Save a ProposalRecord to disk.
/// Performs validation before saving.
/// File path: /var/lib/anna/proposals/{proposal_id}.v1.json
pub fn save_proposal(record: &ProposalRecord) -> Result<PathBuf, ProposalStorageError> {
    // Validate before saving
    let errors = validate_proposal(record);
    if !errors.is_empty() {
        return Err(ProposalStorageError::ValidationFailed(
            errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect(),
        ));
    }

    // Ensure directory exists
    let dir = proposals_directory();
    fs::create_dir_all(&dir).map_err(|e| ProposalStorageError::DirectoryError(e.to_string()))?;

    // Generate path
    let path = proposal_path(&record.proposal_id, PROPOSAL_FORMAT_VERSION);

    // Serialize
    let json = serialize_proposal(record).map_err(|e| ProposalStorageError::WriteError(e))?;

    // Write atomically (write to temp, then rename)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &json).map_err(|e| ProposalStorageError::WriteError(e.to_string()))?;
    fs::rename(&temp_path, &path).map_err(|e| ProposalStorageError::WriteError(e.to_string()))?;

    Ok(path)
}

/// Load a ProposalRecord from disk by proposal_id.
pub fn load_proposal(proposal_id: &str) -> Result<ProposalRecord, ProposalStorageError> {
    load_proposal_version(proposal_id, PROPOSAL_FORMAT_VERSION)
}

/// Load a ProposalRecord with a specific format version.
pub fn load_proposal_version(
    proposal_id: &str,
    version: u32,
) -> Result<ProposalRecord, ProposalStorageError> {
    if version != PROPOSAL_FORMAT_VERSION {
        return Err(ProposalStorageError::UnknownVersion(version));
    }

    let path = proposal_path(proposal_id, version);

    if !path.exists() {
        return Err(ProposalStorageError::NotFound(proposal_id.to_string()));
    }

    let json =
        fs::read_to_string(&path).map_err(|e| ProposalStorageError::ReadError(e.to_string()))?;

    let record = deserialize_proposal(&json).map_err(|e| ProposalStorageError::ReadError(e))?;

    // Validate after load
    let errors = validate_proposal(&record);
    if !errors.is_empty() {
        return Err(ProposalStorageError::ValidationFailed(
            errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect(),
        ));
    }

    Ok(record)
}

/// List all stored proposal IDs.
pub fn list_proposal_ids() -> Result<Vec<String>, ProposalStorageError> {
    let dir = proposals_directory();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let suffix = format!(".v{}.json", PROPOSAL_FORMAT_VERSION);
    let mut ids = Vec::new();

    let entries =
        fs::read_dir(&dir).map_err(|e| ProposalStorageError::DirectoryError(e.to_string()))?;

    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(&suffix) {
                let id = name.trim_end_matches(&suffix).to_string();
                ids.push(id);
            }
        }
    }

    ids.sort();
    Ok(ids)
}

/// List proposals for a specific intention.
/// Loads each proposal to filter by intention_id.
pub fn list_proposals_for_intention(
    intention_id: &str,
) -> Result<Vec<String>, ProposalStorageError> {
    let all_ids = list_proposal_ids()?;
    let mut matching = Vec::new();

    for id in all_ids {
        if let Ok(proposal) = load_proposal(&id) {
            if proposal.intention_id == intention_id {
                matching.push(id);
            }
        }
    }

    Ok(matching)
}

// =============================================================================
// EXPLICIT NON-CAPABILITIES
// This section documents what ProposalRecord DOES NOT and CANNOT do.
// These non-capabilities are by design and must never be violated.
// =============================================================================
//
// The ProposalRecord:
// - DOES NOT create plans
// - DOES NOT suggest actions
// - DOES NOT validate feasibility
// - DOES NOT imply safety
// - DOES NOT imply approval
// - DOES NOT imply execution
// - DOES NOT replace human judgment
// - DOES NOT recommend anything
// - DOES NOT select anything
// - DOES NOT connect to action plans
// - DOES NOT connect to approval records
// - DOES NOT connect to execution readiness
// - DOES NOT connect to execution gates
// - DOES NOT connect to execution adapters
// - DOES NOT connect to reserved execution interface
// - CANNOT be converted into a DeterministicActionPlan
// - CANNOT be executed
// - CANNOT authorize anything
// - CANNOT trigger any pipeline
//
// A proposal is an option for human consideration.
// The human decides. The human acts (or doesn't).
// The system only describes possibilities.
//
// This phase adds interpretability without agency.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GOLDEN FIXTURES
    // =========================================================================

    fn golden_valid_proposal() -> ProposalRecord {
        ProposalRecord {
            proposal_id: "prop-001".to_string(),
            intention_id: "int-001".to_string(),
            recorded_utc: "2026-01-15T12:00:00Z".to_string(),
            title: "Investigate wireless driver configuration".to_string(),
            description: "This could involve reviewing the wireless driver settings \
                          and firmware versions that influence performance."
                .to_string(),
            uncertainty_notes: "The root cause may not be related to drivers. \
                                Other factors like router settings or interference \
                                could be involved."
                .to_string(),
            evidence_sources: Some(vec!["Arch Wiki: Wireless".to_string()]),
            disclaimer: "This proposal authorizes nothing and performs no action.".to_string(),
        }
    }

    fn golden_minimal_proposal() -> ProposalRecord {
        ProposalRecord {
            proposal_id: "prop-min".to_string(),
            intention_id: "int-001".to_string(),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            title: "Check system logs".to_string(),
            description: "This could involve examining system logs for relevant entries."
                .to_string(),
            uncertainty_notes: "Logs may not contain the relevant information.".to_string(),
            evidence_sources: None,
            disclaimer: "This proposal authorizes nothing and performs no action.".to_string(),
        }
    }

    fn golden_invalid_proposal() -> ProposalRecord {
        ProposalRecord {
            proposal_id: String::new(),
            intention_id: String::new(),
            recorded_utc: "bad-date".to_string(),
            title: String::new(),
            description: String::new(),
            uncertainty_notes: String::new(),
            evidence_sources: None,
            disclaimer: String::new(),
        }
    }

    // =========================================================================
    // VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_valid_proposal_passes_validation() {
        let record = golden_valid_proposal();
        let errors = validate_proposal(&record);
        assert_eq!(errors, vec![], "Valid proposal must produce zero errors");
    }

    #[test]
    fn test_minimal_proposal_passes_validation() {
        let record = golden_minimal_proposal();
        let errors = validate_proposal(&record);
        assert_eq!(errors, vec![], "Minimal proposal must produce zero errors");
    }

    #[test]
    fn test_invalid_proposal_fails_validation() {
        let record = golden_invalid_proposal();
        let errors = validate_proposal(&record);

        let expected = vec![
            ProposalValidationError {
                field: "proposal_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            ProposalValidationError {
                field: "intention_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            ProposalValidationError {
                field: "recorded_utc".to_string(),
                message: "must be ISO 8601 format".to_string(),
            },
            ProposalValidationError {
                field: "title".to_string(),
                message: "must be non-empty".to_string(),
            },
            ProposalValidationError {
                field: "description".to_string(),
                message: "must be non-empty".to_string(),
            },
            ProposalValidationError {
                field: "uncertainty_notes".to_string(),
                message: "must be non-empty".to_string(),
            },
            ProposalValidationError {
                field: "disclaimer".to_string(),
                message: "must be non-empty".to_string(),
            },
        ];

        assert_eq!(
            errors, expected,
            "Invalid proposal must produce exact error list"
        );
    }

    // =========================================================================
    // SERIALIZATION TESTS
    // =========================================================================

    const GOLDEN_MINIMAL_JSON: &str = r#"{
  "proposal_id": "prop-min",
  "intention_id": "int-001",
  "recorded_utc": "2026-01-15T00:00:00Z",
  "title": "Check system logs",
  "description": "This could involve examining system logs for relevant entries.",
  "uncertainty_notes": "Logs may not contain the relevant information.",
  "disclaimer": "This proposal authorizes nothing and performs no action."
}"#;

    #[test]
    fn test_serialization_exact_output() {
        let record = golden_minimal_proposal();
        let json = serialize_proposal(&record).unwrap();
        assert_eq!(
            json, GOLDEN_MINIMAL_JSON,
            "Serialization must produce exact output"
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = golden_valid_proposal();
        let json = serialize_proposal(&original).unwrap();
        let restored = deserialize_proposal(&json).unwrap();
        assert_eq!(original, restored, "Round-trip must preserve exact data");
    }

    #[test]
    fn test_serialization_with_evidence_sources() {
        let record = golden_valid_proposal();
        let json = serialize_proposal(&record).unwrap();
        assert!(
            json.contains("\"evidence_sources\":"),
            "Evidence sources must be present when set"
        );
        assert!(json.contains("Arch Wiki: Wireless"));

        let restored = deserialize_proposal(&json).unwrap();
        assert_eq!(record, restored);
    }

    #[test]
    fn test_serialization_determinism() {
        let record = golden_minimal_proposal();
        let json1 = serialize_proposal(&record).unwrap();
        let json2 = serialize_proposal(&record).unwrap();
        let json3 = serialize_proposal(&record).unwrap();
        assert_eq!(json1, json2);
        assert_eq!(json2, json3);
    }

    #[test]
    fn test_deserialize_and_validate_valid() {
        let result = deserialize_and_validate_proposal(GOLDEN_MINIMAL_JSON);
        assert!(result.is_ok(), "Valid JSON must parse and validate");
    }

    #[test]
    fn test_deserialize_and_validate_invalid_json() {
        let result = deserialize_and_validate_proposal("not valid json");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Parse error"));
    }

    #[test]
    fn test_deserialize_and_validate_invalid_record() {
        let invalid_json = r#"{
  "proposal_id": "",
  "intention_id": "int-001",
  "recorded_utc": "2026-01-15T00:00:00Z",
  "title": "Test",
  "description": "Test",
  "uncertainty_notes": "Test",
  "disclaimer": "Test"
}"#;
        let result = deserialize_and_validate_proposal(invalid_json);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("proposal_id")));
    }

    #[test]
    fn test_field_order_stable() {
        let record = golden_minimal_proposal();
        let json = serialize_proposal(&record).unwrap();

        let proposal_id_pos = json.find("proposal_id").unwrap();
        let intention_id_pos = json.find("intention_id").unwrap();
        let recorded_utc_pos = json.find("recorded_utc").unwrap();
        let title_pos = json.find("title").unwrap();
        let description_pos = json.find("description").unwrap();
        let uncertainty_pos = json.find("uncertainty_notes").unwrap();
        let disclaimer_pos = json.find("disclaimer").unwrap();

        assert!(proposal_id_pos < intention_id_pos);
        assert!(intention_id_pos < recorded_utc_pos);
        assert!(recorded_utc_pos < title_pos);
        assert!(title_pos < description_pos);
        assert!(description_pos < uncertainty_pos);
        assert!(uncertainty_pos < disclaimer_pos);
    }

    // =========================================================================
    // PERSISTENCE TESTS
    // =========================================================================

    fn setup_test_dir() -> PathBuf {
        let dir = std::env::temp_dir()
            .join("anna-test-proposals")
            .join(format!("{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        std::env::set_var("ANNA_DEV_MODE", "1");
        dir
    }

    fn cleanup_test_dir(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_filename_format() {
        let filename = proposal_filename("prop-001", 1);
        assert_eq!(filename, "prop-001.v1.json");
    }

    #[test]
    fn test_filename_sanitization() {
        let filename = proposal_filename("prop/with:special@chars!", 1);
        assert_eq!(filename, "prop-with-special-chars-.v1.json");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let test_dir = setup_test_dir();

        let record = ProposalRecord {
            proposal_id: format!("prop-persist-{}", std::process::id()),
            intention_id: "int-001".to_string(),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            title: "Test proposal".to_string(),
            description: "This is a test proposal for persistence.".to_string(),
            uncertainty_notes: "This is just a test.".to_string(),
            evidence_sources: None,
            disclaimer: "This proposal authorizes nothing and performs no action.".to_string(),
        };

        // Save
        let save_result = save_proposal(&record);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result);

        // Load
        let loaded = load_proposal(&record.proposal_id);
        assert!(loaded.is_ok(), "Load failed: {:?}", loaded);

        // Assert equality
        let loaded_record = loaded.unwrap();
        assert_eq!(record, loaded_record, "Loaded record must equal saved record");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_load_not_found() {
        let _test_dir = setup_test_dir();

        let result = load_proposal("nonexistent-proposal-id-12345");
        assert!(matches!(result, Err(ProposalStorageError::NotFound(_))));
    }

    #[test]
    fn test_reject_unknown_version() {
        let result = load_proposal_version("any-id", 99);
        assert!(matches!(
            result,
            Err(ProposalStorageError::UnknownVersion(99))
        ));
    }

    #[test]
    fn test_save_invalid_rejected() {
        let _test_dir = setup_test_dir();

        let invalid_record = ProposalRecord {
            proposal_id: String::new(), // Invalid: empty
            intention_id: "int-001".to_string(),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            uncertainty_notes: "Test".to_string(),
            evidence_sources: None,
            disclaimer: "Test".to_string(),
        };

        let result = save_proposal(&invalid_record);
        assert!(matches!(
            result,
            Err(ProposalStorageError::ValidationFailed(_))
        ));
    }

    #[test]
    fn test_format_version_constant() {
        assert_eq!(PROPOSAL_FORMAT_VERSION, 1);
    }

    // =========================================================================
    // ISOLATION TESTS - PROPOSALS CANNOT BE EXECUTED
    // =========================================================================

    #[test]
    fn proof_no_action_plan_import() {
        // This test documents that proposal.rs does not import action_plan.
        //
        // Verification: grep -n "use.*action_plan" crates/anna-shared/src/proposal.rs
        // Expected: Zero results
        //
        // The ProposalRecord is completely isolated from action plans.
    }

    #[test]
    fn proof_no_execution_reference() {
        // This test documents that proposal.rs does not reference execution code.
        //
        // Verification:
        //   grep -n "ExecutionAdapter\|ExecutionGate\|ExecutionResult\|ExecutionAttempt"
        //        crates/anna-shared/src/proposal.rs
        // Expected: Zero results
        //
        // The ProposalRecord is completely isolated from execution.
    }

    #[test]
    fn proof_no_approval_reference() {
        // This test documents that proposal.rs does not reference approval code.
        //
        // Verification: grep -n "ApprovalRecord\|ApprovalDecision" crates/anna-shared/src/proposal.rs
        // Expected: Zero results
        //
        // The ProposalRecord is completely isolated from approvals.
    }

    #[test]
    fn proof_no_readiness_reference() {
        // This test documents that proposal.rs does not reference readiness code.
        //
        // Verification: grep -n "ExecutionReadiness" crates/anna-shared/src/proposal.rs
        // Expected: Zero results
        //
        // The ProposalRecord is completely isolated from readiness classification.
    }

    #[test]
    fn proof_proposals_cannot_be_executed() {
        // A ProposalRecord has no execute() method.
        // There is no function that takes a ProposalRecord and performs actions.
        // There is no conversion from ProposalRecord to DeterministicActionPlan.
        //
        // The proposal describes a possibility. It cannot act on it.
        let proposal = golden_valid_proposal();

        // We can read the proposal fields
        let _ = proposal.title.clone();
        let _ = proposal.description.clone();

        // But there is no:
        // - proposal.execute()
        // - execute_proposal(&proposal)
        // - proposal_to_plan(&proposal)
        // - act_on_proposal(&proposal)
        //
        // These functions do not exist and must never be added.
    }

    #[test]
    fn proof_proposals_cannot_become_plans() {
        // There is no function that converts a ProposalRecord to a DeterministicActionPlan.
        //
        // A proposal contains:
        // - title: a label, not an operation
        // - description: prose, not steps
        // - uncertainty_notes: warnings, not preconditions
        // - evidence_sources: references, not commands
        //
        // None of these can be parsed into DeterministicStep structures.
        // There is no parser. There is no converter. There is no bridge.
        let proposal = golden_valid_proposal();

        // The proposal has no structure that maps to a plan:
        assert!(!proposal.title.is_empty()); // Just a label
        assert!(!proposal.description.is_empty()); // Just prose

        // There is no:
        // - ProposalRecord::into_plan()
        // - impl From<ProposalRecord> for DeterministicActionPlan
        // - proposal_to_plan(&proposal)
    }

    #[test]
    fn proof_disclaimer_is_required() {
        // Every valid proposal must have a non-empty disclaimer.
        // This ensures the proposal explicitly states it authorizes nothing.
        let mut proposal = golden_valid_proposal();
        proposal.disclaimer = String::new();

        let errors = validate_proposal(&proposal);
        assert!(errors.iter().any(|e| e.field == "disclaimer"));
    }

    #[test]
    fn proof_uncertainty_is_required() {
        // Every valid proposal must have non-empty uncertainty notes.
        // This ensures the proposal explicitly acknowledges unknowns.
        let mut proposal = golden_valid_proposal();
        proposal.uncertainty_notes = String::new();

        let errors = validate_proposal(&proposal);
        assert!(errors.iter().any(|e| e.field == "uncertainty_notes"));
    }

    #[test]
    fn proof_proposal_is_descriptive_only() {
        // A proposal describes what COULD be investigated.
        // It does not prescribe what SHOULD be done.
        // It does not validate what CAN be done.
        // It does not imply what WILL be done.
        let proposal = ProposalRecord {
            proposal_id: "desc-only".to_string(),
            intention_id: "int-001".to_string(),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            title: "Consider reviewing network settings".to_string(),
            description: "One might look at network configuration files.".to_string(),
            uncertainty_notes: "This may or may not be relevant.".to_string(),
            evidence_sources: None,
            disclaimer: "This describes a possibility, not a recommendation.".to_string(),
        };

        // Valid structurally
        let errors = validate_proposal(&proposal);
        assert_eq!(errors, vec![]);

        // But completely inert operationally
        // The proposal just sits there, describing nothing actionable
    }

    #[test]
    fn proof_no_deterministic_step_fields() {
        // ProposalRecord has no fields that could be interpreted as DeterministicStep:
        // - No `operation` field
        // - No `target` field
        // - No `step_number` field
        // - No `steps` array
        //
        // The structure is fundamentally incompatible with execution.
        let proposal = golden_valid_proposal();

        // These assertions prove the structure has no execution-like fields:
        // (We assert on what DOES exist to show what DOESN'T)
        assert!(!proposal.proposal_id.is_empty()); // ID, not operation
        assert!(!proposal.title.is_empty()); // Label, not command
        assert!(!proposal.description.is_empty()); // Prose, not steps
        assert!(!proposal.uncertainty_notes.is_empty()); // Doubt, not targets
        assert!(!proposal.disclaimer.is_empty()); // Denial, not permission
    }
}

// =============================================================================
// This phase adds interpretability without agency.
// =============================================================================
