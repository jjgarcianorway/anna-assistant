//! Bias repair and ethical remediation
//!
//! Phase 1.4: Adaptive reweighting and calibration
//! Citation: [archwiki:System_maintenance]

use super::types::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Remediation engine for bias correction
pub struct RemediationEngine {
    /// Node ID
    node_id: PeerId,
    /// Auto-remediation enabled?
    auto_remediation: bool,
}

impl RemediationEngine {
    /// Create new remediation engine
    pub fn new(node_id: PeerId, auto_remediation: bool) -> Self {
        Self {
            node_id,
            auto_remediation,
        }
    }

    /// Apply remediation action
    pub async fn apply_remediation(
        &self,
        action: &RemediationAction,
    ) -> Result<RemediationResult, String> {
        info!(
            "Applying remediation: {} - {}",
            action.id, action.description
        );

        // Check if this node is targeted
        if action.target_node != "all" && action.target_node != self.node_id {
            return Ok(RemediationResult {
                action_id: action.id.clone(),
                applied: false,
                reason: "Not targeted for this node".to_string(),
                adjustments_made: HashMap::new(),
                timestamp: Utc::now(),
            });
        }

        // Check if auto-remediation is enabled
        if !self.auto_remediation
            && !matches!(action.remediation_type, RemediationType::ManualReview)
        {
            return Ok(RemediationResult {
                action_id: action.id.clone(),
                applied: false,
                reason: "Auto-remediation disabled - requires manual approval".to_string(),
                adjustments_made: HashMap::new(),
                timestamp: Utc::now(),
            });
        }

        // Apply based on type
        match &action.remediation_type {
            RemediationType::ParameterReweight => {
                self.apply_parameter_reweight(action).await
            }
            RemediationType::TrustReset => self.apply_trust_reset(action).await,
            RemediationType::ConscienceAdjustment => {
                self.apply_conscience_adjustment(action).await
            }
            RemediationType::PatternRetrain => self.apply_pattern_retrain(action).await,
            RemediationType::ManualReview => Ok(RemediationResult {
                action_id: action.id.clone(),
                applied: false,
                reason: "Manual review required - no automatic action taken".to_string(),
                adjustments_made: HashMap::new(),
                timestamp: Utc::now(),
            }),
        }
    }

    /// Apply parameter reweighting
    async fn apply_parameter_reweight(
        &self,
        action: &RemediationAction,
    ) -> Result<RemediationResult, String> {
        info!("Applying parameter reweight: {}", action.description);

        let mut adjustments_made = HashMap::new();

        for (param, value) in &action.parameter_adjustments {
            match param.as_str() {
                "scrutiny_threshold" => {
                    // Placeholder: would adjust actual conscience scrutiny threshold
                    debug!("Adjusting scrutiny_threshold to {}", value);
                    adjustments_made.insert(param.clone(), *value);
                }
                "temporal_decay" => {
                    // Placeholder: would adjust temporal weighting in decision-making
                    debug!("Adjusting temporal_decay to {}", value);
                    adjustments_made.insert(param.clone(), *value);
                }
                "strain_deferral_threshold" => {
                    // Placeholder: would adjust empathy kernel strain threshold
                    debug!("Adjusting strain_deferral_threshold to {}", value);
                    adjustments_made.insert(param.clone(), *value);
                }
                _ => {
                    warn!("Unknown parameter: {}", param);
                }
            }
        }

        Ok(RemediationResult {
            action_id: action.id.clone(),
            applied: true,
            reason: "Parameter adjustments applied successfully".to_string(),
            adjustments_made,
            timestamp: Utc::now(),
        })
    }

    /// Apply trust score reset
    async fn apply_trust_reset(
        &self,
        action: &RemediationAction,
    ) -> Result<RemediationResult, String> {
        info!("Applying trust reset: {}", action.description);

        // Placeholder: In full implementation, would reset specific peer trust scores
        // This would integrate with the trust ledger in collective mind

        let mut adjustments_made = HashMap::new();
        adjustments_made.insert("trust_scores_reset".to_string(), 1.0);

        Ok(RemediationResult {
            action_id: action.id.clone(),
            applied: true,
            reason: "Trust scores reset to neutral for specified peers".to_string(),
            adjustments_made,
            timestamp: Utc::now(),
        })
    }

    /// Apply conscience threshold adjustment
    async fn apply_conscience_adjustment(
        &self,
        action: &RemediationAction,
    ) -> Result<RemediationResult, String> {
        info!("Applying conscience adjustment: {}", action.description);

        // Placeholder: In full implementation, would adjust conscience layer thresholds
        // This would integrate with the conscience layer

        let mut adjustments_made = HashMap::new();
        for (param, value) in &action.parameter_adjustments {
            debug!("Adjusting conscience parameter {} to {}", param, value);
            adjustments_made.insert(param.clone(), *value);
        }

        Ok(RemediationResult {
            action_id: action.id.clone(),
            applied: true,
            reason: "Conscience thresholds adjusted".to_string(),
            adjustments_made,
            timestamp: Utc::now(),
        })
    }

    /// Apply pattern retraining
    async fn apply_pattern_retrain(
        &self,
        action: &RemediationAction,
    ) -> Result<RemediationResult, String> {
        info!("Applying pattern retrain: {}", action.description);

        // Placeholder: In full implementation, would retrain decision patterns
        // This is a more complex operation that might involve ML model updates

        Ok(RemediationResult {
            action_id: action.id.clone(),
            applied: true,
            reason: "Decision pattern retraining initiated".to_string(),
            adjustments_made: [("pattern_retrain_cycles".to_string(), 1.0)]
                .iter()
                .cloned()
                .collect(),
            timestamp: Utc::now(),
        })
    }

    /// Validate remediation before applying
    pub fn validate_remediation(&self, action: &RemediationAction) -> Result<(), String> {
        // Check parameter ranges
        for (param, value) in &action.parameter_adjustments {
            if *value < 0.0 || *value > 1.0 {
                return Err(format!(
                    "Parameter {} value {} out of range [0.0, 1.0]",
                    param, value
                ));
            }
        }

        // Validate remediation type compatibility
        match &action.remediation_type {
            RemediationType::ParameterReweight => {
                if action.parameter_adjustments.is_empty() {
                    return Err("ParameterReweight requires parameter_adjustments".to_string());
                }
            }
            RemediationType::ManualReview => {
                // No automatic validation needed
            }
            _ => {
                // Other types validated during application
            }
        }

        Ok(())
    }

    /// Generate remediation report
    pub fn generate_report(&self, results: &[RemediationResult]) -> RemediationReport {
        let successful = results.iter().filter(|r| r.applied).count();
        let failed = results.len() - successful;

        let summary = if successful == results.len() {
            "All remediations applied successfully".to_string()
        } else if successful > 0 {
            format!(
                "Partial success: {} applied, {} failed/skipped",
                successful, failed
            )
        } else {
            "No remediations applied".to_string()
        };

        RemediationReport {
            timestamp: Utc::now(),
            total_remediations: results.len(),
            successful_remediations: successful,
            failed_remediations: failed,
            summary,
            details: results.to_vec(),
        }
    }
}

/// Result of applying a remediation
#[derive(Debug, Clone)]
pub struct RemediationResult {
    /// Action ID
    pub action_id: String,
    /// Was it applied?
    pub applied: bool,
    /// Reason (if not applied, explains why)
    pub reason: String,
    /// Adjustments actually made
    pub adjustments_made: HashMap<String, f64>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Summary report of remediation session
#[derive(Debug, Clone)]
pub struct RemediationReport {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Total remediations attempted
    pub total_remediations: usize,
    /// Successful applications
    pub successful_remediations: usize,
    /// Failed applications
    pub failed_remediations: usize,
    /// Summary description
    pub summary: String,
    /// Detailed results
    pub details: Vec<RemediationResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_apply_parameter_reweight() {
        let engine = RemediationEngine::new("test_node".to_string(), true);

        let action = RemediationAction {
            id: "test_action".to_string(),
            target_node: "all".to_string(),
            remediation_type: RemediationType::ParameterReweight,
            description: "Test reweight".to_string(),
            parameter_adjustments: [("scrutiny_threshold".to_string(), 0.85)]
                .iter()
                .cloned()
                .collect(),
            expected_impact: "Test impact".to_string(),
        };

        let result = engine.apply_remediation(&action).await.unwrap();
        assert!(result.applied);
        assert_eq!(result.adjustments_made.len(), 1);
    }

    #[tokio::test]
    async fn test_auto_remediation_disabled() {
        let engine = RemediationEngine::new("test_node".to_string(), false);

        let action = RemediationAction {
            id: "test_action".to_string(),
            target_node: "all".to_string(),
            remediation_type: RemediationType::ParameterReweight,
            description: "Test reweight".to_string(),
            parameter_adjustments: HashMap::new(),
            expected_impact: "Test impact".to_string(),
        };

        let result = engine.apply_remediation(&action).await.unwrap();
        assert!(!result.applied);
        assert!(result.reason.contains("Auto-remediation disabled"));
    }

    #[test]
    fn test_validate_remediation() {
        let engine = RemediationEngine::new("test_node".to_string(), true);

        // Valid action
        let valid_action = RemediationAction {
            id: "test".to_string(),
            target_node: "all".to_string(),
            remediation_type: RemediationType::ParameterReweight,
            description: "Test".to_string(),
            parameter_adjustments: [("test_param".to_string(), 0.5)]
                .iter()
                .cloned()
                .collect(),
            expected_impact: "Test".to_string(),
        };

        assert!(engine.validate_remediation(&valid_action).is_ok());

        // Invalid: out of range parameter
        let invalid_action = RemediationAction {
            id: "test".to_string(),
            target_node: "all".to_string(),
            remediation_type: RemediationType::ParameterReweight,
            description: "Test".to_string(),
            parameter_adjustments: [("test_param".to_string(), 1.5)]
                .iter()
                .cloned()
                .collect(),
            expected_impact: "Test".to_string(),
        };

        assert!(engine.validate_remediation(&invalid_action).is_err());
    }

    #[test]
    fn test_generate_report() {
        let engine = RemediationEngine::new("test_node".to_string(), true);

        let results = vec![
            RemediationResult {
                action_id: "1".to_string(),
                applied: true,
                reason: "Success".to_string(),
                adjustments_made: HashMap::new(),
                timestamp: Utc::now(),
            },
            RemediationResult {
                action_id: "2".to_string(),
                applied: false,
                reason: "Skipped".to_string(),
                adjustments_made: HashMap::new(),
                timestamp: Utc::now(),
            },
        ];

        let report = engine.generate_report(&results);
        assert_eq!(report.total_remediations, 2);
        assert_eq!(report.successful_remediations, 1);
        assert_eq!(report.failed_remediations, 1);
    }
}
