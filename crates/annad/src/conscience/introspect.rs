//! Introspection and self-review system for conscience layer
//!
//! Phase 1.1: Periodic self-reflection on past decisions
//! Citation: [archwiki:System_maintenance]

use super::types::{
    ConscienceDecision, ConscienceState, DecisionOutcome, EthicalViolation, IntrospectionReport,
};
use anyhow::Result;
use chrono::{Duration, Utc};
use tracing::{debug, info};

/// Introspection engine for self-review
pub struct Introspector {
    /// Minimum ethical score threshold
    ethical_threshold: f64,
    /// Minimum confidence threshold
    confidence_threshold: f64,
}

impl Introspector {
    /// Create new introspector
    pub fn new(ethical_threshold: f64, confidence_threshold: f64) -> Self {
        Self {
            ethical_threshold,
            confidence_threshold,
        }
    }

    /// Perform introspection on conscience state
    pub async fn introspect(&self, state: &ConscienceState) -> Result<IntrospectionReport> {
        info!("Beginning conscience introspection...");

        // Determine time period for analysis (last 6 hours)
        let now = Utc::now();
        let period_start = now - Duration::hours(6);
        let period_end = now;

        // Filter decisions within time period
        let recent_decisions: Vec<&ConscienceDecision> = state
            .decision_history
            .iter()
            .filter(|d| d.timestamp >= period_start && d.timestamp <= period_end)
            .collect();

        debug!("Analyzing {} recent decisions", recent_decisions.len());

        // Count decision outcomes
        let mut approved_count = 0;
        let mut rejected_count = 0;
        let mut flagged_count = 0;

        for decision in &recent_decisions {
            match &decision.outcome {
                DecisionOutcome::Approved { .. } => approved_count += 1,
                DecisionOutcome::Rejected { .. } => rejected_count += 1,
                DecisionOutcome::Flagged { .. } => flagged_count += 1,
                DecisionOutcome::Pending => {}
            }
        }

        // Calculate average scores
        let avg_ethical_score = if !recent_decisions.is_empty() {
            recent_decisions
                .iter()
                .map(|d| d.ethical_score.overall())
                .sum::<f64>()
                / recent_decisions.len() as f64
        } else {
            0.0
        };

        let avg_confidence = if !recent_decisions.is_empty() {
            recent_decisions.iter().map(|d| d.confidence).sum::<f64>()
                / recent_decisions.len() as f64
        } else {
            0.0
        };

        // Detect ethical violations
        let violations = self.detect_violations(&recent_decisions);

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            approved_count,
            rejected_count,
            flagged_count,
            avg_ethical_score,
            avg_confidence,
            &violations,
        );

        let report = IntrospectionReport {
            timestamp: now,
            period_start,
            period_end,
            decisions_reviewed: recent_decisions.len() as u64,
            approved_count,
            rejected_count,
            flagged_count,
            avg_ethical_score,
            avg_confidence,
            violations,
            recommendations,
        };

        info!(
            "Introspection complete: {} decisions reviewed, {} violations detected",
            report.decisions_reviewed,
            report.violations.len()
        );

        Ok(report)
    }

    /// Detect ethical violations in decisions
    fn detect_violations(&self, decisions: &[&ConscienceDecision]) -> Vec<EthicalViolation> {
        let mut violations = Vec::new();

        for decision in decisions {
            // Check if ethical score is below threshold
            if decision.ethical_score.overall() < self.ethical_threshold {
                violations.push(EthicalViolation {
                    timestamp: decision.timestamp,
                    action: decision.action.clone(),
                    dimension: "overall".to_string(),
                    score: decision.ethical_score.overall(),
                    threshold: self.ethical_threshold,
                    description: format!(
                        "Overall ethical score ({:.1}%) below threshold ({:.1}%)",
                        decision.ethical_score.overall() * 100.0,
                        self.ethical_threshold * 100.0
                    ),
                });
            }

            // Check individual dimensions
            if decision.ethical_score.safety < self.ethical_threshold {
                violations.push(EthicalViolation {
                    timestamp: decision.timestamp,
                    action: decision.action.clone(),
                    dimension: "safety".to_string(),
                    score: decision.ethical_score.safety,
                    threshold: self.ethical_threshold,
                    description: format!(
                        "Safety score ({:.1}%) below threshold ({:.1}%)",
                        decision.ethical_score.safety * 100.0,
                        self.ethical_threshold * 100.0
                    ),
                });
            }

            if decision.ethical_score.privacy < self.ethical_threshold {
                violations.push(EthicalViolation {
                    timestamp: decision.timestamp,
                    action: decision.action.clone(),
                    dimension: "privacy".to_string(),
                    score: decision.ethical_score.privacy,
                    threshold: self.ethical_threshold,
                    description: format!(
                        "Privacy score ({:.1}%) below threshold ({:.1}%)",
                        decision.ethical_score.privacy * 100.0,
                        self.ethical_threshold * 100.0
                    ),
                });
            }

            if decision.ethical_score.integrity < self.ethical_threshold {
                violations.push(EthicalViolation {
                    timestamp: decision.timestamp,
                    action: decision.action.clone(),
                    dimension: "integrity".to_string(),
                    score: decision.ethical_score.integrity,
                    threshold: self.ethical_threshold,
                    description: format!(
                        "Integrity score ({:.1}%) below threshold ({:.1}%)",
                        decision.ethical_score.integrity * 100.0,
                        self.ethical_threshold * 100.0
                    ),
                });
            }

            if decision.ethical_score.autonomy < self.ethical_threshold {
                violations.push(EthicalViolation {
                    timestamp: decision.timestamp,
                    action: decision.action.clone(),
                    dimension: "autonomy".to_string(),
                    score: decision.ethical_score.autonomy,
                    threshold: self.ethical_threshold,
                    description: format!(
                        "Autonomy score ({:.1}%) below threshold ({:.1}%)",
                        decision.ethical_score.autonomy * 100.0,
                        self.ethical_threshold * 100.0
                    ),
                });
            }

            // Check confidence
            if decision.confidence < self.confidence_threshold {
                violations.push(EthicalViolation {
                    timestamp: decision.timestamp,
                    action: decision.action.clone(),
                    dimension: "confidence".to_string(),
                    score: decision.confidence,
                    threshold: self.confidence_threshold,
                    description: format!(
                        "Confidence ({:.1}%) below threshold ({:.1}%)",
                        decision.confidence * 100.0,
                        self.confidence_threshold * 100.0
                    ),
                });
            }
        }

        violations
    }

    /// Generate recommendations based on introspection analysis
    fn generate_recommendations(
        &self,
        approved: u64,
        rejected: u64,
        flagged: u64,
        avg_ethical: f64,
        avg_confidence: f64,
        violations: &[EthicalViolation],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Recommendation based on approval rate
        let total = approved + rejected + flagged;
        if total > 0 {
            let approval_rate = approved as f64 / total as f64;

            if approval_rate < 0.5 {
                recommendations.push(format!(
                    "Low approval rate ({:.1}%). Consider reviewing ethical thresholds.",
                    approval_rate * 100.0
                ));
            }

            if flagged as f64 / total as f64 > 0.3 {
                recommendations.push(
                    "High flagging rate (>30%). Consider tuning uncertainty threshold."
                        .to_string(),
                );
            }
        }

        // Recommendation based on ethical score
        if avg_ethical < 0.7 {
            recommendations.push(format!(
                "Average ethical score is low ({:.1}%). Review automated action policies.",
                avg_ethical * 100.0
            ));
        }

        // Recommendation based on confidence
        if avg_confidence < 0.6 {
            recommendations.push(format!(
                "Average confidence is low ({:.1}%). System may need more training data.",
                avg_confidence * 100.0
            ));
        }

        // Recommendations based on violations
        if !violations.is_empty() {
            let violation_count_by_dim = violations
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, v| {
                    *acc.entry(v.dimension.clone()).or_insert(0) += 1;
                    acc
                });

            for (dimension, count) in violation_count_by_dim {
                if count > 2 {
                    recommendations.push(format!(
                        "Repeated violations in {} dimension ({} occurrences). Review evaluation logic.",
                        dimension, count
                    ));
                }
            }
        }

        // Default recommendation if everything looks good
        if recommendations.is_empty() && total > 0 {
            recommendations.push("Conscience system operating within normal parameters.".to_string());
        }

        recommendations
    }

    /// Format introspection report for display
    pub fn format_report(report: &IntrospectionReport) -> String {
        let mut output = String::new();

        output.push_str("╔═══════════════════════════════════════════════════════════╗\n");
        output.push_str("║ CONSCIENCE INTROSPECTION REPORT                           ║\n");
        output.push_str("╚═══════════════════════════════════════════════════════════╝\n\n");

        output.push_str(&format!(
            "Report Generated: {}\n",
            report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!(
            "Analysis Period:  {} to {}\n\n",
            report.period_start.format("%H:%M:%S"),
            report.period_end.format("%H:%M:%S")
        ));

        output.push_str("DECISION SUMMARY:\n");
        output.push_str(&format!(
            "  Total Reviewed: {}\n",
            report.decisions_reviewed
        ));
        output.push_str(&format!("  Approved:       {}\n", report.approved_count));
        output.push_str(&format!("  Rejected:       {}\n", report.rejected_count));
        output.push_str(&format!("  Flagged:        {}\n\n", report.flagged_count));

        output.push_str("QUALITY METRICS:\n");
        output.push_str(&format!(
            "  Avg Ethical Score: {:.1}%\n",
            report.avg_ethical_score * 100.0
        ));
        output.push_str(&format!(
            "  Avg Confidence:    {:.1}%\n\n",
            report.avg_confidence * 100.0
        ));

        if !report.violations.is_empty() {
            output.push_str(&format!(
                "VIOLATIONS DETECTED: {}\n",
                report.violations.len()
            ));
            for (i, violation) in report.violations.iter().take(5).enumerate() {
                output.push_str(&format!(
                    "  {}. {} - {}\n",
                    i + 1,
                    violation.dimension,
                    violation.description
                ));
            }
            if report.violations.len() > 5 {
                output.push_str(&format!(
                    "  ... and {} more\n",
                    report.violations.len() - 5
                ));
            }
            output.push('\n');
        }

        output.push_str("RECOMMENDATIONS:\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, rec));
        }

        output.push_str("\n───────────────────────────────────────────────────────────\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conscience::types::EthicalScore;
    use crate::sentinel::SentinelAction;

    #[test]
    fn test_violation_detection() {
        let introspector = Introspector::new(0.7, 0.6);

        let decision = ConscienceDecision {
            id: "test".to_string(),
            timestamp: Utc::now(),
            action: SentinelAction::None,
            reasoning: crate::conscience::types::ReasoningTree::new("test".to_string()),
            ethical_score: EthicalScore {
                safety: 0.5, // Below threshold
                privacy: 0.8,
                integrity: 0.8,
                autonomy: 0.8,
            },
            confidence: 0.8,
            outcome: DecisionOutcome::Pending,
            rollback_plan: None,
        };

        let violations = introspector.detect_violations(&[&decision]);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.dimension == "safety"));
    }

    #[test]
    fn test_recommendations() {
        let introspector = Introspector::new(0.7, 0.6);

        // Low approval rate should generate recommendation
        let recs = introspector.generate_recommendations(2, 8, 0, 0.8, 0.8, &[]);
        assert!(!recs.is_empty());

        // High flagging rate should generate recommendation
        let recs = introspector.generate_recommendations(5, 0, 5, 0.8, 0.8, &[]);
        assert!(recs.iter().any(|r| r.contains("flagging rate")));
    }
}
