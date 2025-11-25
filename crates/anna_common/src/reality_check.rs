//! Reality Check Engine - Multi-signal truth verification
//!
//! v6.48.0: Validates LLM outputs against system reality through 4-step verification

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reality check result after verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RealityCheckResult {
    /// Overall verification status
    pub status: VerificationStatus,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Individual signal results
    pub signals: Vec<SignalResult>,

    /// Discrepancies found (if any)
    pub discrepancies: Vec<Discrepancy>,

    /// Timestamp of check
    pub checked_at: DateTime<Utc>,
}

/// Overall verification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    /// All signals agree with LLM output
    Verified,

    /// Some signals contradict LLM output
    Contradicted { severity: ContradictionSeverity },

    /// Insufficient signals to verify
    Inconclusive { reason: String },

    /// Unable to perform check
    Failed { error: String },
}

/// Severity of contradiction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContradictionSeverity {
    Minor,   // Single signal disagrees
    Major,   // Multiple signals disagree
    Critical, // All signals disagree or safety concern
}

/// Result from a single verification signal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalResult {
    /// Signal type
    pub signal_type: SignalType,

    /// Agreement with LLM output
    pub agreement: Agreement,

    /// Confidence of this signal (0.0 - 1.0)
    pub confidence: f64,

    /// Optional explanation
    pub explanation: Option<String>,
}

/// Types of verification signals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    /// Direct system telemetry check
    Telemetry,

    /// File system verification
    FileSystem,

    /// Process/service status
    ProcessStatus,

    /// Historical pattern comparison
    HistoricalPattern,

    /// Logical consistency check
    LogicalConsistency,

    /// Safety rails validation
    SafetyValidation,
}

/// Agreement level with LLM output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Agreement {
    Agrees,
    Disagrees { reason: String },
    Uncertain,
}

/// A discrepancy between LLM output and reality
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Discrepancy {
    /// What the LLM claimed
    pub llm_claim: String,

    /// What reality shows
    pub reality: String,

    /// Which signal detected this
    pub detected_by: SignalType,

    /// Severity of discrepancy
    pub severity: DiscrepancySeverity,
}

/// Severity of a discrepancy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscrepancySeverity {
    Harmless, // Minor inaccuracy, no impact
    Misleading, // Could lead to wrong decisions
    Dangerous, // Could cause system harm
}

/// Output shape specification for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputShape {
    /// Expected fields and their types
    pub expected_fields: HashMap<String, FieldType>,

    /// Required verifications
    pub verifications: Vec<VerificationRequirement>,

    /// Maximum acceptable discrepancy level
    pub max_discrepancy: DiscrepancySeverity,
}

/// Expected field type in output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    ServiceStatus { service_name: String },
    PackageInfo { package_name: String },
    ResourceMetric { metric_name: String },
    FileExists { path: String },
    Command { expected_output: String },
}

/// A verification requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequirement {
    /// Field to verify
    pub field_name: String,

    /// Signal type to use
    pub signal_type: SignalType,

    /// Whether this is required (vs optional)
    pub required: bool,
}

/// Reality check engine
#[derive(Debug, Clone)]
pub struct RealityCheckEngine {
    /// Minimum confidence threshold
    confidence_threshold: f64,

    /// Maximum signals to wait for
    max_signals: usize,
}

impl RealityCheckEngine {
    /// Create new reality check engine
    pub fn new(confidence_threshold: f64, max_signals: usize) -> Self {
        Self {
            confidence_threshold,
            max_signals,
        }
    }

    /// Create with default settings
    pub fn default() -> Self {
        Self::new(0.7, 6)
    }

    /// Perform reality check on LLM output
    pub fn check(
        &self,
        output_shape: &OutputShape,
        signal_results: Vec<SignalResult>,
        now: DateTime<Utc>,
    ) -> RealityCheckResult {
        if signal_results.is_empty() {
            return RealityCheckResult {
                status: VerificationStatus::Inconclusive {
                    reason: "No verification signals available".to_string(),
                },
                confidence: 0.0,
                signals: vec![],
                discrepancies: vec![],
                checked_at: now,
            };
        }

        // Analyze signals
        let agrees_count = signal_results.iter().filter(|s| matches!(s.agreement, Agreement::Agrees)).count();
        let disagrees_count = signal_results.iter().filter(|s| matches!(s.agreement, Agreement::Disagrees { .. })).count();
        let total_count = signal_results.len();

        // Calculate overall confidence
        let confidence = if total_count > 0 {
            let weighted_sum: f64 = signal_results.iter()
                .map(|s| match &s.agreement {
                    Agreement::Agrees => s.confidence,
                    Agreement::Disagrees { .. } => -s.confidence,
                    Agreement::Uncertain => 0.0,
                })
                .sum();
            ((weighted_sum / total_count as f64) + 1.0) / 2.0 // Normalize to 0-1
        } else {
            0.0
        };

        // Extract discrepancies
        let discrepancies = signal_results.iter()
            .filter_map(|s| {
                if let Agreement::Disagrees { reason } = &s.agreement {
                    Some(Discrepancy {
                        llm_claim: "LLM output".to_string(),
                        reality: reason.clone(),
                        detected_by: s.signal_type.clone(),
                        severity: DiscrepancySeverity::Misleading,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Determine status
        let status = if disagrees_count == 0 && agrees_count > 0 {
            VerificationStatus::Verified
        } else if disagrees_count == total_count {
            VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Critical,
            }
        } else if disagrees_count > agrees_count {
            VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Major,
            }
        } else if disagrees_count > 0 {
            VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Minor,
            }
        } else {
            VerificationStatus::Inconclusive {
                reason: "All signals uncertain".to_string(),
            }
        };

        RealityCheckResult {
            status,
            confidence,
            signals: signal_results,
            discrepancies,
            checked_at: now,
        }
    }

    /// Check if result meets confidence threshold
    pub fn meets_threshold(&self, result: &RealityCheckResult) -> bool {
        result.confidence >= self.confidence_threshold
    }

    /// Get recommended action based on result
    pub fn recommend_action(&self, result: &RealityCheckResult) -> RecommendedAction {
        match &result.status {
            VerificationStatus::Verified => {
                if result.confidence >= self.confidence_threshold {
                    RecommendedAction::Proceed
                } else {
                    RecommendedAction::ProceedWithCaution {
                        reason: "Low confidence despite verification".to_string(),
                    }
                }
            }
            VerificationStatus::Contradicted { severity } => match severity {
                ContradictionSeverity::Critical => RecommendedAction::Abort {
                    reason: "Critical contradiction detected".to_string(),
                },
                ContradictionSeverity::Major => RecommendedAction::RequestClarification {
                    discrepancies: result.discrepancies.clone(),
                },
                ContradictionSeverity::Minor => RecommendedAction::ProceedWithCaution {
                    reason: "Minor contradiction detected".to_string(),
                },
            },
            VerificationStatus::Inconclusive { reason } => {
                RecommendedAction::RequestMoreSignals {
                    reason: reason.clone(),
                }
            }
            VerificationStatus::Failed { error } => RecommendedAction::Abort {
                reason: format!("Verification failed: {}", error),
            },
        }
    }
}

/// Recommended action based on reality check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendedAction {
    /// Safe to proceed with LLM output
    Proceed,

    /// Proceed but with caution
    ProceedWithCaution { reason: String },

    /// Request clarification from LLM
    RequestClarification { discrepancies: Vec<Discrepancy> },

    /// Request more verification signals
    RequestMoreSignals { reason: String },

    /// Do not proceed
    Abort { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_output_shape() -> OutputShape {
        let mut expected_fields = HashMap::new();
        expected_fields.insert(
            "nginx_status".to_string(),
            FieldType::ServiceStatus {
                service_name: "nginx".to_string(),
            },
        );

        OutputShape {
            expected_fields,
            verifications: vec![VerificationRequirement {
                field_name: "nginx_status".to_string(),
                signal_type: SignalType::ProcessStatus,
                required: true,
            }],
            max_discrepancy: DiscrepancySeverity::Misleading,
        }
    }

    #[test]
    fn test_all_signals_agree() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let signals = vec![
            SignalResult {
                signal_type: SignalType::Telemetry,
                agreement: Agreement::Agrees,
                confidence: 0.9,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::ProcessStatus,
                agreement: Agreement::Agrees,
                confidence: 0.85,
                explanation: None,
            },
        ];

        let result = engine.check(&test_output_shape(), signals, now);
        assert_eq!(result.status, VerificationStatus::Verified);
        assert!(result.confidence > 0.7);
        assert!(result.discrepancies.is_empty());
    }

    #[test]
    fn test_all_signals_disagree() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let signals = vec![
            SignalResult {
                signal_type: SignalType::Telemetry,
                agreement: Agreement::Disagrees {
                    reason: "Service is down".to_string(),
                },
                confidence: 0.9,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::ProcessStatus,
                agreement: Agreement::Disagrees {
                    reason: "Process not found".to_string(),
                },
                confidence: 0.85,
                explanation: None,
            },
        ];

        let result = engine.check(&test_output_shape(), signals, now);
        assert!(matches!(
            result.status,
            VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Critical
            }
        ));
        assert_eq!(result.discrepancies.len(), 2);
    }

    #[test]
    fn test_mixed_signals_major() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let signals = vec![
            SignalResult {
                signal_type: SignalType::Telemetry,
                agreement: Agreement::Agrees,
                confidence: 0.8,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::ProcessStatus,
                agreement: Agreement::Disagrees {
                    reason: "Process crashed".to_string(),
                },
                confidence: 0.9,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::FileSystem,
                agreement: Agreement::Disagrees {
                    reason: "Config file missing".to_string(),
                },
                confidence: 0.85,
                explanation: None,
            },
        ];

        let result = engine.check(&test_output_shape(), signals, now);
        assert!(matches!(
            result.status,
            VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Major
            }
        ));
    }

    #[test]
    fn test_mixed_signals_minor() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let signals = vec![
            SignalResult {
                signal_type: SignalType::Telemetry,
                agreement: Agreement::Agrees,
                confidence: 0.9,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::ProcessStatus,
                agreement: Agreement::Agrees,
                confidence: 0.85,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::FileSystem,
                agreement: Agreement::Disagrees {
                    reason: "Timestamp mismatch".to_string(),
                },
                confidence: 0.7,
                explanation: None,
            },
        ];

        let result = engine.check(&test_output_shape(), signals, now);
        assert!(matches!(
            result.status,
            VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Minor
            }
        ));
    }

    #[test]
    fn test_no_signals() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let result = engine.check(&test_output_shape(), vec![], now);
        assert!(matches!(
            result.status,
            VerificationStatus::Inconclusive { .. }
        ));
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_meets_threshold() {
        let engine = RealityCheckEngine::new(0.8, 6);
        let now = Utc::now();

        let result_high = RealityCheckResult {
            status: VerificationStatus::Verified,
            confidence: 0.9,
            signals: vec![],
            discrepancies: vec![],
            checked_at: now,
        };

        let result_low = RealityCheckResult {
            status: VerificationStatus::Verified,
            confidence: 0.5,
            signals: vec![],
            discrepancies: vec![],
            checked_at: now,
        };

        assert!(engine.meets_threshold(&result_high));
        assert!(!engine.meets_threshold(&result_low));
    }

    #[test]
    fn test_recommend_proceed() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let result = RealityCheckResult {
            status: VerificationStatus::Verified,
            confidence: 0.9,
            signals: vec![],
            discrepancies: vec![],
            checked_at: now,
        };

        let action = engine.recommend_action(&result);
        assert_eq!(action, RecommendedAction::Proceed);
    }

    #[test]
    fn test_recommend_abort_critical() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let result = RealityCheckResult {
            status: VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Critical,
            },
            confidence: 0.2,
            signals: vec![],
            discrepancies: vec![],
            checked_at: now,
        };

        let action = engine.recommend_action(&result);
        assert!(matches!(action, RecommendedAction::Abort { .. }));
    }

    #[test]
    fn test_recommend_clarification_major() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let result = RealityCheckResult {
            status: VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Major,
            },
            confidence: 0.4,
            signals: vec![],
            discrepancies: vec![Discrepancy {
                llm_claim: "Service running".to_string(),
                reality: "Service stopped".to_string(),
                detected_by: SignalType::ProcessStatus,
                severity: DiscrepancySeverity::Misleading,
            }],
            checked_at: now,
        };

        let action = engine.recommend_action(&result);
        assert!(matches!(action, RecommendedAction::RequestClarification { .. }));
    }

    #[test]
    fn test_recommend_caution_minor() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        let result = RealityCheckResult {
            status: VerificationStatus::Contradicted {
                severity: ContradictionSeverity::Minor,
            },
            confidence: 0.75,
            signals: vec![],
            discrepancies: vec![],
            checked_at: now,
        };

        let action = engine.recommend_action(&result);
        assert!(matches!(action, RecommendedAction::ProceedWithCaution { .. }));
    }

    #[test]
    fn test_confidence_calculation() {
        let engine = RealityCheckEngine::default();
        let now = Utc::now();

        // High agreement should give high confidence
        let signals_high = vec![
            SignalResult {
                signal_type: SignalType::Telemetry,
                agreement: Agreement::Agrees,
                confidence: 1.0,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::ProcessStatus,
                agreement: Agreement::Agrees,
                confidence: 1.0,
                explanation: None,
            },
        ];

        let result_high = engine.check(&test_output_shape(), signals_high, now);
        assert!(result_high.confidence > 0.9);

        // High disagreement should give low confidence
        let signals_low = vec![
            SignalResult {
                signal_type: SignalType::Telemetry,
                agreement: Agreement::Disagrees {
                    reason: "Test".to_string(),
                },
                confidence: 1.0,
                explanation: None,
            },
            SignalResult {
                signal_type: SignalType::ProcessStatus,
                agreement: Agreement::Disagrees {
                    reason: "Test".to_string(),
                },
                confidence: 1.0,
                explanation: None,
            },
        ];

        let result_low = engine.check(&test_output_shape(), signals_low, now);
        assert!(result_low.confidence < 0.1);
    }
}
