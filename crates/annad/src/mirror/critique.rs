//! Peer critique - Evaluate other nodes' reflections
//!
//! Phase 1.4: Cross-node ethical evaluation
//! Citation: [archwiki:System_maintenance]

use super::types::*;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

/// Critique evaluator
pub struct CritiqueEvaluator {
    /// This node's ID
    node_id: PeerId,
    /// Private key for signing
    private_key: String,
    /// This node's trust score
    node_trust: f64,
}

impl CritiqueEvaluator {
    /// Create new critique evaluator
    pub fn new(node_id: PeerId, private_key: String, node_trust: f64) -> Self {
        Self {
            node_id,
            private_key,
            node_trust,
        }
    }

    /// Evaluate a peer's reflection report
    pub fn evaluate_reflection(&self, reflection: &ReflectionReport) -> PeerCritique {
        info!(
            "Evaluating reflection {} from node {}",
            reflection.id, reflection.node_id
        );

        // Assess coherence
        let coherence_assessment = self.assess_coherence(reflection);

        // Identify inconsistencies
        let inconsistencies = self.identify_inconsistencies(reflection);

        // Detect biases
        let identified_biases = self.detect_biases(reflection);

        // Generate reasoning
        let reasoning = self.generate_critique_reasoning(
            coherence_assessment,
            &inconsistencies,
            &identified_biases,
        );

        // Generate recommendations
        let recommendations = self.generate_recommendations(&inconsistencies, &identified_biases);

        let critique = PeerCritique {
            id: Uuid::new_v4().to_string(),
            critic_id: self.node_id.clone(),
            reflection_id: reflection.id.clone(),
            target_node_id: reflection.node_id.clone(),
            timestamp: Utc::now(),
            coherence_assessment,
            inconsistencies,
            identified_biases,
            reasoning,
            recommendations,
            critic_trust: self.node_trust,
            signature: self.sign_critique(),
        };

        debug!(
            "Critique generated: coherence={:.2}, inconsistencies={}, biases={}",
            critique.coherence_assessment,
            critique.inconsistencies.len(),
            critique.identified_biases.len()
        );

        critique
    }

    /// Assess overall coherence of reflection
    fn assess_coherence(&self, reflection: &ReflectionReport) -> f64 {
        let mut coherence_factors = Vec::new();

        // Factor 1: Self-coherence vs actual decision consistency
        let decision_consistency = self.calculate_decision_consistency(&reflection.ethical_decisions);
        let self_awareness_accuracy = 1.0 - (reflection.self_coherence - decision_consistency).abs();
        coherence_factors.push(self_awareness_accuracy);

        // Factor 2: Empathy-ethics alignment
        let empathy_alignment = self.assess_empathy_ethics_alignment(
            &reflection.empathy_summary,
            &reflection.ethical_decisions,
        );
        coherence_factors.push(empathy_alignment);

        // Factor 3: Trust delta rationality
        let trust_rationality = self.assess_trust_delta_rationality(&reflection.trust_deltas);
        coherence_factors.push(trust_rationality);

        // Factor 4: Bias self-awareness
        let bias_awareness = if reflection.self_identified_biases.is_empty() {
            0.5 // Neutral - might be truly unbiased or lacking awareness
        } else {
            0.8 // Good - demonstrates introspection
        };
        coherence_factors.push(bias_awareness);

        // Average coherence
        coherence_factors.iter().sum::<f64>() / coherence_factors.len() as f64
    }

    /// Calculate decision consistency
    fn calculate_decision_consistency(&self, decisions: &[EthicalDecisionRecord]) -> f64 {
        if decisions.len() < 2 {
            return 0.8; // Assume consistent if too few samples
        }

        let scores: Vec<f64> = decisions.iter().map(|d| d.ethical_score).collect();
        let avg = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores.iter().map(|s| (s - avg).powi(2)).sum::<f64>() / scores.len() as f64;

        1.0 - variance.min(1.0)
    }

    /// Assess alignment between empathy state and ethical decisions
    fn assess_empathy_ethics_alignment(
        &self,
        empathy: &EmpathySummary,
        decisions: &[EthicalDecisionRecord],
    ) -> f64 {
        if decisions.is_empty() {
            return 0.5;
        }

        // High empathy should correlate with cautious decisions
        let avg_ethical_score = decisions.iter().map(|d| d.ethical_score).sum::<f64>()
            / decisions.len() as f64;

        // Expect high empathy (>0.7) -> high ethical scores (>0.75)
        if empathy.avg_empathy_index > 0.7 {
            if avg_ethical_score > 0.75 {
                0.9 // Well aligned
            } else {
                0.6 // Misalignment: high empathy but risky decisions
            }
        } else {
            0.7 // Neutral empathy
        }
    }

    /// Assess rationality of trust score changes
    fn assess_trust_delta_rationality(&self, deltas: &HashMap<PeerId, f64>) -> f64 {
        if deltas.is_empty() {
            return 0.8; // No changes is acceptable
        }

        // Check for extreme changes (potential bias)
        let extreme_changes = deltas.values().filter(|&&delta| delta.abs() > 0.5).count();

        if extreme_changes as f64 / deltas.len() as f64 > 0.3 {
            0.5 // Many extreme changes suggest volatility
        } else {
            0.85 // Reasonable trust adjustments
        }
    }

    /// Identify inconsistencies in reflection
    fn identify_inconsistencies(&self, reflection: &ReflectionReport) -> Vec<Inconsistency> {
        let mut inconsistencies = Vec::new();

        // Check for empathy-strain contradiction
        if reflection.empathy_summary.avg_empathy_index > 0.8
            && reflection.empathy_summary.avg_strain_index > 0.8
        {
            inconsistencies.push(Inconsistency {
                inconsistency_type: "EmpathyStrainContradiction".to_string(),
                description: "High empathy and high strain simultaneously - unusual pattern"
                    .to_string(),
                severity: 0.6,
                evidence: vec![
                    format!("Empathy: {:.2}", reflection.empathy_summary.avg_empathy_index),
                    format!("Strain: {:.2}", reflection.empathy_summary.avg_strain_index),
                ],
            });
        }

        // Check for self-coherence vs bias mismatch
        if reflection.self_coherence > 0.9 && !reflection.self_identified_biases.is_empty() {
            inconsistencies.push(Inconsistency {
                inconsistency_type: "CoherenceBiasMismatch".to_string(),
                description: "Claims high coherence but identifies biases - contradiction".to_string(),
                severity: 0.5,
                evidence: vec![
                    format!("Self-coherence: {:.2}", reflection.self_coherence),
                    format!("Biases identified: {}", reflection.self_identified_biases.len()),
                ],
            });
        }

        inconsistencies
    }

    /// Detect biases in peer's behavior
    fn detect_biases(&self, reflection: &ReflectionReport) -> Vec<BiasDetection> {
        let mut biases = Vec::new();

        // Detect confirmation bias (overly consistent decisions)
        if let Some(bias) = self.detect_confirmation_bias(&reflection.ethical_decisions) {
            biases.push(bias);
        }

        // Detect recency bias (recent decisions dominating)
        if let Some(bias) = self.detect_recency_bias(&reflection.ethical_decisions) {
            biases.push(bias);
        }

        // Detect availability bias (overweighting recent empathy adaptations)
        if reflection.empathy_summary.adaptations_count > 10 {
            biases.push(BiasDetection {
                bias_type: "AvailabilityBias".to_string(),
                description: "Excessive empathy adaptations suggest over-reactivity".to_string(),
                confidence: 0.7,
                affected_decisions: vec!["empathy_adaptations".to_string()],
                correction: "Reduce adaptation frequency, increase adaptation threshold".to_string(),
            });
        }

        biases
    }

    /// Detect confirmation bias
    fn detect_confirmation_bias(
        &self,
        decisions: &[EthicalDecisionRecord],
    ) -> Option<BiasDetection> {
        if decisions.len() < 5 {
            return None;
        }

        let approval_rate = decisions
            .iter()
            .filter(|d| d.outcome == "Approved")
            .count() as f64
            / decisions.len() as f64;

        if approval_rate > 0.95 || approval_rate < 0.05 {
            Some(BiasDetection {
                bias_type: "ConfirmationBias".to_string(),
                description: format!(
                    "Extreme approval rate ({:.1}%) suggests confirmation bias",
                    approval_rate * 100.0
                ),
                confidence: 0.85,
                affected_decisions: decisions.iter().map(|d| d.id.clone()).collect(),
                correction: "Increase scrutiny of decisions that align with prior patterns"
                    .to_string(),
            })
        } else {
            None
        }
    }

    /// Detect recency bias
    fn detect_recency_bias(&self, decisions: &[EthicalDecisionRecord]) -> Option<BiasDetection> {
        if decisions.len() < 10 {
            return None;
        }

        // Check if recent decisions (last 20%) have significantly different scores
        let cutoff_index = (decisions.len() as f64 * 0.8) as usize;
        let recent: Vec<&EthicalDecisionRecord> = decisions.iter().skip(cutoff_index).collect();
        let older: Vec<&EthicalDecisionRecord> = decisions.iter().take(cutoff_index).collect();

        if recent.is_empty() || older.is_empty() {
            return None;
        }

        let recent_avg = recent.iter().map(|d| d.ethical_score).sum::<f64>() / recent.len() as f64;
        let older_avg = older.iter().map(|d| d.ethical_score).sum::<f64>() / older.len() as f64;

        let diff = (recent_avg - older_avg).abs();

        if diff > 0.2 {
            Some(BiasDetection {
                bias_type: "RecencyBias".to_string(),
                description: format!(
                    "Recent decisions differ significantly from older ones (Δ={:.2})",
                    diff
                ),
                confidence: 0.75,
                affected_decisions: recent.iter().map(|d| d.id.clone()).collect(),
                correction: "Ensure temporal consistency in ethical evaluation criteria".to_string(),
            })
        } else {
            None
        }
    }

    /// Generate critique reasoning
    fn generate_critique_reasoning(
        &self,
        coherence: f64,
        inconsistencies: &[Inconsistency],
        biases: &[BiasDetection],
    ) -> String {
        let mut reasoning = Vec::new();

        if coherence > 0.8 {
            reasoning.push("High coherence observed - ethical decisions align well with empathy state and stated values.".to_string());
        } else if coherence > 0.6 {
            reasoning.push(
                "Moderate coherence - some alignment issues detected.".to_string(),
            );
        } else {
            reasoning.push(
                "Low coherence - significant ethical inconsistencies detected.".to_string(),
            );
        }

        if !inconsistencies.is_empty() {
            reasoning.push(format!(
                "{} inconsistencies identified requiring attention.",
                inconsistencies.len()
            ));
        }

        if !biases.is_empty() {
            reasoning.push(format!(
                "{} potential biases detected in decision-making patterns.",
                biases.len()
            ));
        }

        if inconsistencies.is_empty() && biases.is_empty() && coherence > 0.8 {
            reasoning.push("Overall assessment: Ethical behavior is coherent and well-calibrated.".to_string());
        }

        reasoning.join(" ")
    }

    /// Generate recommendations for improvement
    fn generate_recommendations(
        &self,
        inconsistencies: &[Inconsistency],
        biases: &[BiasDetection],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        for inconsistency in inconsistencies {
            if inconsistency.severity > 0.5 {
                recommendations.push(format!(
                    "Address {}: {}",
                    inconsistency.inconsistency_type, inconsistency.description
                ));
            }
        }

        for bias in biases {
            if bias.confidence > 0.7 {
                recommendations.push(format!("Mitigate {}: {}", bias.bias_type, bias.correction));
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Maintain current ethical calibration.".to_string());
        }

        recommendations
    }

    /// Sign critique
    fn sign_critique(&self) -> String {
        // Placeholder: In full implementation, would use actual crypto
        let key_len = self.private_key.len().min(16);
        format!("sig_{}", &self.private_key[..key_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_reflection() -> ReflectionReport {
        ReflectionReport {
            id: "test_reflection".to_string(),
            node_id: "test_node".to_string(),
            timestamp: Utc::now(),
            period_start: Utc::now() - chrono::Duration::hours(24),
            period_end: Utc::now(),
            ethical_decisions: vec![],
            empathy_summary: EmpathySummary {
                avg_empathy_index: 0.75,
                avg_strain_index: 0.25,
                empathy_trend: "stable".to_string(),
                strain_trend: "stable".to_string(),
                adaptations_count: 3,
            },
            conscience_actions: vec![],
            trust_deltas: HashMap::new(),
            self_coherence: 0.8,
            self_identified_biases: vec![],
            signature: "test_sig".to_string(),
        }
    }

    #[test]
    fn test_evaluate_reflection() {
        let evaluator =
            CritiqueEvaluator::new("critic".to_string(), "key".to_string(), 0.9);
        let reflection = create_test_reflection();

        let critique = evaluator.evaluate_reflection(&reflection);

        assert_eq!(critique.critic_id, "critic");
        assert_eq!(critique.target_node_id, "test_node");
        assert!(critique.coherence_assessment >= 0.0 && critique.coherence_assessment <= 1.0);
    }

    #[test]
    fn test_detect_confirmation_bias() {
        let evaluator =
            CritiqueEvaluator::new("critic".to_string(), "key".to_string(), 0.9);

        // Create decisions with 100% approval
        let decisions: Vec<EthicalDecisionRecord> = (0..10)
            .map(|i| EthicalDecisionRecord {
                id: i.to_string(),
                timestamp: Utc::now(),
                action: "test".to_string(),
                outcome: "Approved".to_string(),
                ethical_score: 0.8,
                reasoning: "test".to_string(),
                stakeholder_impacts: HashMap::new(),
            })
            .collect();

        let bias = evaluator.detect_confirmation_bias(&decisions);
        assert!(bias.is_some());
        assert_eq!(bias.unwrap().bias_type, "ConfirmationBias");
    }
}
