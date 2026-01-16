//! Telemetry Consumer - Rolling window aggregation from outcome ledger.
//!
//! Phase 24: Reads outcomes.jsonl and provides aggregates for policy decisions.
//! Phase 26: Added abstained tracking for decision quality metrics.
//! No new metrics - only aggregations derivable from existing ledger fields.

use crate::outcome_ledger::{read_all_outcomes, IntentClassRecord, Outcome, OutcomeRecord};
use chrono::{DateTime, Duration, Utc};

/// Rolling window configuration.
pub const DEFAULT_WINDOW_HOURS: i64 = 24;
pub const MINIMUM_SAMPLE_SIZE: usize = 10;

/// Aggregated telemetry for policy decisions.
#[derive(Debug, Clone, Default)]
pub struct TelemetrySnapshot {
    /// Total outcomes in window
    pub total: usize,
    /// Resolved outcomes
    pub resolved: usize,
    /// Failed outcomes
    pub failed: usize,
    /// Cancelled outcomes
    pub cancelled: usize,
    /// Expired outcomes
    pub expired: usize,
    /// Phase 26: Abstained outcomes (low confidence, no error)
    pub abstained: usize,
    /// Escalated count
    pub escalated: usize,
    /// READ_ONLY outcomes
    pub read_only: usize,
    /// MUTATING outcomes
    pub mutating: usize,
    /// Duration samples for percentile calculation
    pub durations_ms: Vec<u64>,
    /// Whether we have enough data for reliable decisions
    pub sufficient_data: bool,
}

impl TelemetrySnapshot {
    /// Load snapshot from ledger with rolling window.
    pub fn load(window_hours: i64) -> Self {
        let records = read_all_outcomes().unwrap_or_default();
        Self::from_records_windowed(&records, window_hours)
    }

    /// Load with default window.
    pub fn load_default() -> Self {
        Self::load(DEFAULT_WINDOW_HOURS)
    }

    /// Create snapshot from records within time window.
    pub fn from_records_windowed(records: &[OutcomeRecord], window_hours: i64) -> Self {
        let cutoff = Utc::now() - Duration::hours(window_hours);
        let filtered: Vec<_> = records
            .iter()
            .filter(|r| {
                DateTime::parse_from_rfc3339(&r.ts_utc)
                    .map(|dt| dt.with_timezone(&Utc) > cutoff)
                    .unwrap_or(false)
            })
            .collect();

        Self::from_records(&filtered)
    }

    /// Create snapshot from record slice.
    fn from_records(records: &[&OutcomeRecord]) -> Self {
        let mut snapshot = Self::default();

        for record in records {
            snapshot.total += 1;

            match record.outcome {
                Outcome::Resolved => snapshot.resolved += 1,
                Outcome::Failed => snapshot.failed += 1,
                Outcome::Cancelled => snapshot.cancelled += 1,
                Outcome::Expired => snapshot.expired += 1,
                Outcome::Abstained => snapshot.abstained += 1,
            }

            if record.escalated {
                snapshot.escalated += 1;
            }

            match record.intent {
                IntentClassRecord::ReadOnly => snapshot.read_only += 1,
                IntentClassRecord::Mutating => snapshot.mutating += 1,
            }

            snapshot.durations_ms.push(record.duration_ms);
        }

        snapshot.sufficient_data = snapshot.total >= MINIMUM_SAMPLE_SIZE;
        snapshot
    }

    /// Success rate (resolved / (resolved + failed)), None if no decisive outcomes.
    pub fn success_rate(&self) -> Option<f64> {
        let decisive = self.resolved + self.failed;
        if decisive == 0 {
            None
        } else {
            Some(self.resolved as f64 / decisive as f64)
        }
    }

    /// Failure rate (failed / (resolved + failed)), None if no decisive outcomes.
    pub fn failure_rate(&self) -> Option<f64> {
        self.success_rate().map(|sr| 1.0 - sr)
    }

    /// Escalation rate (escalated / total), None if no data.
    pub fn escalation_rate(&self) -> Option<f64> {
        if self.total == 0 {
            None
        } else {
            Some(self.escalated as f64 / self.total as f64)
        }
    }

    /// Cancellation rate (cancelled / mutating), None if no mutating.
    pub fn cancellation_rate(&self) -> Option<f64> {
        if self.mutating == 0 {
            None
        } else {
            Some(self.cancelled as f64 / self.mutating as f64)
        }
    }

    /// Expiration rate (expired / total), None if no data.
    pub fn expiration_rate(&self) -> Option<f64> {
        if self.total == 0 {
            None
        } else {
            Some(self.expired as f64 / self.total as f64)
        }
    }

    /// Phase 26: Abstention rate (abstained / total), None if no data.
    pub fn abstention_rate(&self) -> Option<f64> {
        if self.total == 0 {
            None
        } else {
            Some(self.abstained as f64 / self.total as f64)
        }
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

    /// Percentile duration (e.g., 50.0 for p50, 90.0 for p90).
    pub fn percentile_duration_ms(&self, percentile: f64) -> Option<u64> {
        if self.durations_ms.is_empty() {
            return None;
        }

        let mut sorted = self.durations_ms.clone();
        sorted.sort_unstable();

        let idx = ((percentile / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    /// Mutating success rate (resolved mutating / total mutating decisive).
    pub fn mutating_success_rate(&self, records: &[OutcomeRecord]) -> Option<f64> {
        let cutoff = Utc::now() - Duration::hours(DEFAULT_WINDOW_HOURS);
        let mutating_outcomes: Vec<_> = records
            .iter()
            .filter(|r| {
                r.intent == IntentClassRecord::Mutating
                    && DateTime::parse_from_rfc3339(&r.ts_utc)
                        .map(|dt| dt.with_timezone(&Utc) > cutoff)
                        .unwrap_or(false)
            })
            .collect();

        let resolved = mutating_outcomes
            .iter()
            .filter(|r| r.outcome == Outcome::Resolved)
            .count();
        let failed = mutating_outcomes
            .iter()
            .filter(|r| r.outcome == Outcome::Failed)
            .count();
        let decisive = resolved + failed;

        if decisive == 0 {
            None
        } else {
            Some(resolved as f64 / decisive as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent_class::IntentClass;
    use crate::outcome_ledger::RequestMode;

    fn make_record(outcome: Outcome, intent: IntentClass, escalated: bool, duration_ms: u64) -> OutcomeRecord {
        OutcomeRecord::new(
            &uuid::Uuid::new_v4().to_string(),
            RequestMode::Dialogue,
            intent,
            outcome,
            escalated,
            duration_ms,
        )
    }

    #[test]
    fn test_empty_snapshot() {
        let snapshot = TelemetrySnapshot::from_records(&[]);
        assert_eq!(snapshot.total, 0);
        assert!(!snapshot.sufficient_data);
        assert!(snapshot.success_rate().is_none());
    }

    #[test]
    fn test_success_rate_calculation() {
        let records: Vec<OutcomeRecord> = vec![
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 100),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 100),
            make_record(Outcome::Failed, IntentClass::ReadOnly, false, 100),
        ];
        let refs: Vec<&OutcomeRecord> = records.iter().collect();
        let snapshot = TelemetrySnapshot::from_records(&refs);

        let rate = snapshot.success_rate().unwrap();
        assert!((rate - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_sufficient_data_threshold() {
        let records: Vec<OutcomeRecord> = (0..10)
            .map(|_| make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 100))
            .collect();
        let refs: Vec<&OutcomeRecord> = records.iter().collect();
        let snapshot = TelemetrySnapshot::from_records(&refs);

        assert!(snapshot.sufficient_data);
    }

    #[test]
    fn test_escalation_rate() {
        let records: Vec<OutcomeRecord> = vec![
            make_record(Outcome::Resolved, IntentClass::ReadOnly, true, 100),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 100),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 100),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 100),
        ];
        let refs: Vec<&OutcomeRecord> = records.iter().collect();
        let snapshot = TelemetrySnapshot::from_records(&refs);

        let rate = snapshot.escalation_rate().unwrap();
        assert!((rate - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_percentile_calculation() {
        let records: Vec<OutcomeRecord> = vec![
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 100),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 200),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 300),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 400),
            make_record(Outcome::Resolved, IntentClass::ReadOnly, false, 500),
        ];
        let refs: Vec<&OutcomeRecord> = records.iter().collect();
        let snapshot = TelemetrySnapshot::from_records(&refs);

        assert_eq!(snapshot.percentile_duration_ms(50.0), Some(300));
        assert_eq!(snapshot.avg_duration_ms(), Some(300));
    }
}
