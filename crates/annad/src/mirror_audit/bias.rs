//! Bias detection: identify systematic forecast errors
//!
//! Analyzes patterns across multiple audits to detect confirmation, recency, and availability biases
//! Citation: [archwiki:System_maintenance]

use super::align::{detect_directional_bias, DirectionalBias};
use super::types::{AuditEntry, BiasFinding, BiasKind, SystemMetrics};
use anyhow::Result;

/// Minimum sample size for bias detection
const MIN_SAMPLE_SIZE: usize = 5;

/// Confidence threshold for reporting a bias
const MIN_CONFIDENCE: f64 = 0.6;

/// Scan audit history for systematic biases
pub fn scan_for_biases(audits: &[AuditEntry]) -> Result<Vec<BiasFinding>> {
    if audits.len() < MIN_SAMPLE_SIZE {
        return Ok(Vec::new()); // Not enough data
    }

    let mut findings = Vec::new();

    // Check for confirmation bias (over-optimism)
    if let Some(finding) = detect_confirmation_bias(audits)? {
        findings.push(finding);
    }

    // Check for recency bias
    if let Some(finding) = detect_recency_bias(audits)? {
        findings.push(finding);
    }

    // Check for availability bias
    if let Some(finding) = detect_availability_bias(audits)? {
        findings.push(finding);
    }

    // Check for directional biases using align module
    if let Some(finding) = detect_systematic_directional_bias(audits)? {
        findings.push(finding);
    }

    // Filter by minimum confidence
    Ok(findings
        .into_iter()
        .filter(|f| f.confidence >= MIN_CONFIDENCE)
        .collect())
}

/// Detect confirmation bias: consistently over-optimistic predictions
fn detect_confirmation_bias(audits: &[AuditEntry]) -> Result<Option<BiasFinding>> {
    // Count how often we over-predicted positive metrics
    let mut optimistic_count = 0;
    let mut total_predictions = 0;

    for audit in audits {
        // Check each metric for optimistic bias
        if audit.predicted.health_score > audit.actual.health_score + 0.1 {
            optimistic_count += 1;
        }
        if audit.predicted.empathy_index > audit.actual.empathy_index + 0.1 {
            optimistic_count += 1;
        }
        if audit.predicted.strain_index < audit.actual.strain_index - 0.1 {
            // Predicting lower strain than actual
            optimistic_count += 1;
        }
        total_predictions += 3;
    }

    let optimism_rate = optimistic_count as f64 / total_predictions as f64;

    // Threshold: >60% of predictions are optimistic
    if optimism_rate > 0.6 {
        let confidence = ((optimism_rate - 0.6) / 0.4).clamp(0.6, 1.0);

        Ok(Some(BiasFinding {
            kind: BiasKind::ConfirmationBias,
            confidence,
            evidence: format!(
                "{}% of predictions were over-optimistic (sample size: {})",
                (optimism_rate * 100.0) as u32,
                audits.len()
            ),
            magnitude: optimism_rate,
            sample_size: audits.len(),
        }))
    } else {
        Ok(None)
    }
}

/// Detect recency bias: recent forecasts differ significantly from older ones
fn detect_recency_bias(audits: &[AuditEntry]) -> Result<Option<BiasFinding>> {
    if audits.len() < 10 {
        return Ok(None); // Need more data
    }

    // Compare recent 20% vs older 80%
    let split_point = (audits.len() as f64 * 0.8) as usize;
    let older = &audits[..split_point];
    let recent = &audits[split_point..];

    let older_avg_error: f64 = older
        .iter()
        .map(|a| a.errors.mean_absolute_error)
        .sum::<f64>()
        / older.len() as f64;
    let recent_avg_error: f64 = recent
        .iter()
        .map(|a| a.errors.mean_absolute_error)
        .sum::<f64>()
        / recent.len() as f64;

    let error_delta = (recent_avg_error - older_avg_error).abs();

    // Threshold: >0.2 difference in error rates
    if error_delta > 0.2 {
        let confidence = (error_delta / 0.5).clamp(0.6, 1.0);

        Ok(Some(BiasFinding {
            kind: BiasKind::RecencyBias,
            confidence,
            evidence: format!(
                "Recent forecasts have {:.1}% different error rate than historical (delta: {:.3})",
                (error_delta * 100.0),
                error_delta
            ),
            magnitude: error_delta,
            sample_size: audits.len(),
        }))
    } else {
        Ok(None)
    }
}

/// Detect availability bias: over-estimating resource availability
fn detect_availability_bias(audits: &[AuditEntry]) -> Result<Option<BiasFinding>> {
    // Check if we consistently under-predict strain and over-predict health
    let mut underestimated_strain = 0;
    let mut overestimated_health = 0;

    for audit in audits {
        if audit.predicted.strain_index < audit.actual.strain_index - 0.1 {
            underestimated_strain += 1;
        }
        if audit.predicted.health_score > audit.actual.health_score + 0.1 {
            overestimated_health += 1;
        }
    }

    let strain_bias_rate = underestimated_strain as f64 / audits.len() as f64;
    let health_bias_rate = overestimated_health as f64 / audits.len() as f64;

    // Availability bias shows up as both strain underestimation and health overestimation
    let combined_bias_rate = (strain_bias_rate + health_bias_rate) / 2.0;

    if combined_bias_rate > 0.5 {
        let confidence = ((combined_bias_rate - 0.5) / 0.5).clamp(0.6, 1.0);

        Ok(Some(BiasFinding {
            kind: BiasKind::AvailabilityBias,
            confidence,
            evidence: format!(
                "Systematically underestimated strain ({:.0}%) and overestimated health ({:.0}%)",
                strain_bias_rate * 100.0,
                health_bias_rate * 100.0
            ),
            magnitude: combined_bias_rate,
            sample_size: audits.len(),
        }))
    } else {
        Ok(None)
    }
}

/// Detect systematic directional biases using align module
fn detect_systematic_directional_bias(audits: &[AuditEntry]) -> Result<Option<BiasFinding>> {
    let predicted: Vec<SystemMetrics> = audits.iter().map(|a| a.predicted.clone()).collect();
    let actual: Vec<SystemMetrics> = audits.iter().map(|a| a.actual.clone()).collect();

    match detect_directional_bias(&predicted, &actual)? {
        DirectionalBias::HealthOverestimation(mag) => Ok(Some(BiasFinding {
            kind: BiasKind::HealthOverestimation,
            confidence: (mag / 0.3).clamp(0.6, 1.0),
            evidence: format!(
                "Systematic health overestimation by {:.1}% on average",
                mag * 100.0
            ),
            magnitude: mag,
            sample_size: audits.len(),
        })),
        DirectionalBias::StrainUnderestimation(mag) => Ok(Some(BiasFinding {
            kind: BiasKind::StrainUnderestimation,
            confidence: (mag / 0.3).clamp(0.6, 1.0),
            evidence: format!(
                "Systematic strain underestimation by {:.1}% on average",
                mag * 100.0
            ),
            magnitude: mag,
            sample_size: audits.len(),
        })),
        DirectionalBias::EmpathyInconsistency(mag) => Ok(Some(BiasFinding {
            kind: BiasKind::EmpathyInconsistency,
            confidence: (mag / 0.3).clamp(0.6, 1.0),
            evidence: format!(
                "Inconsistent empathy predictions with {:.1}% average deviation",
                mag * 100.0
            ),
            magnitude: mag,
            sample_size: audits.len(),
        })),
        DirectionalBias::None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_audit(predicted: SystemMetrics, actual: SystemMetrics) -> AuditEntry {
        let errors = super::super::align::compute_errors(&predicted, &actual);

        AuditEntry {
            audit_id: uuid::Uuid::new_v4().to_string(),
            forecast_id: uuid::Uuid::new_v4().to_string(),
            audited_at: Utc::now(),
            forecast_generated_at: Utc::now(),
            horizon_hours: 24,
            predicted,
            actual,
            errors,
            temporal_integrity_score: 0.8,
            bias_findings: Vec::new(),
            adjustment_plan: None,
        }
    }

    #[test]
    fn test_confirmation_bias_detection() {
        // Create 10 audits with consistently optimistic predictions
        let mut audits = Vec::new();

        for _ in 0..10 {
            let predicted = SystemMetrics {
                health_score: 0.9,
                empathy_index: 0.85,
                strain_index: 0.2,
                network_coherence: 0.8,
                avg_trust_score: 0.8,
            };

            let actual = SystemMetrics {
                health_score: 0.7,  // Lower than predicted
                empathy_index: 0.7, // Lower than predicted
                strain_index: 0.4,  // Higher than predicted
                network_coherence: 0.8,
                avg_trust_score: 0.8,
            };

            audits.push(create_test_audit(predicted, actual));
        }

        let finding = detect_confirmation_bias(&audits).unwrap();
        assert!(finding.is_some(), "Should detect confirmation bias");

        let finding = finding.unwrap();
        assert_eq!(finding.kind, BiasKind::ConfirmationBias);
        assert!(finding.confidence >= MIN_CONFIDENCE);
    }

    #[test]
    fn test_no_bias_with_accurate_predictions() {
        let mut audits = Vec::new();

        for _ in 0..10 {
            let metrics = SystemMetrics {
                health_score: 0.8,
                empathy_index: 0.75,
                strain_index: 0.3,
                network_coherence: 0.8,
                avg_trust_score: 0.8,
            };

            audits.push(create_test_audit(metrics.clone(), metrics));
        }

        let findings = scan_for_biases(&audits).unwrap();
        assert_eq!(
            findings.len(),
            0,
            "Should detect no biases with perfect predictions"
        );
    }
}
