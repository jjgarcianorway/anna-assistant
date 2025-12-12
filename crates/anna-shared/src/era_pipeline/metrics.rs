//! Honest Metrics (Part F) - v0.0.441.
//!
//! Kill fake success metrics.
//!
//! Ticket is RESOLVED only if:
//! - can_answer=true
//! - missing=[]
//! - answer delivered
//!
//! Everything else is NOT resolved.
//! Stats must reflect this or the system is lying.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::evidence::EvidenceBundle;
use super::pipeline::{EraPipeline, PipelineStage, PipelineStatus};
use super::reasoning::ReasoningOutput;

/// Resolution status (honest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResolutionStatus {
    /// Fully resolved: can_answer=true, missing=[], answer delivered.
    Resolved,
    /// Partially resolved: answer given but with caveats.
    Partial,
    /// Cannot answer: missing facts, specialist said no.
    CannotAnswer,
    /// Failed: pipeline error, timeout, etc.
    Failed,
    /// In progress.
    InProgress,
}

impl ResolutionStatus {
    /// Get label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Resolved => "RESOLVED",
            Self::Partial => "PARTIAL",
            Self::CannotAnswer => "CANNOT_ANSWER",
            Self::Failed => "FAILED",
            Self::InProgress => "IN_PROGRESS",
        }
    }

    /// Is this a success for metrics?
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Resolved)
    }

    /// Is this a failure for metrics?
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::CannotAnswer | Self::Failed)
    }
}

/// Resolution criteria (strict).
#[derive(Debug, Clone)]
pub struct ResolutionCriteria {
    /// Reasoning says can_answer.
    pub can_answer: bool,
    /// No missing facts.
    pub no_missing: bool,
    /// Answer was delivered.
    pub answer_delivered: bool,
    /// Confidence threshold met.
    pub confidence_ok: bool,
}

impl ResolutionCriteria {
    /// Check from pipeline state.
    pub fn from_pipeline(
        pipeline: &EraPipeline,
        confidence_threshold: f64,
    ) -> Self {
        let can_answer = pipeline
            .reasoning
            .as_ref()
            .map(|r| r.can_answer)
            .unwrap_or(false);

        let no_missing = pipeline
            .evidence
            .as_ref()
            .map(|e| e.missing.is_empty())
            .unwrap_or(false);

        let answer_delivered = pipeline.answer.is_some();

        let confidence_ok = pipeline
            .reasoning
            .as_ref()
            .map(|r| r.confidence >= confidence_threshold)
            .unwrap_or(false);

        Self {
            can_answer,
            no_missing,
            answer_delivered,
            confidence_ok,
        }
    }

    /// Is fully resolved?
    pub fn is_resolved(&self) -> bool {
        self.can_answer && self.no_missing && self.answer_delivered
    }

    /// Is partially resolved?
    pub fn is_partial(&self) -> bool {
        self.answer_delivered && (!self.can_answer || !self.no_missing || !self.confidence_ok)
    }

    /// Get resolution status.
    pub fn status(&self) -> ResolutionStatus {
        if self.is_resolved() && self.confidence_ok {
            ResolutionStatus::Resolved
        } else if self.is_partial() {
            ResolutionStatus::Partial
        } else if !self.can_answer {
            ResolutionStatus::CannotAnswer
        } else {
            ResolutionStatus::Failed
        }
    }

    /// Get failure reasons.
    pub fn failure_reasons(&self) -> Vec<&'static str> {
        let mut reasons = Vec::new();
        if !self.can_answer {
            reasons.push("Specialist cannot answer from evidence");
        }
        if !self.no_missing {
            reasons.push("Missing required facts");
        }
        if !self.answer_delivered {
            reasons.push("Answer not delivered");
        }
        if !self.confidence_ok {
            reasons.push("Confidence below threshold");
        }
        reasons
    }
}

/// Honest metrics tracker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HonestMetrics {
    /// Counts by status.
    status_counts: HashMap<ResolutionStatus, u64>,
    /// Total tickets.
    total: u64,
    /// Average confidence of resolved tickets.
    avg_confidence: f64,
    /// Sum of confidences (for calculating average).
    confidence_sum: f64,
    /// Confidence count.
    confidence_count: u64,
    /// Fast-path usage.
    fast_path_count: u64,
    /// Fast-path successes.
    fast_path_successes: u64,
}

impl HonestMetrics {
    /// Create empty metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a ticket resolution.
    pub fn record(&mut self, status: ResolutionStatus, confidence: f64) {
        *self.status_counts.entry(status).or_insert(0) += 1;
        self.total += 1;

        if status == ResolutionStatus::Resolved {
            self.confidence_sum += confidence;
            self.confidence_count += 1;
            self.avg_confidence = self.confidence_sum / self.confidence_count as f64;
        }
    }

    /// Record fast-path attempt.
    pub fn record_fast_path(&mut self, success: bool) {
        self.fast_path_count += 1;
        if success {
            self.fast_path_successes += 1;
        }
    }

    /// Get count for status.
    pub fn count(&self, status: ResolutionStatus) -> u64 {
        self.status_counts.get(&status).copied().unwrap_or(0)
    }

    /// Get resolved count (ONLY fully resolved).
    pub fn resolved(&self) -> u64 {
        self.count(ResolutionStatus::Resolved)
    }

    /// Get failed count.
    pub fn failed(&self) -> u64 {
        self.count(ResolutionStatus::CannotAnswer) + self.count(ResolutionStatus::Failed)
    }

    /// Get total.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Get SUCCESS RATE (honest).
    /// Only counts fully resolved tickets as success.
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.resolved() as f64 / self.total as f64
        }
    }

    /// Get failure rate.
    pub fn failure_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.failed() as f64 / self.total as f64
        }
    }

    /// Get fast-path success rate.
    pub fn fast_path_rate(&self) -> f64 {
        if self.fast_path_count == 0 {
            0.0
        } else {
            self.fast_path_successes as f64 / self.fast_path_count as f64
        }
    }

    /// Get average confidence.
    pub fn average_confidence(&self) -> f64 {
        self.avg_confidence
    }

    /// Get summary.
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total: self.total,
            resolved: self.resolved(),
            partial: self.count(ResolutionStatus::Partial),
            cannot_answer: self.count(ResolutionStatus::CannotAnswer),
            failed: self.count(ResolutionStatus::Failed),
            success_rate: self.success_rate(),
            failure_rate: self.failure_rate(),
            avg_confidence: self.avg_confidence,
            fast_path_rate: self.fast_path_rate(),
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Metrics summary for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total tickets.
    pub total: u64,
    /// Fully resolved.
    pub resolved: u64,
    /// Partially resolved.
    pub partial: u64,
    /// Cannot answer.
    pub cannot_answer: u64,
    /// Failed.
    pub failed: u64,
    /// Success rate (resolved / total).
    pub success_rate: f64,
    /// Failure rate.
    pub failure_rate: f64,
    /// Average confidence of resolved.
    pub avg_confidence: f64,
    /// Fast-path success rate.
    pub fast_path_rate: f64,
}

impl MetricsSummary {
    /// Format for logging.
    pub fn log_message(&self) -> String {
        format!(
            "[metrics] total={} resolved={} partial={} cannot_answer={} failed={} \
             success_rate={:.1}% avg_confidence={:.2}",
            self.total,
            self.resolved,
            self.partial,
            self.cannot_answer,
            self.failed,
            self.success_rate * 100.0,
            self.avg_confidence
        )
    }

    /// Format for display.
    pub fn display(&self) -> String {
        format!(
            "Tickets: {} total, {} resolved ({:.0}%), {} failed",
            self.total,
            self.resolved,
            self.success_rate * 100.0,
            self.failed
        )
    }
}

/// Validate pipeline result and return honest status.
pub fn validate_resolution(
    pipeline: &EraPipeline,
    confidence_threshold: f64,
) -> ResolutionStatus {
    // Check pipeline status first
    match pipeline.status() {
        PipelineStatus::Failed => return ResolutionStatus::Failed,
        PipelineStatus::InProgress(_) => return ResolutionStatus::InProgress,
        PipelineStatus::Complete => {}
    }

    // Check resolution criteria
    let criteria = ResolutionCriteria::from_pipeline(pipeline, confidence_threshold);
    criteria.status()
}

/// Resolution reason for user feedback.
#[derive(Debug, Clone)]
pub struct ResolutionReason {
    /// Status.
    pub status: ResolutionStatus,
    /// Human-readable reason.
    pub reason: String,
    /// Missing facts (if any).
    pub missing: Vec<String>,
}

impl ResolutionReason {
    /// Build from pipeline.
    pub fn from_pipeline(pipeline: &EraPipeline, threshold: f64) -> Self {
        let status = validate_resolution(pipeline, threshold);
        let criteria = ResolutionCriteria::from_pipeline(pipeline, threshold);

        let reason = match status {
            ResolutionStatus::Resolved => "Question answered successfully.".to_string(),
            ResolutionStatus::Partial => {
                let reasons = criteria.failure_reasons();
                format!("Partial answer. Issues: {}", reasons.join(", "))
            }
            ResolutionStatus::CannotAnswer => {
                if let Some(reasoning) = &pipeline.reasoning {
                    if !reasoning.requires.is_empty() {
                        format!("Cannot answer. Need: {}", reasoning.requires.join(", "))
                    } else {
                        "Cannot answer from available evidence.".to_string()
                    }
                } else {
                    "Cannot answer. Reasoning stage failed.".to_string()
                }
            }
            ResolutionStatus::Failed => {
                if !pipeline.errors.is_empty() {
                    format!("Failed: {}", pipeline.errors[0].message)
                } else {
                    "Pipeline failed.".to_string()
                }
            }
            ResolutionStatus::InProgress => {
                format!("In progress at stage: {}", pipeline.stage.label())
            }
        };

        let missing = pipeline
            .evidence
            .as_ref()
            .map(|e| e.missing.clone())
            .unwrap_or_default();

        Self {
            status,
            reason,
            missing,
        }
    }
}

/// Default confidence threshold.
pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.6;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::era_pipeline::evidence::EvidenceBundle;
    use crate::era_pipeline::reasoning::ReasoningOutput;

    #[test]
    fn test_resolution_criteria_resolved() {
        let mut pipeline = EraPipeline::new("DSK-0127");
        pipeline.evidence = Some(EvidenceBundle::new("DSK-0127"));
        pipeline.reasoning = Some(ReasoningOutput::answerable("DSK-0127", "Test", 0.9));
        pipeline.answer = Some("17.0 GiB".to_string());

        let criteria = ResolutionCriteria::from_pipeline(&pipeline, 0.6);
        assert!(criteria.is_resolved());
        assert_eq!(criteria.status(), ResolutionStatus::Resolved);
    }

    #[test]
    fn test_resolution_criteria_cannot_answer() {
        let mut pipeline = EraPipeline::new("DSK-0127");
        pipeline.evidence = Some(EvidenceBundle::new("DSK-0127"));
        pipeline.reasoning = Some(ReasoningOutput::unanswerable(
            "DSK-0127",
            "Missing data",
            vec!["boot.blame"],
        ));

        let criteria = ResolutionCriteria::from_pipeline(&pipeline, 0.6);
        assert!(!criteria.can_answer);
        assert_eq!(criteria.status(), ResolutionStatus::CannotAnswer);
    }

    #[test]
    fn test_honest_metrics() {
        let mut metrics = HonestMetrics::new();

        metrics.record(ResolutionStatus::Resolved, 0.9);
        metrics.record(ResolutionStatus::Resolved, 0.85);
        metrics.record(ResolutionStatus::CannotAnswer, 0.0);
        metrics.record(ResolutionStatus::Failed, 0.0);

        assert_eq!(metrics.total(), 4);
        assert_eq!(metrics.resolved(), 2);
        assert_eq!(metrics.failed(), 2);
        assert!((metrics.success_rate() - 0.5).abs() < 0.01);
        assert!((metrics.average_confidence() - 0.875).abs() < 0.01);
    }

    #[test]
    fn test_metrics_summary() {
        let mut metrics = HonestMetrics::new();
        metrics.record(ResolutionStatus::Resolved, 0.9);
        metrics.record(ResolutionStatus::Partial, 0.5);
        metrics.record(ResolutionStatus::CannotAnswer, 0.0);

        let summary = metrics.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.resolved, 1);
        assert_eq!(summary.partial, 1);
        assert_eq!(summary.cannot_answer, 1);
    }

    #[test]
    fn test_fast_path_tracking() {
        let mut metrics = HonestMetrics::new();
        metrics.record_fast_path(true);
        metrics.record_fast_path(true);
        metrics.record_fast_path(false);

        assert_eq!(metrics.fast_path_count, 3);
        assert_eq!(metrics.fast_path_successes, 2);
        assert!((metrics.fast_path_rate() - 0.666).abs() < 0.01);
    }
}
