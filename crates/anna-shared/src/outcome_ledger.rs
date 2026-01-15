//! Outcome Ledger - Truthful request outcome tracking.
//!
//! Phase 23: Records exactly one outcome per request. No fake stats.
//! Phase 25: Extended with preflight/verification status for actions.
//! Append-only JSONL format at /var/lib/anna/outcomes.jsonl.

use crate::action_plan::{PreflightResult, VerificationStatus};
use crate::intent_class::IntentClass;
use crate::paths::paths;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};

/// Request mode - how the request was processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestMode {
    /// Information/diagnosis only, no system changes
    Dialogue,
    /// System changes via ActionPlan
    Action,
}

/// Request outcome - terminal state of a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Answer delivered and accepted (success)
    Resolved,
    /// Execution attempted but failed (error)
    Failed,
    /// User cancelled or rejected the action
    Cancelled,
    /// Request timed out or TTL expired
    Expired,
}

impl Outcome {
    /// Whether this outcome counts as a success.
    pub fn is_success(&self) -> bool {
        *self == Outcome::Resolved
    }

    /// Whether this outcome counts as a failure.
    pub fn is_failure(&self) -> bool {
        *self == Outcome::Failed
    }
}

/// A single outcome record in the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    /// RFC3339 timestamp in UTC
    pub ts_utc: String,
    /// Unique request identifier
    pub request_id: String,
    /// How the request was processed
    pub mode: RequestMode,
    /// READ_ONLY or MUTATING classification
    pub intent: IntentClassRecord,
    /// Terminal outcome
    pub outcome: Outcome,
    /// Whether request was escalated to senior specialist
    pub escalated: bool,
    /// Total request duration in milliseconds
    pub duration_ms: u64,
    /// Phase 25: Preflight result (only for Action mode)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight: Option<PreflightResult>,
    /// Phase 25: Verification status (only for Action mode)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationStatus>,
    /// Phase 25: Was elevated confirmation required
    #[serde(default)]
    pub elevated_confirmation: bool,
}

/// Serializable intent class (avoids importing full IntentClass in stats)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentClassRecord {
    ReadOnly,
    Mutating,
}

impl From<IntentClass> for IntentClassRecord {
    fn from(intent: IntentClass) -> Self {
        match intent {
            IntentClass::ReadOnly => IntentClassRecord::ReadOnly,
            IntentClass::Mutating => IntentClassRecord::Mutating,
        }
    }
}

impl OutcomeRecord {
    /// Create a new outcome record (for Dialogue mode).
    pub fn new(
        request_id: &str,
        mode: RequestMode,
        intent: IntentClass,
        outcome: Outcome,
        escalated: bool,
        duration_ms: u64,
    ) -> Self {
        Self {
            ts_utc: chrono::Utc::now().to_rfc3339(),
            request_id: request_id.to_string(),
            mode,
            intent: intent.into(),
            outcome,
            escalated,
            duration_ms,
            preflight: None,
            verification: None,
            elevated_confirmation: false,
        }
    }

    /// Phase 25: Create outcome record for Action mode with extended telemetry.
    pub fn new_action(
        request_id: &str,
        intent: IntentClass,
        outcome: Outcome,
        escalated: bool,
        duration_ms: u64,
        preflight: PreflightResult,
        verification: VerificationStatus,
        elevated_confirmation: bool,
    ) -> Self {
        Self {
            ts_utc: chrono::Utc::now().to_rfc3339(),
            request_id: request_id.to_string(),
            mode: RequestMode::Action,
            intent: intent.into(),
            outcome,
            escalated,
            duration_ms,
            preflight: Some(preflight),
            verification: Some(verification),
            elevated_confirmation,
        }
    }
}

/// Append an outcome record to the ledger.
/// This is the ONLY way to record outcomes.
pub fn append_outcome(record: &OutcomeRecord) -> Result<()> {
    let path = paths().outcomes_ledger_file();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    let line = serde_json::to_string(record)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Read all outcome records from the ledger.
pub fn read_all_outcomes() -> Result<Vec<OutcomeRecord>> {
    let path = paths().outcomes_ledger_file();

    if !path.exists() {
        return Ok(vec![]);
    }

    let file = fs::File::open(&path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<OutcomeRecord>(&line) {
            records.push(record);
        }
    }

    Ok(records)
}

/// Aggregated statistics from the outcome ledger.
#[derive(Debug, Clone, Default)]
pub struct OutcomeStats {
    /// Total requests tracked
    pub total: u64,
    /// Read-only requests
    pub read_only: u64,
    /// Mutating requests
    pub mutating: u64,
    /// Resolved (success)
    pub resolved: u64,
    /// Failed
    pub failed: u64,
    /// Cancelled
    pub cancelled: u64,
    /// Expired
    pub expired: u64,
    /// Escalated requests
    pub escalated: u64,
    /// All durations for percentile calculations
    pub durations_ms: Vec<u64>,
    /// Phase 25: Actions with preflight passed
    pub preflight_passed: u64,
    /// Phase 25: Actions with preflight blocked
    pub preflight_blocked: u64,
    /// Phase 25: Actions with preflight unknown
    pub preflight_unknown: u64,
    /// Phase 25: Actions with verification passed
    pub verification_passed: u64,
    /// Phase 25: Actions with verification failed
    pub verification_failed: u64,
    /// Phase 25: Actions with verification unknown
    pub verification_unknown: u64,
    /// Phase 25: Actions with elevated confirmation
    pub elevated_confirmations: u64,
}

impl OutcomeStats {
    /// Compute statistics from ledger records.
    pub fn from_records(records: &[OutcomeRecord]) -> Self {
        use crate::action_plan::{PreflightResult, VerificationStatus};

        let mut stats = Self::default();

        for record in records {
            stats.total += 1;

            match record.intent {
                IntentClassRecord::ReadOnly => stats.read_only += 1,
                IntentClassRecord::Mutating => stats.mutating += 1,
            }

            match record.outcome {
                Outcome::Resolved => stats.resolved += 1,
                Outcome::Failed => stats.failed += 1,
                Outcome::Cancelled => stats.cancelled += 1,
                Outcome::Expired => stats.expired += 1,
            }

            if record.escalated {
                stats.escalated += 1;
            }

            // Phase 25: Track preflight/verification stats for actions
            if let Some(preflight) = record.preflight {
                match preflight {
                    PreflightResult::Passed => stats.preflight_passed += 1,
                    PreflightResult::Blocked => stats.preflight_blocked += 1,
                    PreflightResult::Unknown => stats.preflight_unknown += 1,
                }
            }

            if let Some(verification) = record.verification {
                match verification {
                    VerificationStatus::Passed => stats.verification_passed += 1,
                    VerificationStatus::Failed => stats.verification_failed += 1,
                    VerificationStatus::Unknown => stats.verification_unknown += 1,
                }
            }

            if record.elevated_confirmation {
                stats.elevated_confirmations += 1;
            }

            stats.durations_ms.push(record.duration_ms);
        }

        stats
    }

    /// Load stats directly from ledger file.
    pub fn load() -> Result<Self> {
        let records = read_all_outcomes()?;
        Ok(Self::from_records(&records))
    }

    /// Average duration in milliseconds.
    pub fn avg_duration_ms(&self) -> Option<u64> {
        if self.durations_ms.is_empty() {
            None
        } else {
            let sum: u64 = self.durations_ms.iter().sum();
            Some(sum / self.durations_ms.len() as u64)
        }
    }

    /// Percentile duration (p50, p90, etc.)
    pub fn percentile_duration_ms(&self, percentile: f64) -> Option<u64> {
        if self.durations_ms.is_empty() {
            return None;
        }

        let mut sorted = self.durations_ms.clone();
        sorted.sort_unstable();

        let idx = ((percentile / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    /// Success rate as percentage.
    pub fn success_rate(&self) -> Option<f64> {
        let decisive = self.resolved + self.failed;
        if decisive == 0 {
            None
        } else {
            Some((self.resolved as f64 / decisive as f64) * 100.0)
        }
    }

    /// Escalation rate as percentage.
    pub fn escalation_rate(&self) -> Option<f64> {
        if self.total == 0 {
            None
        } else {
            Some((self.escalated as f64 / self.total as f64) * 100.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_record_serialization() {
        let record = OutcomeRecord::new(
            "test-123",
            RequestMode::Dialogue,
            IntentClass::ReadOnly,
            Outcome::Resolved,
            false,
            150,
        );

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"request_id\":\"test-123\""));
        assert!(json.contains("\"mode\":\"DIALOGUE\""));
        assert!(json.contains("\"intent\":\"READ_ONLY\""));
        assert!(json.contains("\"outcome\":\"resolved\""));
        assert!(json.contains("\"escalated\":false"));
        assert!(json.contains("\"duration_ms\":150"));
    }

    #[test]
    fn test_outcome_stats_calculation() {
        let records = vec![
            OutcomeRecord::new("1", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 100),
            OutcomeRecord::new("2", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 200),
            OutcomeRecord::new("3", RequestMode::Action, IntentClass::Mutating, Outcome::Failed, true, 300),
            OutcomeRecord::new("4", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Cancelled, false, 50),
        ];

        let stats = OutcomeStats::from_records(&records);
        assert_eq!(stats.total, 4);
        assert_eq!(stats.read_only, 3);
        assert_eq!(stats.mutating, 1);
        assert_eq!(stats.resolved, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.cancelled, 1);
        assert_eq!(stats.escalated, 1);
    }

    #[test]
    fn test_percentile_calculation() {
        let records = vec![
            OutcomeRecord::new("1", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 100),
            OutcomeRecord::new("2", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 200),
            OutcomeRecord::new("3", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 300),
            OutcomeRecord::new("4", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 400),
            OutcomeRecord::new("5", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 500),
        ];

        let stats = OutcomeStats::from_records(&records);
        assert_eq!(stats.percentile_duration_ms(50.0), Some(300)); // median
        assert_eq!(stats.avg_duration_ms(), Some(300));
    }

    #[test]
    fn test_success_rate() {
        let records = vec![
            OutcomeRecord::new("1", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 100),
            OutcomeRecord::new("2", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Resolved, false, 100),
            OutcomeRecord::new("3", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Failed, false, 100),
            OutcomeRecord::new("4", RequestMode::Dialogue, IntentClass::ReadOnly, Outcome::Cancelled, false, 100),
        ];

        let stats = OutcomeStats::from_records(&records);
        // Success rate excludes cancelled, only resolved vs failed
        let rate = stats.success_rate().unwrap();
        assert!((rate - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_intent_class_conversion() {
        assert_eq!(IntentClassRecord::from(IntentClass::ReadOnly), IntentClassRecord::ReadOnly);
        assert_eq!(IntentClassRecord::from(IntentClass::Mutating), IntentClassRecord::Mutating);
    }

    #[test]
    fn test_outcome_properties() {
        assert!(Outcome::Resolved.is_success());
        assert!(!Outcome::Resolved.is_failure());
        assert!(Outcome::Failed.is_failure());
        assert!(!Outcome::Failed.is_success());
        assert!(!Outcome::Cancelled.is_success());
        assert!(!Outcome::Cancelled.is_failure());
    }

    #[test]
    fn test_phase25_action_record() {
        use crate::action_plan::{PreflightResult, VerificationStatus};

        let record = OutcomeRecord::new_action(
            "action-123",
            IntentClass::Mutating,
            Outcome::Resolved,
            false,
            500,
            PreflightResult::Passed,
            VerificationStatus::Passed,
            false,
        );

        assert_eq!(record.mode, RequestMode::Action);
        assert_eq!(record.preflight, Some(PreflightResult::Passed));
        assert_eq!(record.verification, Some(VerificationStatus::Passed));
        assert!(!record.elevated_confirmation);

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"preflight\":\"passed\""));
        assert!(json.contains("\"verification\":\"passed\""));
    }

    #[test]
    fn test_phase25_stats_aggregation() {
        use crate::action_plan::{PreflightResult, VerificationStatus};

        let records = vec![
            OutcomeRecord::new_action(
                "1", IntentClass::Mutating, Outcome::Resolved, false, 100,
                PreflightResult::Passed, VerificationStatus::Passed, false,
            ),
            OutcomeRecord::new_action(
                "2", IntentClass::Mutating, Outcome::Failed, false, 200,
                PreflightResult::Passed, VerificationStatus::Failed, false,
            ),
            OutcomeRecord::new_action(
                "3", IntentClass::Mutating, Outcome::Cancelled, false, 50,
                PreflightResult::Unknown, VerificationStatus::Unknown, true,
            ),
        ];

        let stats = OutcomeStats::from_records(&records);
        assert_eq!(stats.preflight_passed, 2);
        assert_eq!(stats.preflight_unknown, 1);
        assert_eq!(stats.verification_passed, 1);
        assert_eq!(stats.verification_failed, 1);
        assert_eq!(stats.verification_unknown, 1);
        assert_eq!(stats.elevated_confirmations, 1);
    }
}
