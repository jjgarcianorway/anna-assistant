//! Evidence Snapshot for Phase 27 Confidence Calculation.
//!
//! Collects evidence metrics used to compute confidence without LLM assertion.

use anna_shared::fingerprint::FactFingerprint;
use anna_shared::intent_class::IntentClass;
use anna_shared::telemetry_consumer::TelemetrySnapshot;

/// Snapshot of evidence collected during query processing.
#[derive(Debug, Clone)]
pub struct EvidenceSnapshot {
    /// Number of probes executed
    pub probes_run: usize,
    /// Evidence completeness (0.0-1.0)
    pub evidence_completeness: f32,
    /// Fingerprint of current system state
    pub fingerprint: Option<FactFingerprint>,
    /// Similarity to best historical match (0.0-1.0)
    pub fingerprint_similarity: f32,
    /// Historical success rate for this query class
    pub historical_success_rate: f32,
}

impl EvidenceSnapshot {
    /// Build evidence snapshot from current iteration state.
    pub fn build(
        probes_run: usize,
        intent: IntentClass,
        probe_outputs: &[&str],
        historical_fingerprints: &[FactFingerprint],
        telemetry: &TelemetrySnapshot,
    ) -> Self {
        // Calculate evidence completeness
        let expected_probes = match intent {
            IntentClass::ReadOnly => 2,
            IntentClass::Mutating => 3,
        };
        let evidence_completeness = (probes_run as f32 / expected_probes as f32).min(1.0);

        // Build fingerprint and find similarity
        let (fingerprint, fingerprint_similarity) = if !probe_outputs.is_empty() {
            let fp = FactFingerprint::from_probe_outputs(probe_outputs);
            let similarity = anna_shared::fingerprint::find_best_match(&fp, historical_fingerprints)
                .map(|(score, _)| score)
                .unwrap_or(0.0);
            (Some(fp), similarity)
        } else {
            (None, 0.0)
        };

        // Historical success rate
        let historical_success_rate = telemetry
            .success_rate()
            .map(|r| r as f32)
            .unwrap_or(0.5); // Unknown prior

        Self {
            probes_run,
            evidence_completeness,
            fingerprint,
            fingerprint_similarity,
            historical_success_rate,
        }
    }

    /// Simplified builder for cold start (no historical data).
    pub fn cold_start(probes_run: usize, intent: IntentClass, probe_outputs: &[&str]) -> Self {
        let expected_probes = match intent {
            IntentClass::ReadOnly => 2,
            IntentClass::Mutating => 3,
        };
        let evidence_completeness = (probes_run as f32 / expected_probes as f32).min(1.0);

        let fingerprint = if !probe_outputs.is_empty() {
            Some(FactFingerprint::from_probe_outputs(probe_outputs))
        } else {
            None
        };

        Self {
            probes_run,
            evidence_completeness,
            fingerprint,
            fingerprint_similarity: 0.0, // No historical data
            historical_success_rate: 0.5, // Unknown prior
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_completeness_readonly() {
        let snapshot = EvidenceSnapshot::cold_start(
            2,
            IntentClass::ReadOnly,
            &["output1", "output2"],
        );
        assert!((snapshot.evidence_completeness - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_evidence_completeness_mutating() {
        let snapshot = EvidenceSnapshot::cold_start(
            2,
            IntentClass::Mutating,
            &["output1", "output2"],
        );
        // 2/3 = 0.67
        assert!((snapshot.evidence_completeness - 0.67).abs() < 0.1);
    }

    #[test]
    fn test_evidence_completeness_capped() {
        let snapshot = EvidenceSnapshot::cold_start(
            5,
            IntentClass::ReadOnly,
            &["o1", "o2", "o3", "o4", "o5"],
        );
        assert!((snapshot.evidence_completeness - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cold_start_defaults() {
        let snapshot = EvidenceSnapshot::cold_start(0, IntentClass::ReadOnly, &[]);
        assert_eq!(snapshot.fingerprint_similarity, 0.0);
        assert_eq!(snapshot.historical_success_rate, 0.5);
        assert!(snapshot.fingerprint.is_none());
    }
}
