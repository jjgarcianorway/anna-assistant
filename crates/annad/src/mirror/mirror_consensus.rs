//! Mirror consensus - Collective ethical alignment sessions
//!
//! Phase 1.4: Quorum-based synthesis and meta-evaluation
//! Citation: [archwiki:System_maintenance]

use super::types::*;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Mirror consensus coordinator
pub struct MirrorConsensusCoordinator {
    /// Node ID
    node_id: PeerId,
    /// Minimum nodes for quorum
    min_nodes: usize,
    /// Coherence threshold for action
    coherence_threshold: f64,
}

impl MirrorConsensusCoordinator {
    /// Create new consensus coordinator
    pub fn new(node_id: PeerId, min_nodes: usize, coherence_threshold: f64) -> Self {
        Self {
            node_id,
            min_nodes,
            coherence_threshold,
        }
    }

    /// Initiate mirror consensus session
    pub fn initiate_session(
        &self,
        reflections: Vec<ReflectionReport>,
        critiques: Vec<PeerCritique>,
    ) -> MirrorConsensusSession {
        info!(
            "Initiating mirror consensus session with {} reflections, {} critiques",
            reflections.len(),
            critiques.len()
        );

        // Extract participants
        let mut participants: Vec<PeerId> = reflections.iter().map(|r| r.node_id.clone()).collect();
        participants.sort();
        participants.dedup();

        // Check quorum
        if participants.len() < self.min_nodes {
            warn!(
                "Insufficient participants ({}/{}) for consensus",
                participants.len(),
                self.min_nodes
            );
        }

        // Analyze ethical trends
        let ethical_trends = self.analyze_ethical_trends(&reflections);

        // Detect systemic biases
        let systemic_biases = self.detect_systemic_biases(&reflections, &critiques);

        // Calculate network coherence
        let network_coherence = self.calculate_network_coherence(&reflections, &critiques);

        // Determine outcome
        let outcome = self.determine_outcome(network_coherence, &systemic_biases);

        // Generate remediation actions
        let approved_remediations = self.generate_remediations(&systemic_biases, &outcome);

        let session = MirrorConsensusSession {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            participants,
            reflections_analyzed: reflections.iter().map(|r| r.id.clone()).collect(),
            critiques_received: critiques.iter().map(|c| c.id.clone()).collect(),
            ethical_trends,
            systemic_biases,
            network_coherence,
            outcome,
            approved_remediations,
        };

        info!(
            "Consensus session {} completed: outcome={:?}, coherence={:.2}, remediations={}",
            session.id,
            session.outcome,
            session.network_coherence,
            session.approved_remediations.len()
        );

        session
    }

    /// Analyze ethical trends across network
    fn analyze_ethical_trends(&self, reflections: &[ReflectionReport]) -> EthicalTrends {
        if reflections.is_empty() {
            return EthicalTrends {
                avg_coherence: 0.5,
                coherence_trend: "stable".to_string(),
                avg_empathy: 0.5,
                avg_strain: 0.5,
                common_patterns: vec![],
            };
        }

        // Calculate averages
        let avg_coherence =
            reflections.iter().map(|r| r.self_coherence).sum::<f64>() / reflections.len() as f64;

        let avg_empathy = reflections
            .iter()
            .map(|r| r.empathy_summary.avg_empathy_index)
            .sum::<f64>()
            / reflections.len() as f64;

        let avg_strain = reflections
            .iter()
            .map(|r| r.empathy_summary.avg_strain_index)
            .sum::<f64>()
            / reflections.len() as f64;

        // Determine trends (placeholder - would need historical data)
        let coherence_trend = if avg_coherence > 0.75 {
            "improving"
        } else if avg_coherence < 0.65 {
            "declining"
        } else {
            "stable"
        }
        .to_string();

        // Identify common patterns
        let mut common_patterns = Vec::new();

        // Check for common approval patterns
        let high_approval_nodes = reflections
            .iter()
            .filter(|r| {
                let approval_rate = r
                    .ethical_decisions
                    .iter()
                    .filter(|d| d.outcome == "Approved")
                    .count() as f64
                    / r.ethical_decisions.len().max(1) as f64;
                approval_rate > 0.9
            })
            .count();

        if high_approval_nodes as f64 / reflections.len() as f64 > 0.5 {
            common_patterns.push("Network-wide high approval tendency".to_string());
        }

        // Check for common empathy levels
        if avg_empathy > 0.8 {
            common_patterns.push("Network exhibiting high empathy".to_string());
        }

        if avg_strain > 0.7 {
            common_patterns.push("Network under elevated strain".to_string());
        }

        EthicalTrends {
            avg_coherence,
            coherence_trend,
            avg_empathy,
            avg_strain,
            common_patterns,
        }
    }

    /// Detect systemic biases affecting multiple nodes
    fn detect_systemic_biases(
        &self,
        reflections: &[ReflectionReport],
        critiques: &[PeerCritique],
    ) -> Vec<SystemicBias> {
        let mut systemic_biases = Vec::new();

        // Analyze critiques for common bias patterns
        let mut bias_counts: HashMap<String, Vec<PeerId>> = HashMap::new();

        for critique in critiques {
            for bias in &critique.identified_biases {
                if bias.confidence > 0.7 {
                    bias_counts
                        .entry(bias.bias_type.clone())
                        .or_insert_with(Vec::new)
                        .push(critique.target_node_id.clone());
                }
            }
        }

        // Identify biases affecting multiple nodes
        for (bias_type, affected_nodes) in bias_counts {
            if affected_nodes.len() >= 2 {
                let severity = affected_nodes.len() as f64 / reflections.len() as f64;

                systemic_biases.push(SystemicBias {
                    bias_type: bias_type.clone(),
                    description: format!(
                        "{} detected across {} nodes ({:.0}% of network)",
                        bias_type,
                        affected_nodes.len(),
                        severity * 100.0
                    ),
                    affected_nodes,
                    severity,
                    root_cause: self.hypothesize_root_cause(&bias_type),
                });
            }
        }

        // Check for network-wide strain patterns
        let high_strain_nodes: Vec<PeerId> = reflections
            .iter()
            .filter(|r| r.empathy_summary.avg_strain_index > 0.7)
            .map(|r| r.node_id.clone())
            .collect();

        if high_strain_nodes.len() as f64 / reflections.len() as f64 > 0.5 {
            systemic_biases.push(SystemicBias {
                bias_type: "NetworkStrain".to_string(),
                description: "Majority of network under high strain".to_string(),
                affected_nodes: high_strain_nodes,
                severity: 0.8,
                root_cause: "Possible external stressor or resource constraint".to_string(),
            });
        }

        systemic_biases
    }

    /// Hypothesize root cause for bias type
    fn hypothesize_root_cause(&self, bias_type: &str) -> String {
        match bias_type {
            "ConfirmationBias" => {
                "Possible cause: Nodes favoring decisions that align with prior patterns"
            }
            "RecencyBias" => "Possible cause: Over-weighting recent events in decision-making",
            "AvailabilityBias" => "Possible cause: Over-reacting to easily recalled incidents",
            "AuthorityBias" => "Possible cause: Over-trusting certain information sources",
            _ => "Root cause unknown - requires further investigation",
        }
        .to_string()
    }

    /// Calculate network-wide coherence
    fn calculate_network_coherence(
        &self,
        reflections: &[ReflectionReport],
        critiques: &[PeerCritique],
    ) -> f64 {
        if reflections.is_empty() {
            return 0.5;
        }

        let mut coherence_factors = Vec::new();

        // Factor 1: Average self-coherence
        let avg_self_coherence =
            reflections.iter().map(|r| r.self_coherence).sum::<f64>() / reflections.len() as f64;
        coherence_factors.push(avg_self_coherence);

        // Factor 2: Average peer-assessed coherence
        if !critiques.is_empty() {
            let avg_peer_coherence = critiques
                .iter()
                .map(|c| c.coherence_assessment)
                .sum::<f64>()
                / critiques.len() as f64;
            coherence_factors.push(avg_peer_coherence);
        }

        // Factor 3: Agreement between self and peer assessments
        let mut agreement_scores = Vec::new();
        for reflection in reflections {
            let peer_critiques: Vec<&PeerCritique> = critiques
                .iter()
                .filter(|c| c.reflection_id == reflection.id)
                .collect();

            if !peer_critiques.is_empty() {
                let avg_peer_assessment = peer_critiques
                    .iter()
                    .map(|c| c.coherence_assessment)
                    .sum::<f64>()
                    / peer_critiques.len() as f64;

                let agreement = 1.0 - (reflection.self_coherence - avg_peer_assessment).abs();
                agreement_scores.push(agreement);
            }
        }

        if !agreement_scores.is_empty() {
            let avg_agreement =
                agreement_scores.iter().sum::<f64>() / agreement_scores.len() as f64;
            coherence_factors.push(avg_agreement);
        }

        // Overall coherence
        coherence_factors.iter().sum::<f64>() / coherence_factors.len() as f64
    }

    /// Determine consensus outcome
    fn determine_outcome(
        &self,
        network_coherence: f64,
        systemic_biases: &[SystemicBias],
    ) -> ConsensusOutcome {
        // Check for critical issues
        let critical_biases = systemic_biases.iter().filter(|b| b.severity > 0.8).count();

        if critical_biases > 0 || network_coherence < 0.5 {
            return ConsensusOutcome::CriticalDivergence;
        }

        // Check for significant issues
        let significant_biases = systemic_biases.iter().filter(|b| b.severity > 0.5).count();

        if significant_biases > 0 || network_coherence < self.coherence_threshold {
            return ConsensusOutcome::SignificantRemediation;
        }

        // Check for minor issues
        if !systemic_biases.is_empty() || network_coherence < 0.85 {
            return ConsensusOutcome::MinorAdjustment;
        }

        ConsensusOutcome::Coherent
    }

    /// Generate remediation actions
    fn generate_remediations(
        &self,
        systemic_biases: &[SystemicBias],
        outcome: &ConsensusOutcome,
    ) -> Vec<RemediationAction> {
        let mut remediations = Vec::new();

        if *outcome == ConsensusOutcome::Coherent {
            return remediations; // No action needed
        }

        for bias in systemic_biases {
            let remediation = match bias.bias_type.as_str() {
                "ConfirmationBias" => RemediationAction {
                    id: Uuid::new_v4().to_string(),
                    target_node: "all".to_string(),
                    remediation_type: RemediationType::ParameterReweight,
                    description: "Reduce confirmation bias by increasing decision scrutiny"
                        .to_string(),
                    parameter_adjustments: [("scrutiny_threshold".to_string(), 0.85)]
                        .iter()
                        .cloned()
                        .collect(),
                    expected_impact: "More critical evaluation of decisions aligning with patterns"
                        .to_string(),
                },
                "RecencyBias" => RemediationAction {
                    id: Uuid::new_v4().to_string(),
                    target_node: "all".to_string(),
                    remediation_type: RemediationType::ParameterReweight,
                    description: "Balance temporal weighting in decision-making".to_string(),
                    parameter_adjustments: [("temporal_decay".to_string(), 0.95)]
                        .iter()
                        .cloned()
                        .collect(),
                    expected_impact: "Equal consideration of historical and recent events"
                        .to_string(),
                },
                "NetworkStrain" => RemediationAction {
                    id: Uuid::new_v4().to_string(),
                    target_node: "all".to_string(),
                    remediation_type: RemediationType::ConscienceAdjustment,
                    description: "Increase deferral thresholds during high strain".to_string(),
                    parameter_adjustments: [("strain_deferral_threshold".to_string(), 0.6)]
                        .iter()
                        .cloned()
                        .collect(),
                    expected_impact: "More cautious decisions during network stress".to_string(),
                },
                _ => RemediationAction {
                    id: Uuid::new_v4().to_string(),
                    target_node: "all".to_string(),
                    remediation_type: RemediationType::ManualReview,
                    description: format!("Manual review required for {}", bias.bias_type),
                    parameter_adjustments: HashMap::new(),
                    expected_impact: "Human oversight for unknown bias pattern".to_string(),
                },
            };

            remediations.push(remediation);
        }

        remediations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_reflection(node_id: &str, coherence: f64) -> ReflectionReport {
        ReflectionReport {
            id: format!("reflection_{}", node_id),
            node_id: node_id.to_string(),
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
            self_coherence: coherence,
            self_identified_biases: vec![],
            signature: "test_sig".to_string(),
        }
    }

    #[test]
    fn test_initiate_session() {
        let coordinator =
            MirrorConsensusCoordinator::new("coordinator".to_string(), 3, 0.7);

        let reflections = vec![
            create_test_reflection("node1", 0.8),
            create_test_reflection("node2", 0.85),
            create_test_reflection("node3", 0.75),
        ];

        let session = coordinator.initiate_session(reflections, vec![]);

        assert_eq!(session.participants.len(), 3);
        assert!(session.network_coherence > 0.7);
        assert_eq!(session.outcome, ConsensusOutcome::Coherent);
    }

    #[test]
    fn test_detect_critical_divergence() {
        let coordinator =
            MirrorConsensusCoordinator::new("coordinator".to_string(), 3, 0.7);

        let reflections = vec![
            create_test_reflection("node1", 0.3),
            create_test_reflection("node2", 0.4),
            create_test_reflection("node3", 0.35),
        ];

        let session = coordinator.initiate_session(reflections, vec![]);

        assert_eq!(session.outcome, ConsensusOutcome::CriticalDivergence);
    }
}
