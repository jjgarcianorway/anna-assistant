//! Adjustment plan generation: recommend parameter tuning based on bias findings
//!
//! All adjustments are advisory only - never auto-applied
//! Citation: [archwiki:System_maintenance]

use super::types::{
    AdjustmentPlan, AdjustmentTarget, BiasFinding, BiasKind, ParameterAdjustment,
    TemporalIntegrityScore,
};
use anyhow::Result;
use chrono::Utc;

/// Generate adjustment plan based on bias findings and integrity scores
pub fn generate_adjustment_plan(
    biases: &[BiasFinding],
    integrity_scores: &[TemporalIntegrityScore],
) -> Result<Option<AdjustmentPlan>> {
    if biases.is_empty() {
        return Ok(None); // No biases, no adjustments needed
    }

    // Calculate expected improvement
    let current_avg_score = if integrity_scores.is_empty() {
        0.5
    } else {
        integrity_scores.iter().map(|s| s.overall).sum::<f64>() / integrity_scores.len() as f64
    };

    let expected_improvement = estimate_improvement(biases, current_avg_score);

    // Generate adjustments for each significant bias
    let mut adjustments = Vec::new();
    let mut target = AdjustmentTarget::ChronosForecast; // Default target

    for bias in biases {
        if bias.confidence < 0.7 {
            continue; // Only act on high-confidence findings
        }

        match bias.kind {
            BiasKind::ConfirmationBias => {
                adjustments.extend(adjust_for_confirmation_bias(bias));
                target = AdjustmentTarget::ChronosForecast;
            }
            BiasKind::RecencyBias => {
                adjustments.extend(adjust_for_recency_bias(bias));
                target = AdjustmentTarget::ChronosForecast;
            }
            BiasKind::AvailabilityBias => {
                adjustments.extend(adjust_for_availability_bias(bias));
                target = AdjustmentTarget::ChronosForecast;
            }
            BiasKind::StrainUnderestimation => {
                adjustments.extend(adjust_for_strain_underestimation(bias));
                target = AdjustmentTarget::Empathy;
            }
            BiasKind::HealthOverestimation => {
                adjustments.extend(adjust_for_health_overestimation(bias));
                target = AdjustmentTarget::Conscience;
            }
            BiasKind::EmpathyInconsistency => {
                adjustments.extend(adjust_for_empathy_inconsistency(bias));
                target = AdjustmentTarget::Empathy;
            }
        }
    }

    if adjustments.is_empty() {
        return Ok(None);
    }

    // Build rationale
    let rationale = format!(
        "Detected {} systematic bias pattern(s) with average confidence {:.1}%. \
         Adjustments target {} to improve temporal integrity from {:.1}% to {:.1}%.",
        biases.len(),
        biases.iter().map(|b| b.confidence).sum::<f64>() / biases.len() as f64 * 100.0,
        format!("{:?}", target),
        current_avg_score * 100.0,
        (current_avg_score + expected_improvement) * 100.0
    );

    Ok(Some(AdjustmentPlan {
        plan_id: uuid::Uuid::new_v4().to_string(),
        created_at: Utc::now(),
        target,
        adjustments,
        expected_improvement,
        rationale,
    }))
}

/// Estimate expected improvement from addressing biases
fn estimate_improvement(biases: &[BiasFinding], current_score: f64) -> f64 {
    // Each high-confidence bias reduces score by ~0.1
    // Fixing them should improve by roughly that amount
    let potential_gain: f64 = biases
        .iter()
        .map(|b| b.magnitude * b.confidence * 0.15)
        .sum();

    // Cap improvement at 0.3 (realistic maximum)
    potential_gain.min(0.3).min(1.0 - current_score)
}

/// Generate adjustments for confirmation bias
fn adjust_for_confirmation_bias(bias: &BiasFinding) -> Vec<ParameterAdjustment> {
    vec![
        ParameterAdjustment {
            parameter: "monte_carlo_iterations".to_string(),
            current_value: Some(100.0),
            recommended_value: 150.0,
            reason: format!(
                "Increase simulation diversity to reduce optimism bias (detected magnitude: {:.2})",
                bias.magnitude
            ),
        },
        ParameterAdjustment {
            parameter: "noise_factor".to_string(),
            current_value: Some(0.15),
            recommended_value: 0.20,
            reason: "Increase stochastic variation to capture more pessimistic scenarios"
                .to_string(),
        },
    ]
}

/// Generate adjustments for recency bias
fn adjust_for_recency_bias(bias: &BiasFinding) -> Vec<ParameterAdjustment> {
    vec![ParameterAdjustment {
        parameter: "trend_damping_factor".to_string(),
        current_value: Some(0.95),
        recommended_value: 0.90,
        reason: format!(
            "Reduce trend extrapolation to prevent over-fitting recent data (detected delta: {:.2})",
            bias.magnitude
        ),
    }]
}

/// Generate adjustments for availability bias
fn adjust_for_availability_bias(bias: &BiasFinding) -> Vec<ParameterAdjustment> {
    vec![
        ParameterAdjustment {
            parameter: "strain_sensitivity".to_string(),
            current_value: None,
            recommended_value: 1.2,
            reason: format!(
                "Increase sensitivity to strain indicators (currently underestimated by {:.1}%)",
                bias.magnitude * 100.0
            ),
        },
        ParameterAdjustment {
            parameter: "resource_pessimism_factor".to_string(),
            current_value: None,
            recommended_value: 1.1,
            reason: "Apply conservative multiplier to resource consumption forecasts".to_string(),
        },
    ]
}

/// Generate adjustments for strain underestimation
fn adjust_for_strain_underestimation(bias: &BiasFinding) -> Vec<ParameterAdjustment> {
    vec![ParameterAdjustment {
        parameter: "empathy_strain_coupling".to_string(),
        current_value: None,
        recommended_value: 0.8,
        reason: format!(
            "Strengthen coupling between empathy and strain to improve prediction accuracy (bias magnitude: {:.2})",
            bias.magnitude
        ),
    }]
}

/// Generate adjustments for health overestimation
fn adjust_for_health_overestimation(bias: &BiasFinding) -> Vec<ParameterAdjustment> {
    vec![ParameterAdjustment {
        parameter: "health_score_threshold".to_string(),
        current_value: Some(0.6),
        recommended_value: 0.7,
        reason: format!(
            "Raise health acceptability threshold to account for systematic overestimation (bias: {:.1}%)",
            bias.magnitude * 100.0
        ),
    }]
}

/// Generate adjustments for empathy inconsistency
fn adjust_for_empathy_inconsistency(bias: &BiasFinding) -> Vec<ParameterAdjustment> {
    vec![ParameterAdjustment {
        parameter: "empathy_smoothing_window".to_string(),
        current_value: None,
        recommended_value: 5.0,
        reason: format!(
            "Apply moving average smoothing to reduce empathy prediction variance (inconsistency: {:.1}%)",
            bias.magnitude * 100.0
        ),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_plan_for_confirmation_bias() {
        let biases = vec![BiasFinding {
            kind: BiasKind::ConfirmationBias,
            confidence: 0.85,
            evidence: "Test evidence".to_string(),
            magnitude: 0.25,
            sample_size: 10,
        }];

        let scores = vec![TemporalIntegrityScore {
            overall: 0.7,
            prediction_accuracy: 0.7,
            ethical_alignment: 0.8,
            coherence_stability: 0.75,
            confidence: 0.8,
        }];

        let plan = generate_adjustment_plan(&biases, &scores).unwrap();
        assert!(plan.is_some());

        let plan = plan.unwrap();
        assert!(!plan.adjustments.is_empty());
        assert!(plan.expected_improvement > 0.0);
        assert_eq!(plan.target, AdjustmentTarget::ChronosForecast);
    }

    #[test]
    fn test_no_plan_for_low_confidence_bias() {
        let biases = vec![BiasFinding {
            kind: BiasKind::ConfirmationBias,
            confidence: 0.5, // Below 0.7 threshold
            evidence: "Test evidence".to_string(),
            magnitude: 0.25,
            sample_size: 10,
        }];

        let plan = generate_adjustment_plan(&biases, &[]).unwrap();
        assert!(plan.is_none());
    }
}
