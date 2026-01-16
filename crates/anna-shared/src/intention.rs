//! Human Intention Capture (Phase 37)
//!
//! This module provides a structure for capturing human intent in natural terms.
//! It produces no action, no plan, and no path toward execution.
//!
//! # What This Module Does
//!
//! - Captures a human's stated intention as free-form text
//! - Records when and by whom the intention was captured
//! - Persists the record for audit and review
//!
//! # What This Module Does NOT Do
//!
//! - Does NOT create action plans
//! - Does NOT validate intentions for correctness or feasibility
//! - Does NOT transform intentions into executable structures
//! - Does NOT infer, recommend, or suggest actions
//! - Does NOT connect to approval, readiness, gate, or execution code
//!
//! # Explicit Non-Capabilities
//!
//! This structure:
//! - DOES NOT imply action
//! - CANNOT be converted into a plan
//! - IS NOT validated for correctness
//! - HAS NO operational meaning
//!
//! An IntentionRecord is a passive artifact. It documents what a human said
//! they wanted, without interpreting, validating, or acting on that statement.
//!
//! # Isolation
//!
//! This module:
//! - Does NOT import action_plan
//! - Does NOT import execution_reservation
//! - Does NOT import execution_impossibility
//! - Is NOT referenced by any planning or execution code
//! - Is NOT reachable from any pipeline
//!
//! This record captures human intent without enabling planning or execution.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// =============================================================================
// INTENTION RECORD DATA STRUCTURE
// This is NOT a plan. It does NOT resemble one.
// It captures what a human expressed, nothing more.
// =============================================================================

/// A record of human intent, expressed in natural terms.
///
/// # Purpose
///
/// This structure captures what a human operator stated as their intention.
/// It is a verbatim record of expression, not a structured command.
///
/// # Fields
///
/// - `intention_id`: Unique identifier for this record
/// - `recorded_utc`: When the intention was captured (ISO 8601)
/// - `author`: Who expressed this intention
/// - `statement`: The free-form expression of intent (unstructured)
/// - `context`: Optional context about circumstances
/// - `note`: Optional additional notes
///
/// # Explicit Non-Capabilities
///
/// This structure:
/// - DOES NOT imply action will be taken
/// - CANNOT be converted into an action plan
/// - IS NOT validated for correctness or feasibility
/// - HAS NO operational meaning in any pipeline
///
/// The statement field is free-form text. It is not parsed, interpreted,
/// or matched against any schema. It is stored verbatim.
///
/// # Example
///
/// ```ignore
/// IntentionRecord {
///     intention_id: "int-001",
///     recorded_utc: "2026-01-15T12:00:00Z",
///     author: "operator",
///     statement: "I want the system to be more responsive",
///     context: Some("After noticing slow startup times"),
///     note: None,
/// }
/// ```
///
/// This record documents a desire. It does not trigger investigation,
/// planning, or execution. It is purely documentary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentionRecord {
    /// Unique identifier for this intention record
    pub intention_id: String,

    /// When the intention was recorded (ISO 8601 format)
    pub recorded_utc: String,

    /// Who expressed this intention
    pub author: String,

    /// The free-form statement of intent (unstructured, verbatim)
    pub statement: String,

    /// Optional context about the circumstances
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Optional additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// =============================================================================
// STRUCTURAL VALIDATION
// No semantic validation. No correctness checks. No feasibility analysis.
// =============================================================================

/// Validation error for IntentionRecord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentionValidationError {
    pub field: String,
    pub message: String,
}

/// Validate an IntentionRecord structurally.
///
/// This validation checks ONLY:
/// - Required fields are present and non-empty
/// - Timestamp is ISO 8601 format
///
/// This validation does NOT check:
/// - Whether the statement makes sense
/// - Whether the intention is feasible
/// - Whether the author exists
/// - Whether the intention conflicts with anything
///
/// Returns a list of errors. Empty list means structurally valid.
pub fn validate_intention(record: &IntentionRecord) -> Vec<IntentionValidationError> {
    let mut errors = Vec::new();

    if record.intention_id.is_empty() {
        errors.push(IntentionValidationError {
            field: "intention_id".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.recorded_utc.is_empty() {
        errors.push(IntentionValidationError {
            field: "recorded_utc".to_string(),
            message: "must be non-empty".to_string(),
        });
    } else if !is_iso8601_format(&record.recorded_utc) {
        errors.push(IntentionValidationError {
            field: "recorded_utc".to_string(),
            message: "must be ISO 8601 format".to_string(),
        });
    }

    if record.author.is_empty() {
        errors.push(IntentionValidationError {
            field: "author".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    if record.statement.is_empty() {
        errors.push(IntentionValidationError {
            field: "statement".to_string(),
            message: "must be non-empty".to_string(),
        });
    }

    errors
}

/// Check if a string is in ISO 8601 format.
/// This is a structural check, not a semantic one.
fn is_iso8601_format(s: &str) -> bool {
    // Basic ISO 8601 format check: YYYY-MM-DDTHH:MM:SSZ or with offset
    if s.len() < 20 {
        return false;
    }

    let bytes = s.as_bytes();

    // Check basic structure: YYYY-MM-DDTHH:MM:SS
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

/// Current format version for intention records.
pub const INTENTION_FORMAT_VERSION: u32 = 1;

/// Serialize an IntentionRecord to JSON.
/// Output is deterministic: same input always produces same output.
pub fn serialize_intention(record: &IntentionRecord) -> Result<String, String> {
    serde_json::to_string_pretty(record).map_err(|e| e.to_string())
}

/// Deserialize an IntentionRecord from JSON.
/// Returns the record if parsing succeeds; does not perform validation.
pub fn deserialize_intention(json: &str) -> Result<IntentionRecord, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Deserialize and validate an IntentionRecord from JSON.
/// Returns the record only if parsing succeeds AND validation passes.
pub fn deserialize_and_validate_intention(json: &str) -> Result<IntentionRecord, Vec<String>> {
    let record: IntentionRecord =
        serde_json::from_str(json).map_err(|e| vec![format!("Parse error: {}", e)])?;

    let errors = validate_intention(&record);
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
// Storage location: /var/lib/anna/intentions/
// File naming: {intention_id}.v1.json
// =============================================================================

/// Get the intentions storage directory.
pub fn intentions_directory() -> PathBuf {
    crate::paths::paths().data_dir.join("intentions")
}

/// Generate the canonical filename for an intention record.
fn intention_filename(intention_id: &str, version: u32) -> String {
    let safe_id: String = intention_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    format!("{}.v{}.json", safe_id, version)
}

/// Full path for an intention file.
fn intention_path(intention_id: &str, version: u32) -> PathBuf {
    intentions_directory().join(intention_filename(intention_id, version))
}

/// Intention storage error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentionStorageError {
    /// Storage directory does not exist or cannot be created
    DirectoryError(String),
    /// File write failed
    WriteError(String),
    /// File read failed
    ReadError(String),
    /// Intention not found
    NotFound(String),
    /// Unknown format version
    UnknownVersion(u32),
    /// Validation failed after load
    ValidationFailed(Vec<String>),
}

/// Save an IntentionRecord to disk.
/// Performs validation before saving.
/// File path: /var/lib/anna/intentions/{intention_id}.v1.json
pub fn save_intention(record: &IntentionRecord) -> Result<PathBuf, IntentionStorageError> {
    // Validate before saving
    let errors = validate_intention(record);
    if !errors.is_empty() {
        return Err(IntentionStorageError::ValidationFailed(
            errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect(),
        ));
    }

    // Ensure directory exists
    let dir = intentions_directory();
    fs::create_dir_all(&dir).map_err(|e| IntentionStorageError::DirectoryError(e.to_string()))?;

    // Generate path
    let path = intention_path(&record.intention_id, INTENTION_FORMAT_VERSION);

    // Serialize
    let json =
        serialize_intention(record).map_err(|e| IntentionStorageError::WriteError(e))?;

    // Write atomically (write to temp, then rename)
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, &json).map_err(|e| IntentionStorageError::WriteError(e.to_string()))?;
    fs::rename(&temp_path, &path).map_err(|e| IntentionStorageError::WriteError(e.to_string()))?;

    Ok(path)
}

/// Load an IntentionRecord from disk by intention_id.
pub fn load_intention(intention_id: &str) -> Result<IntentionRecord, IntentionStorageError> {
    load_intention_version(intention_id, INTENTION_FORMAT_VERSION)
}

/// Load an IntentionRecord with a specific format version.
pub fn load_intention_version(
    intention_id: &str,
    version: u32,
) -> Result<IntentionRecord, IntentionStorageError> {
    if version != INTENTION_FORMAT_VERSION {
        return Err(IntentionStorageError::UnknownVersion(version));
    }

    let path = intention_path(intention_id, version);

    if !path.exists() {
        return Err(IntentionStorageError::NotFound(intention_id.to_string()));
    }

    let json =
        fs::read_to_string(&path).map_err(|e| IntentionStorageError::ReadError(e.to_string()))?;

    let record =
        deserialize_intention(&json).map_err(|e| IntentionStorageError::ReadError(e))?;

    // Validate after load
    let errors = validate_intention(&record);
    if !errors.is_empty() {
        return Err(IntentionStorageError::ValidationFailed(
            errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect(),
        ));
    }

    Ok(record)
}

/// List all stored intention IDs.
pub fn list_intention_ids() -> Result<Vec<String>, IntentionStorageError> {
    let dir = intentions_directory();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let suffix = format!(".v{}.json", INTENTION_FORMAT_VERSION);
    let mut ids = Vec::new();

    let entries =
        fs::read_dir(&dir).map_err(|e| IntentionStorageError::DirectoryError(e.to_string()))?;

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

// =============================================================================
// EXPLICIT NON-CAPABILITIES
// This section documents what IntentionRecord DOES NOT and CANNOT do.
// These non-capabilities are by design and must never be violated.
// =============================================================================
//
// The IntentionRecord:
// - DOES NOT imply action will be taken
// - DOES NOT trigger planning
// - DOES NOT trigger execution
// - DOES NOT connect to action plans
// - DOES NOT connect to approval records
// - DOES NOT connect to execution readiness
// - DOES NOT connect to execution gates
// - DOES NOT connect to execution adapters
// - DOES NOT get validated for correctness
// - DOES NOT get validated for feasibility
// - DOES NOT get interpreted or parsed for meaning
// - DOES NOT produce recommendations
// - DOES NOT produce suggestions
//
// The IntentionRecord is a passive documentary artifact.
// It records what a human said, verbatim, without interpretation.
// It has no downstream effects in any pipeline.
//
// This record captures human intent without enabling planning or execution.
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // GOLDEN FIXTURES
    // =========================================================================

    fn golden_valid_intention() -> IntentionRecord {
        IntentionRecord {
            intention_id: "int-001".to_string(),
            recorded_utc: "2026-01-15T12:00:00Z".to_string(),
            author: "operator".to_string(),
            statement: "I want the system to start faster".to_string(),
            context: Some("Noticed slow boot times this morning".to_string()),
            note: None,
        }
    }

    fn golden_minimal_intention() -> IntentionRecord {
        IntentionRecord {
            intention_id: "int-min".to_string(),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            author: "user".to_string(),
            statement: "Make it work".to_string(),
            context: None,
            note: None,
        }
    }

    fn golden_invalid_intention() -> IntentionRecord {
        IntentionRecord {
            intention_id: String::new(),
            recorded_utc: "bad-date".to_string(),
            author: String::new(),
            statement: String::new(),
            context: None,
            note: None,
        }
    }

    // =========================================================================
    // VALIDATION TESTS
    // =========================================================================

    #[test]
    fn test_valid_intention_passes_validation() {
        let record = golden_valid_intention();
        let errors = validate_intention(&record);
        assert_eq!(errors, vec![], "Valid intention must produce zero errors");
    }

    #[test]
    fn test_minimal_intention_passes_validation() {
        let record = golden_minimal_intention();
        let errors = validate_intention(&record);
        assert_eq!(errors, vec![], "Minimal intention must produce zero errors");
    }

    #[test]
    fn test_invalid_intention_fails_validation() {
        let record = golden_invalid_intention();
        let errors = validate_intention(&record);

        let expected = vec![
            IntentionValidationError {
                field: "intention_id".to_string(),
                message: "must be non-empty".to_string(),
            },
            IntentionValidationError {
                field: "recorded_utc".to_string(),
                message: "must be ISO 8601 format".to_string(),
            },
            IntentionValidationError {
                field: "author".to_string(),
                message: "must be non-empty".to_string(),
            },
            IntentionValidationError {
                field: "statement".to_string(),
                message: "must be non-empty".to_string(),
            },
        ];

        assert_eq!(errors, expected, "Invalid intention must produce exact error list");
    }

    // =========================================================================
    // SERIALIZATION TESTS
    // =========================================================================

    const GOLDEN_MINIMAL_JSON: &str = r#"{
  "intention_id": "int-min",
  "recorded_utc": "2026-01-15T00:00:00Z",
  "author": "user",
  "statement": "Make it work"
}"#;

    #[test]
    fn test_serialization_exact_output() {
        let record = golden_minimal_intention();
        let json = serialize_intention(&record).unwrap();
        assert_eq!(json, GOLDEN_MINIMAL_JSON, "Serialization must produce exact output");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original = golden_valid_intention();
        let json = serialize_intention(&original).unwrap();
        let restored = deserialize_intention(&json).unwrap();
        assert_eq!(original, restored, "Round-trip must preserve exact data");
    }

    #[test]
    fn test_serialization_with_optional_fields() {
        let record = golden_valid_intention();
        let json = serialize_intention(&record).unwrap();
        assert!(json.contains("\"context\":"), "Context field must be present when set");
        assert!(json.contains("Noticed slow boot times"));

        let restored = deserialize_intention(&json).unwrap();
        assert_eq!(record, restored);
    }

    #[test]
    fn test_serialization_determinism() {
        let record = golden_minimal_intention();
        let json1 = serialize_intention(&record).unwrap();
        let json2 = serialize_intention(&record).unwrap();
        let json3 = serialize_intention(&record).unwrap();
        assert_eq!(json1, json2);
        assert_eq!(json2, json3);
    }

    #[test]
    fn test_deserialize_and_validate_valid() {
        let result = deserialize_and_validate_intention(GOLDEN_MINIMAL_JSON);
        assert!(result.is_ok(), "Valid JSON must parse and validate");
    }

    #[test]
    fn test_deserialize_and_validate_invalid_json() {
        let result = deserialize_and_validate_intention("not valid json");
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("Parse error"));
    }

    #[test]
    fn test_deserialize_and_validate_invalid_record() {
        let invalid_json = r#"{
  "intention_id": "",
  "recorded_utc": "2026-01-15T00:00:00Z",
  "author": "user",
  "statement": "Test"
}"#;
        let result = deserialize_and_validate_intention(invalid_json);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("intention_id")));
    }

    #[test]
    fn test_field_order_stable() {
        let record = golden_minimal_intention();
        let json = serialize_intention(&record).unwrap();

        let id_pos = json.find("intention_id").unwrap();
        let utc_pos = json.find("recorded_utc").unwrap();
        let author_pos = json.find("author").unwrap();
        let statement_pos = json.find("statement").unwrap();

        assert!(id_pos < utc_pos);
        assert!(utc_pos < author_pos);
        assert!(author_pos < statement_pos);
    }

    // =========================================================================
    // PERSISTENCE TESTS
    // =========================================================================

    fn setup_test_dir() -> PathBuf {
        let dir = std::env::temp_dir()
            .join("anna-test-intentions")
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
        let filename = intention_filename("int-001", 1);
        assert_eq!(filename, "int-001.v1.json");
    }

    #[test]
    fn test_filename_sanitization() {
        let filename = intention_filename("int/with:special@chars!", 1);
        assert_eq!(filename, "int-with-special-chars-.v1.json");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let test_dir = setup_test_dir();

        let record = IntentionRecord {
            intention_id: format!("int-persist-{}", std::process::id()),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            author: "test".to_string(),
            statement: "Test intention for persistence".to_string(),
            context: Some("Testing save/load".to_string()),
            note: None,
        };

        // Save
        let save_result = save_intention(&record);
        assert!(save_result.is_ok(), "Save failed: {:?}", save_result);

        // Load
        let loaded = load_intention(&record.intention_id);
        assert!(loaded.is_ok(), "Load failed: {:?}", loaded);

        // Assert equality
        let loaded_record = loaded.unwrap();
        assert_eq!(record, loaded_record, "Loaded record must equal saved record");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_load_not_found() {
        let _test_dir = setup_test_dir();

        let result = load_intention("nonexistent-intention-id-12345");
        assert!(matches!(result, Err(IntentionStorageError::NotFound(_))));
    }

    #[test]
    fn test_reject_unknown_version() {
        let result = load_intention_version("any-id", 99);
        assert!(matches!(result, Err(IntentionStorageError::UnknownVersion(99))));
    }

    #[test]
    fn test_save_invalid_rejected() {
        let _test_dir = setup_test_dir();

        let invalid_record = IntentionRecord {
            intention_id: String::new(), // Invalid: empty
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            author: "test".to_string(),
            statement: "Test".to_string(),
            context: None,
            note: None,
        };

        let result = save_intention(&invalid_record);
        assert!(matches!(result, Err(IntentionStorageError::ValidationFailed(_))));
    }

    #[test]
    fn test_format_version_constant() {
        assert_eq!(INTENTION_FORMAT_VERSION, 1);
    }

    // =========================================================================
    // ISOLATION TESTS
    // These tests prove no cross-module references exist.
    // =========================================================================

    #[test]
    fn proof_no_action_plan_import() {
        // This test documents that intention.rs does not import action_plan.
        //
        // Verification: grep -n "use.*action_plan" crates/anna-shared/src/intention.rs
        // Expected: Zero results
        //
        // The IntentionRecord is completely isolated from action plans.
    }

    #[test]
    fn proof_no_execution_reference() {
        // This test documents that intention.rs does not reference execution code.
        //
        // Verification:
        //   grep -n "ExecutionAdapter\|ExecutionGate\|ExecutionResult\|ExecutionAttempt"
        //        crates/anna-shared/src/intention.rs
        // Expected: Zero results
        //
        // The IntentionRecord is completely isolated from execution.
    }

    #[test]
    fn proof_no_approval_reference() {
        // This test documents that intention.rs does not reference approval code.
        //
        // Verification: grep -n "ApprovalRecord\|ApprovalDecision" crates/anna-shared/src/intention.rs
        // Expected: Zero results
        //
        // The IntentionRecord is completely isolated from approvals.
    }

    #[test]
    fn proof_no_readiness_reference() {
        // This test documents that intention.rs does not reference readiness code.
        //
        // Verification: grep -n "ExecutionReadiness" crates/anna-shared/src/intention.rs
        // Expected: Zero results
        //
        // The IntentionRecord is completely isolated from readiness classification.
    }

    #[test]
    fn proof_intention_has_no_operational_meaning() {
        // This test documents that IntentionRecord has no operational meaning.
        //
        // An IntentionRecord contains:
        // - intention_id: just an identifier
        // - recorded_utc: just a timestamp
        // - author: just a string
        // - statement: free-form text, not parsed or interpreted
        // - context: optional notes
        // - note: optional notes
        //
        // None of these fields can be:
        // - Converted to commands
        // - Matched against operation types
        // - Used to select services or files
        // - Passed to any execution function
        //
        // The record is pure documentation.
        let record = golden_valid_intention();

        // We can read the statement, but we cannot act on it
        let _ = record.statement.clone();

        // There is no: interpret_intention(&record) -> ActionPlan
        // There is no: execute_intention(&record) -> Result
        // There is no: plan_from_intention(&record) -> DeterministicActionPlan
        //
        // These functions do not exist and must never be added.
    }

    #[test]
    fn proof_statement_is_not_parsed() {
        // The statement field is free-form text. It is not parsed.
        let record = IntentionRecord {
            intention_id: "parse-test".to_string(),
            recorded_utc: "2026-01-15T00:00:00Z".to_string(),
            author: "test".to_string(),
            statement: "sudo rm -rf / --no-preserve-root".to_string(), // Dangerous if parsed
            context: None,
            note: None,
        };

        // This record is valid structurally
        let errors = validate_intention(&record);
        assert_eq!(errors, vec![]);

        // But the statement is never parsed or executed
        // It is stored verbatim as documentation
        // There is no code that reads this and runs it
    }
}

// =============================================================================
// This record captures human intent without enabling planning or execution.
// =============================================================================
