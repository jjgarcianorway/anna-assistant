//! Alignment module: compare predicted vs actual outcomes
//!
//! Computes error vectors and temporal integrity scores
//! Citation: [archwiki:System_maintenance]

use super::types::{ErrorMetrics, OutcomeSnapshot, SystemMetrics, TemporalIntegrityScore};
use anyhow::Result;

/// Compare predicted vs actual metrics and compute error vector
pub fn compute_errors(predicted: &SystemMetrics, actual: &SystemMetrics) -> ErrorMetrics {
    let health_error = (predicted.health_score - actual.health_score).abs();
    let empathy_error = (predicted.empathy_index - actual.empathy_index).abs();
    let strain_error = (predicted.strain_index - actual.strain_index).abs();
    let coherence_error = (predicted.network_coherence - actual.network_coherence).abs();
    let trust_error = (predicted.avg_trust_score - actual.avg_trust_score).abs();

    let mean_absolute_error =
        (health_error + empathy_error + strain_error + coherence_error + trust_error) / 5.0;

    let rmse = ((health_error.powi(2)
        + empathy_error.powi(2)
        + strain_error.powi(2)
        + coherence_error.powi(2)
        + trust_error.powi(2))
        / 5.0)
        .sqrt();

    ErrorMetrics {
        health_error,
        empathy_error,
        strain_error,
        coherence_error,
        trust_error,
        mean_absolute_error,
        rmse,
    }
}

/// Calculate temporal integrity score from error metrics
pub fn calculate_temporal_integrity(
    errors: &ErrorMetrics,
    predicted: &SystemMetrics,
    actual: &SystemMetrics,
) -> Result<TemporalIntegrityScore> {
    // Prediction accuracy: inverse of mean absolute error
    // MAE of 0.0 = 100% accurate, MAE of 1.0 = 0% accurate
    let prediction_accuracy = (1.0 - errors.mean_absolute_error).clamp(0.0, 1.0);

    // Ethical alignment: how well did we preserve ethical trajectory?
    // Check if both predicted and actual stayed in acceptable ranges
    let predicted_ethical = is_ethically_acceptable(predicted);
    let actual_ethical = is_ethically_acceptable(actual);

    let ethical_alignment = if predicted_ethical == actual_ethical {
        // Correct ethical prediction
        1.0
    } else if actual_ethical && !predicted_ethical {
        // Predicted unethical but was ethical (pessimistic)
        0.7
    } else {
        // Predicted ethical but was unethical (dangerous miss)
        0.3
    };

    // Coherence stability: did network coherence remain stable?
    let coherence_delta = (predicted.network_coherence - actual.network_coherence).abs();
    let coherence_stability = (1.0 - coherence_delta).clamp(0.0, 1.0);

    Ok(TemporalIntegrityScore::calculate(
        prediction_accuracy,
        ethical_alignment,
        coherence_stability,
    ))
}

/// Check if metrics are in ethically acceptable ranges
fn is_ethically_acceptable(metrics: &SystemMetrics) -> bool {
    metrics.health_score >= 0.6
        && metrics.strain_index <= 0.7
        && metrics.empathy_index >= 0.5
        && metrics.network_coherence >= 0.6
}

/// Calculate prediction accuracy from a window of audits
pub fn calculate_rolling_accuracy(recent_errors: &[ErrorMetrics]) -> f64 {
    if recent_errors.is_empty() {
        return 0.5; // Unknown, neutral
    }

    let avg_mae: f64 = recent_errors.iter().map(|e| e.mean_absolute_error).sum::<f64>()
        / recent_errors.len() as f64;

    (1.0 - avg_mae).clamp(0.0, 1.0)
}

/// Detect if there's a systematic directional bias in predictions
pub fn detect_directional_bias(
    predicted: &[SystemMetrics],
    actual: &[SystemMetrics],
) -> Result<DirectionalBias> {
    if predicted.len() != actual.len() || predicted.is_empty() {
        return Ok(DirectionalBias::None);
    }

    // Calculate average residuals for each metric
    let mut health_residuals = Vec::new();
    let mut strain_residuals = Vec::new();
    let mut empathy_residuals = Vec::new();

    for (pred, act) in predicted.iter().zip(actual.iter()) {
        health_residuals.push(pred.health_score - act.health_score);
        strain_residuals.push(pred.strain_index - act.strain_index);
        empathy_residuals.push(pred.empathy_index - act.empathy_index);
    }

    let avg_health_bias: f64 = health_residuals.iter().sum::<f64>() / health_residuals.len() as f64;
    let avg_strain_bias: f64 = strain_residuals.iter().sum::<f64>() / strain_residuals.len() as f64;
    let avg_empathy_bias: f64 =
        empathy_residuals.iter().sum::<f64>() / empathy_residuals.len() as f64;

    // Detect systematic bias (threshold: 0.15 consistent error)
    if avg_health_bias > 0.15 {
        Ok(DirectionalBias::HealthOverestimation(avg_health_bias))
    } else if avg_strain_bias < -0.15 {
        Ok(DirectionalBias::StrainUnderestimation(avg_strain_bias.abs()))
    } else if avg_empathy_bias.abs() > 0.15 {
        Ok(DirectionalBias::EmpathyInconsistency(avg_empathy_bias.abs()))
    } else {
        Ok(DirectionalBias::None)
    }
}

/// Directional bias detection result
#[derive(Debug, Clone)]
pub enum DirectionalBias {
    None,
    HealthOverestimation(f64),
    StrainUnderestimation(f64),
    EmpathyInconsistency(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_errors_perfect_prediction() {
        let metrics = SystemMetrics {
            health_score: 0.8,
            empathy_index: 0.75,
            strain_index: 0.3,
            network_coherence: 0.85,
            avg_trust_score: 0.8,
        };

        let errors = compute_errors(&metrics, &metrics);

        assert_eq!(errors.health_error, 0.0);
        assert_eq!(errors.mean_absolute_error, 0.0);
        assert_eq!(errors.rmse, 0.0);
    }

    #[test]
    fn test_temporal_integrity_perfect() {
        let predicted = SystemMetrics {
            health_score: 0.8,
            empathy_index: 0.75,
            strain_index: 0.3,
            network_coherence: 0.85,
            avg_trust_score: 0.8,
        };

        let errors = compute_errors(&predicted, &predicted);
        let score = calculate_temporal_integrity(&errors, &predicted, &predicted).unwrap();

        assert!(score.overall > 0.95);
        assert!(score.prediction_accuracy > 0.95);
    }

    #[test]
    fn test_directional_bias_detection() {
        // Create predictions that consistently overestimate health
        let predicted: Vec<SystemMetrics> = (0..10)
            .map(|_| SystemMetrics {
                health_score: 0.9,
                empathy_index: 0.7,
                strain_index: 0.3,
                network_coherence: 0.8,
                avg_trust_score: 0.8,
            })
            .collect();

        let actual: Vec<SystemMetrics> = (0..10)
            .map(|_| SystemMetrics {
                health_score: 0.7, // Actual is lower
                empathy_index: 0.7,
                strain_index: 0.3,
                network_coherence: 0.8,
                avg_trust_score: 0.8,
            })
            .collect();

        let bias = detect_directional_bias(&predicted, &actual).unwrap();

        match bias {
            DirectionalBias::HealthOverestimation(mag) => {
                assert!(mag > 0.15, "Should detect health overestimation");
            }
            _ => panic!("Should detect health overestimation bias"),
        }
    }
}
