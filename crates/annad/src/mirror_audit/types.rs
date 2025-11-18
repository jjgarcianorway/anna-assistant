//! Core types for Mirror Audit and Temporal Self-Reflection
//!
//! Phase 1.6: Enables Anna to learn from forecast errors and adapt ethical parameters
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a forecast
pub type ForecastId = String;

/// Snapshot of actual system outcome at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeSnapshot {
    /// Timestamp of the outcome
    pub timestamp: DateTime<Utc>,
    /// Actual system metrics observed
    pub metrics: SystemMetrics,
}

/// System metrics matching chronos::timeline::SystemMetrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub health_score: f64,
    pub empathy_index: f64,
    pub strain_index: f64,
    pub network_coherence: f64,
    pub avg_trust_score: f64,
}

/// Complete audit entry comparing forecast to reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique audit ID
    pub audit_id: String,
    /// Forecast being audited
    pub forecast_id: ForecastId,
    /// When audit was performed
    pub audited_at: DateTime<Utc>,
    /// Forecast generation time
    pub forecast_generated_at: DateTime<Utc>,
    /// Forecast horizon (hours)
    pub horizon_hours: u64,
    /// Predicted final state
    pub predicted: SystemMetrics,
    /// Actual final state
    pub actual: SystemMetrics,
    /// Error metrics
    pub errors: ErrorMetrics,
    /// Overall temporal integrity score [0.0..1.0]
    pub temporal_integrity_score: f64,
    /// Detected biases
    pub bias_findings: Vec<BiasFinding>,
    /// Recommended adjustments (advisory only)
    pub adjustment_plan: Option<AdjustmentPlan>,
}

/// Detailed error metrics for forecast comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Health score error (absolute)
    pub health_error: f64,
    /// Empathy index error (absolute)
    pub empathy_error: f64,
    /// Strain index error (absolute)
    pub strain_error: f64,
    /// Network coherence error (absolute)
    pub coherence_error: f64,
    /// Trust score error (absolute)
    pub trust_error: f64,
    /// Mean absolute error across all metrics
    pub mean_absolute_error: f64,
    /// Root mean squared error
    pub rmse: f64,
}

/// Temporal integrity score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalIntegrityScore {
    /// Overall score [0.0..1.0] (1.0 = perfect prediction)
    pub overall: f64,
    /// Prediction accuracy component [0.0..1.0]
    pub prediction_accuracy: f64,
    /// Ethical alignment component [0.0..1.0]
    pub ethical_alignment: f64,
    /// Coherence stability component [0.0..1.0]
    pub coherence_stability: f64,
    /// Confidence in this score
    pub confidence: f64,
}

/// Types of systematic bias that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiasKind {
    /// Consistently over-optimistic predictions
    ConfirmationBias,
    /// Over-weighting recent data
    RecencyBias,
    /// Over-estimating availability of resources
    AvailabilityBias,
    /// Systematic under-prediction of strain
    StrainUnderestimation,
    /// Systematic over-prediction of health
    HealthOverestimation,
    /// Inconsistent empathy projections
    EmpathyInconsistency,
}

/// A detected bias pattern with evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasFinding {
    /// Type of bias detected
    pub kind: BiasKind,
    /// Confidence in detection [0.0..1.0]
    pub confidence: f64,
    /// Supporting evidence
    pub evidence: String,
    /// Magnitude of bias effect
    pub magnitude: f64,
    /// Number of forecasts exhibiting this pattern
    pub sample_size: usize,
}

/// Advisory adjustment plan (never auto-applied)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentPlan {
    /// Plan ID
    pub plan_id: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Target subsystem
    pub target: AdjustmentTarget,
    /// Recommended parameter adjustments
    pub adjustments: Vec<ParameterAdjustment>,
    /// Expected improvement
    pub expected_improvement: f64,
    /// Justification for these adjustments
    pub rationale: String,
}

/// Which subsystem to adjust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustmentTarget {
    /// Chronos forecast engine
    ChronosForecast,
    /// Conscience ethical evaluation
    Conscience,
    /// Empathy kernel
    Empathy,
    /// Mirror protocol
    Mirror,
}

/// A single parameter adjustment recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterAdjustment {
    /// Parameter name
    pub parameter: String,
    /// Current value (if known)
    pub current_value: Option<f64>,
    /// Recommended new value
    pub recommended_value: f64,
    /// Justification
    pub reason: String,
}

/// Rolling state for mirror audit system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorAuditState {
    /// Last audit run
    pub last_audit_at: Option<DateTime<Utc>>,
    /// Total audits performed
    pub total_audits: usize,
    /// Recent temporal integrity scores (last 10)
    pub recent_scores: Vec<f64>,
    /// Active bias findings
    pub active_biases: Vec<BiasFinding>,
    /// Pending adjustment plans
    pub pending_adjustments: Vec<AdjustmentPlan>,
}

impl Default for MirrorAuditState {
    fn default() -> Self {
        Self {
            last_audit_at: None,
            total_audits: 0,
            recent_scores: Vec::new(),
            active_biases: Vec::new(),
            pending_adjustments: Vec::new(),
        }
    }
}

impl TemporalIntegrityScore {
    /// Calculate overall score from components
    pub fn calculate(
        prediction_accuracy: f64,
        ethical_alignment: f64,
        coherence_stability: f64,
    ) -> Self {
        // Weighted average: 50% accuracy, 30% ethics, 20% coherence
        let overall =
            prediction_accuracy * 0.5 + ethical_alignment * 0.3 + coherence_stability * 0.2;

        // Confidence based on variance
        let variance = ((prediction_accuracy - overall).powi(2)
            + (ethical_alignment - overall).powi(2)
            + (coherence_stability - overall).powi(2))
            / 3.0;
        let confidence = (1.0 - variance).clamp(0.0, 1.0);

        Self {
            overall: overall.clamp(0.0, 1.0),
            prediction_accuracy: prediction_accuracy.clamp(0.0, 1.0),
            ethical_alignment: ethical_alignment.clamp(0.0, 1.0),
            coherence_stability: coherence_stability.clamp(0.0, 1.0),
            confidence,
        }
    }
}

impl BiasKind {
    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            BiasKind::ConfirmationBias => "Consistently over-optimistic predictions",
            BiasKind::RecencyBias => "Over-weighting recent data in forecasts",
            BiasKind::AvailabilityBias => "Over-estimating resource availability",
            BiasKind::StrainUnderestimation => "Systematic under-prediction of system strain",
            BiasKind::HealthOverestimation => "Systematic over-prediction of health scores",
            BiasKind::EmpathyInconsistency => "Inconsistent empathy index projections",
        }
    }
}
