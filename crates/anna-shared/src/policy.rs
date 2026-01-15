//! Policy - Pure functions for behavior modulation.
//!
//! Phase 24: Takes telemetry aggregates, outputs dial settings.
//! No new capabilities - only adjusts existing knobs based on track record.
//!
//! All decisions are deterministic and can be traced in Debug mode.

use crate::telemetry_consumer::TelemetrySnapshot;

/// Policy dial settings computed from telemetry.
#[derive(Debug, Clone)]
pub struct PolicyDials {
    /// Max iterations for READ_ONLY (default: 3)
    pub readonly_max_iterations: u32,
    /// Max iterations for MUTATING (default: 5)
    pub mutating_max_iterations: u32,
    /// Whether to require explicit confirmation for actions (default: true)
    pub require_action_confirmation: bool,
    /// Confidence level for phrasing (affects language choice)
    pub confidence_level: ConfidenceLevel,
    /// When to escalate to senior (threshold as failure rate)
    pub escalation_threshold: f64,
    /// Basis for each decision (for Debug mode tracing)
    pub decision_basis: DecisionBasis,
}

/// Confidence level affects phrasing in answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    /// Success rate >= 90%: "I'm confident..."
    High,
    /// Success rate 70-90%: neutral phrasing
    Medium,
    /// Success rate < 70% or insufficient data: hedged language
    Low,
    /// No data: maximum hedging
    Unknown,
}

impl ConfidenceLevel {
    /// Get appropriate phrasing prefix.
    pub fn phrasing_prefix(&self) -> &'static str {
        match self {
            Self::High => "",  // No prefix needed, confident
            Self::Medium => "",  // Neutral
            Self::Low => "Based on available information, ",
            Self::Unknown => "Without sufficient history, ",
        }
    }

    /// Whether to use confident language.
    pub fn allows_confident_language(&self) -> bool {
        *self == Self::High
    }
}

/// Basis for policy decisions (for Debug mode).
#[derive(Debug, Clone, Default)]
pub struct DecisionBasis {
    pub iterations_basis: String,
    pub confirmation_basis: String,
    pub confidence_basis: String,
    pub escalation_basis: String,
}

impl Default for PolicyDials {
    fn default() -> Self {
        Self::cold_start()
    }
}

impl PolicyDials {
    /// Cold start defaults - conservative until we have data.
    pub fn cold_start() -> Self {
        Self {
            readonly_max_iterations: 3,
            mutating_max_iterations: 5,
            require_action_confirmation: true,
            confidence_level: ConfidenceLevel::Unknown,
            escalation_threshold: 0.3, // Escalate if 30% failure rate
            decision_basis: DecisionBasis {
                iterations_basis: "cold start: default limits".to_string(),
                confirmation_basis: "cold start: always confirm".to_string(),
                confidence_basis: "cold start: unknown confidence".to_string(),
                escalation_basis: "cold start: default threshold".to_string(),
            },
        }
    }

    /// Compute dials from telemetry snapshot.
    pub fn from_telemetry(snapshot: &TelemetrySnapshot) -> Self {
        if !snapshot.sufficient_data {
            return Self::cold_start();
        }

        let mut dials = Self::cold_start();
        let mut basis = DecisionBasis::default();

        // 1. Iteration limits based on success rate
        if let Some(success_rate) = snapshot.success_rate() {
            if success_rate >= 0.95 {
                // Excellent track record: allow more iterations
                dials.readonly_max_iterations = 4;
                dials.mutating_max_iterations = 6;
                basis.iterations_basis = format!(
                    "success rate {:.0}% >= 95%: increased limits (RO:4, M:6)",
                    success_rate * 100.0
                );
            } else if success_rate >= 0.80 {
                // Good track record: standard limits
                dials.readonly_max_iterations = 3;
                dials.mutating_max_iterations = 5;
                basis.iterations_basis = format!(
                    "success rate {:.0}% >= 80%: standard limits (RO:3, M:5)",
                    success_rate * 100.0
                );
            } else {
                // Poor track record: reduce iterations to fail faster
                dials.readonly_max_iterations = 2;
                dials.mutating_max_iterations = 3;
                basis.iterations_basis = format!(
                    "success rate {:.0}% < 80%: reduced limits (RO:2, M:3)",
                    success_rate * 100.0
                );
            }
        } else {
            basis.iterations_basis = "no decisive outcomes: default limits".to_string();
        }

        // 2. Confidence level for phrasing
        dials.confidence_level = match snapshot.success_rate() {
            Some(rate) if rate >= 0.90 => {
                basis.confidence_basis = format!(
                    "success rate {:.0}% >= 90%: high confidence",
                    rate * 100.0
                );
                ConfidenceLevel::High
            }
            Some(rate) if rate >= 0.70 => {
                basis.confidence_basis = format!(
                    "success rate {:.0}% >= 70%: medium confidence",
                    rate * 100.0
                );
                ConfidenceLevel::Medium
            }
            Some(rate) => {
                basis.confidence_basis = format!(
                    "success rate {:.0}% < 70%: low confidence",
                    rate * 100.0
                );
                ConfidenceLevel::Low
            }
            None => {
                basis.confidence_basis = "no decisive outcomes: unknown confidence".to_string();
                ConfidenceLevel::Unknown
            }
        };

        // 3. Escalation threshold based on current failure patterns
        if let Some(failure_rate) = snapshot.failure_rate() {
            if failure_rate >= 0.3 {
                // High failure rate: escalate earlier
                dials.escalation_threshold = 0.2;
                basis.escalation_basis = format!(
                    "failure rate {:.0}% >= 30%: lower threshold (20%)",
                    failure_rate * 100.0
                );
            } else if failure_rate >= 0.1 {
                // Normal failure rate
                dials.escalation_threshold = 0.3;
                basis.escalation_basis = format!(
                    "failure rate {:.0}% >= 10%: standard threshold (30%)",
                    failure_rate * 100.0
                );
            } else {
                // Low failure rate: can be more autonomous
                dials.escalation_threshold = 0.4;
                basis.escalation_basis = format!(
                    "failure rate {:.0}% < 10%: relaxed threshold (40%)",
                    failure_rate * 100.0
                );
            }
        } else {
            basis.escalation_basis = "no decisive outcomes: default threshold".to_string();
        }

        // 4. Action confirmation based on cancellation rate
        if let Some(cancel_rate) = snapshot.cancellation_rate() {
            if cancel_rate >= 0.5 {
                // Users cancel a lot: always confirm
                dials.require_action_confirmation = true;
                basis.confirmation_basis = format!(
                    "cancellation rate {:.0}% >= 50%: always confirm",
                    cancel_rate * 100.0
                );
            } else {
                dials.require_action_confirmation = true; // Still confirm for safety
                basis.confirmation_basis = format!(
                    "cancellation rate {:.0}% < 50%: confirm (safety default)",
                    cancel_rate * 100.0
                );
            }
        } else {
            basis.confirmation_basis = "no mutating outcomes: always confirm".to_string();
        }

        dials.decision_basis = basis;
        dials
    }

    /// Format decision basis for Debug mode output.
    pub fn format_debug_basis(&self) -> String {
        format!(
            "Policy basis:\n  iterations: {}\n  confidence: {}\n  escalation: {}\n  confirmation: {}",
            self.decision_basis.iterations_basis,
            self.decision_basis.confidence_basis,
            self.decision_basis.escalation_basis,
            self.decision_basis.confirmation_basis,
        )
    }
}

/// Global policy accessor - loads from telemetry and caches briefly.
use std::sync::Mutex;
use std::time::{Duration, Instant};

static POLICY_CACHE: Mutex<Option<(PolicyDials, Instant)>> = Mutex::new(None);
const POLICY_CACHE_TTL: Duration = Duration::from_secs(60);

/// Get current policy dials (cached for 60s).
pub fn get_policy() -> PolicyDials {
    let mut cache = POLICY_CACHE.lock().unwrap();

    if let Some((ref dials, ref timestamp)) = *cache {
        if timestamp.elapsed() < POLICY_CACHE_TTL {
            return dials.clone();
        }
    }

    let snapshot = TelemetrySnapshot::load_default();
    let dials = PolicyDials::from_telemetry(&snapshot);

    *cache = Some((dials.clone(), Instant::now()));
    dials
}

/// Force policy refresh (for testing or after significant events).
pub fn refresh_policy() -> PolicyDials {
    let snapshot = TelemetrySnapshot::load_default();
    let dials = PolicyDials::from_telemetry(&snapshot);

    let mut cache = POLICY_CACHE.lock().unwrap();
    *cache = Some((dials.clone(), Instant::now()));
    dials
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cold_start_conservative() {
        let dials = PolicyDials::cold_start();
        assert_eq!(dials.readonly_max_iterations, 3);
        assert_eq!(dials.mutating_max_iterations, 5);
        assert!(dials.require_action_confirmation);
        assert_eq!(dials.confidence_level, ConfidenceLevel::Unknown);
    }

    #[test]
    fn test_insufficient_data_returns_cold_start() {
        let snapshot = TelemetrySnapshot {
            total: 5, // Below MINIMUM_SAMPLE_SIZE
            sufficient_data: false,
            ..Default::default()
        };
        let dials = PolicyDials::from_telemetry(&snapshot);
        assert_eq!(dials.confidence_level, ConfidenceLevel::Unknown);
    }

    #[test]
    fn test_high_success_rate_increases_confidence() {
        let snapshot = TelemetrySnapshot {
            total: 100,
            resolved: 95,
            failed: 5,
            sufficient_data: true,
            ..Default::default()
        };
        let dials = PolicyDials::from_telemetry(&snapshot);
        assert_eq!(dials.confidence_level, ConfidenceLevel::High);
        assert_eq!(dials.readonly_max_iterations, 4);
    }

    #[test]
    fn test_low_success_rate_reduces_iterations() {
        let snapshot = TelemetrySnapshot {
            total: 100,
            resolved: 60,
            failed: 40,
            sufficient_data: true,
            ..Default::default()
        };
        let dials = PolicyDials::from_telemetry(&snapshot);
        assert_eq!(dials.confidence_level, ConfidenceLevel::Low);
        assert_eq!(dials.readonly_max_iterations, 2);
    }

    #[test]
    fn test_high_failure_rate_lowers_escalation_threshold() {
        let snapshot = TelemetrySnapshot {
            total: 100,
            resolved: 60,
            failed: 40,
            sufficient_data: true,
            ..Default::default()
        };
        let dials = PolicyDials::from_telemetry(&snapshot);
        assert_eq!(dials.escalation_threshold, 0.2);
    }

    #[test]
    fn test_confidence_phrasing() {
        assert!(!ConfidenceLevel::Low.allows_confident_language());
        assert!(!ConfidenceLevel::Medium.allows_confident_language());
        assert!(ConfidenceLevel::High.allows_confident_language());
        assert!(!ConfidenceLevel::Unknown.allows_confident_language());
    }
}
