//! Case File Schema v0.0.81
//!
//! Stable schema for case files at /var/lib/anna/cases/<case_id>.json
//!
//! Includes:
//! - All mutation artifacts (backups, rollback info, verification results)
//! - Unified structure for all case types (query, diagnose, mutation)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Case files directory
pub const CASES_DIR: &str = "/var/lib/anna/cases";

/// Current schema version
pub const CASE_SCHEMA_VERSION: u32 = 81;

/// Case file v0.0.81
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFileV081 {
    /// Schema version (81 for v0.0.81)
    pub schema_version: u32,

    // Identity
    /// Unique case ID (e.g., "case-1733324800-a1b2c3")
    pub case_id: String,
    /// When case started
    pub created_at: DateTime<Utc>,
    /// When case ended
    pub ended_at: Option<DateTime<Utc>>,

    // Classification
    /// Original user request
    pub request: String,
    /// Intent type (SYSTEM_QUERY, DIAGNOSE, ACTION_REQUEST, etc.)
    pub intent: String,
    /// Intent classification confidence (0-100)
    pub intent_confidence: u8,
    /// Query targets (e.g., ["cpu", "memory"])
    pub targets: Vec<String>,

    // Outcome
    /// Final status
    pub status: CaseStatus,
    /// Reliability score (0-100)
    pub reliability_score: u8,
    /// Final answer or summary
    pub final_answer: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,

    // Evidence
    /// Evidence gathered
    pub evidence: Vec<CaseEvidence>,

    // Timing
    pub timing: CaseTiming,

    // Mutation data (only for ACTION_REQUEST)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation: Option<CaseMutationData>,

    // Doctor data (only for DIAGNOSE)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor: Option<CaseDoctorData>,

    // Junior verification
    pub verification: CaseVerification,
}

/// Case status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    Success,
    Partial,
    Failed,
    Blocked,
    Cancelled,
}

impl Default for CaseStatus {
    fn default() -> Self {
        CaseStatus::Success
    }
}

/// Evidence entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseEvidence {
    /// Evidence ID (e.g., "E1")
    pub evidence_id: String,
    /// Tool that gathered this evidence
    pub tool_name: String,
    /// Human-readable source description
    pub source_description: String,
    /// Key finding from this evidence
    pub key_finding: Option<String>,
    /// When gathered
    pub gathered_at: DateTime<Utc>,
}

/// Timing breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaseTiming {
    /// Total duration in milliseconds
    pub total_ms: u64,
    /// Translator phase
    pub translator_ms: u64,
    /// Evidence gathering phase
    pub evidence_ms: u64,
    /// Junior verification phase
    pub junior_ms: u64,
    /// Mutation execution phase (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutation_ms: Option<u64>,
    /// Doctor flow phase (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor_ms: Option<u64>,
}

/// Mutation data for ACTION_REQUEST cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseMutationData {
    /// Mutation plan ID
    pub plan_id: String,
    /// Risk level
    pub risk: String,
    /// Confirmation phrase used
    pub confirmation_phrase: String,
    /// Whether mutation was executed
    pub executed: bool,
    /// Execution result
    pub execution_result: Option<MutationExecutionRecord>,
    /// Backup locations
    pub backups: Vec<BackupRecord>,
    /// Rollback info
    pub rollback: Option<RollbackRecord>,
    /// Verification results
    pub verifications: Vec<VerificationRecord>,
}

/// Record of mutation execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationExecutionRecord {
    /// Step results
    pub steps: Vec<StepRecord>,
    /// Overall success
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Record of a single mutation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRecord {
    /// Step description
    pub description: String,
    /// Success
    pub success: bool,
    /// Output message
    pub output: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Backup record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    /// Original file path
    pub original_path: String,
    /// Backup file path
    pub backup_path: String,
    /// Backup timestamp
    pub created_at: DateTime<Utc>,
    /// File size in bytes
    pub size_bytes: u64,
}

/// Rollback record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRecord {
    /// Whether rollback was performed
    pub performed: bool,
    /// Rollback result
    pub success: bool,
    /// Steps that were rolled back
    pub steps_rolled_back: Vec<String>,
    /// Error message if rollback failed
    pub error: Option<String>,
}

/// Verification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRecord {
    /// Check description
    pub description: String,
    /// Whether passed
    pub passed: bool,
    /// Expected value
    pub expected: String,
    /// Actual value found
    pub actual: String,
    /// Diagnostic info
    pub diagnostic: Option<String>,
}

/// Doctor data for DIAGNOSE cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseDoctorData {
    /// Doctor ID
    pub doctor_id: String,
    /// Doctor name
    pub doctor_name: String,
    /// Problem domain
    pub domain: String,
    /// Findings
    pub findings: Vec<DoctorFinding>,
    /// Most likely cause
    pub most_likely_cause: Option<String>,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Doctor finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorFinding {
    /// Check that produced this finding
    pub check_id: String,
    /// Severity (info, warning, error)
    pub severity: String,
    /// Finding message
    pub message: String,
}

/// Junior verification data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaseVerification {
    /// Reliability score (0-100)
    pub score: u8,
    /// Critique from Junior
    pub critique: String,
    /// Uncited claims found
    pub uncited_claims: Vec<String>,
    /// Suggestions for improvement
    pub suggestions: String,
}

impl CaseFileV081 {
    /// Create a new case file
    pub fn new(case_id: &str, request: &str) -> Self {
        Self {
            schema_version: CASE_SCHEMA_VERSION,
            case_id: case_id.to_string(),
            created_at: Utc::now(),
            ended_at: None,
            request: request.to_string(),
            intent: "unknown".to_string(),
            intent_confidence: 0,
            targets: Vec::new(),
            status: CaseStatus::default(),
            reliability_score: 0,
            final_answer: None,
            error_message: None,
            evidence: Vec::new(),
            timing: CaseTiming::default(),
            mutation: None,
            doctor: None,
            verification: CaseVerification::default(),
        }
    }

    /// Set intent classification
    pub fn with_intent(mut self, intent: &str, confidence: u8) -> Self {
        self.intent = intent.to_string();
        self.intent_confidence = confidence;
        self
    }

    /// Set targets
    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.targets = targets;
        self
    }

    /// Mark as mutation case
    pub fn with_mutation(mut self, mutation: CaseMutationData) -> Self {
        self.mutation = Some(mutation);
        self
    }

    /// Mark as doctor case
    pub fn with_doctor(mut self, doctor: CaseDoctorData) -> Self {
        self.doctor = Some(doctor);
        self
    }

    /// Add evidence
    pub fn add_evidence(&mut self, evidence: CaseEvidence) {
        self.evidence.push(evidence);
    }

    /// Set verification
    pub fn set_verification(&mut self, verification: CaseVerification) {
        self.reliability_score = verification.score;
        self.verification = verification;
    }

    /// Complete the case
    pub fn complete(&mut self, status: CaseStatus, answer: Option<String>) {
        self.ended_at = Some(Utc::now());
        self.status = status;
        self.final_answer = answer;

        if let (Some(created), Some(ended)) = (Some(self.created_at), self.ended_at) {
            self.timing.total_ms = (ended - created).num_milliseconds() as u64;
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        // Ensure directory exists
        fs::create_dir_all(CASES_DIR).map_err(|e| format!("Cannot create cases dir: {}", e))?;

        let path = format!("{}/{}.json", CASES_DIR, self.case_id);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Cannot serialize case: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Cannot write case file: {}", e))?;

        Ok(())
    }

    /// Load from disk
    pub fn load(case_id: &str) -> Result<Self, String> {
        let path = format!("{}/{}.json", CASES_DIR, case_id);

        let json =
            fs::read_to_string(&path).map_err(|e| format!("Cannot read case file: {}", e))?;

        serde_json::from_str(&json).map_err(|e| format!("Cannot parse case file: {}", e))
    }

    /// List recent cases
    pub fn list_recent(limit: usize) -> Vec<CaseSummaryV081> {
        let mut cases = Vec::new();

        if let Ok(entries) = fs::read_dir(CASES_DIR) {
            for entry in entries.flatten() {
                if let Ok(path) = entry.path().canonicalize() {
                    if path.extension().map(|e| e == "json").unwrap_or(false) {
                        if let Ok(case) = Self::load(
                            path.file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or_default(),
                        ) {
                            cases.push(CaseSummaryV081::from(&case));
                        }
                    }
                }
            }
        }

        // Sort by created_at descending
        cases.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        cases.truncate(limit);
        cases
    }
}

/// Summary for listing cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummaryV081 {
    pub case_id: String,
    pub created_at: DateTime<Utc>,
    pub request: String,
    pub intent: String,
    pub status: CaseStatus,
    pub reliability_score: u8,
    pub is_mutation: bool,
}

impl From<&CaseFileV081> for CaseSummaryV081 {
    fn from(case: &CaseFileV081) -> Self {
        Self {
            case_id: case.case_id.clone(),
            created_at: case.created_at,
            request: if case.request.len() > 50 {
                format!("{}...", &case.request[..47])
            } else {
                case.request.clone()
            },
            intent: case.intent.clone(),
            status: case.status,
            reliability_score: case.reliability_score,
            is_mutation: case.mutation.is_some(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_case_file() {
        let case = CaseFileV081::new("test-case-123", "what cpu do I have");
        assert_eq!(case.schema_version, CASE_SCHEMA_VERSION);
        assert_eq!(case.case_id, "test-case-123");
        assert_eq!(case.request, "what cpu do I have");
        assert_eq!(case.intent, "unknown");
    }

    #[test]
    fn test_with_intent() {
        let case = CaseFileV081::new("test", "test")
            .with_intent("SYSTEM_QUERY", 95)
            .with_targets(vec!["cpu".to_string()]);

        assert_eq!(case.intent, "SYSTEM_QUERY");
        assert_eq!(case.intent_confidence, 95);
        assert_eq!(case.targets, vec!["cpu"]);
    }

    #[test]
    fn test_case_summary() {
        let case = CaseFileV081::new("case-123", "this is a very long request that should be truncated");
        let summary = CaseSummaryV081::from(&case);

        assert_eq!(summary.case_id, "case-123");
        assert!(summary.request.ends_with("..."));
    }

    #[test]
    fn test_serialization() {
        let case = CaseFileV081::new("test", "test query")
            .with_intent("SYSTEM_QUERY", 90);

        let json = serde_json::to_string(&case).unwrap();
        let parsed: CaseFileV081 = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.case_id, case.case_id);
        assert_eq!(parsed.intent, case.intent);
    }
}
