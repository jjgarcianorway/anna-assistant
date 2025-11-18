// Phase 1.8: Distributed Audit Consensus - Core Module (POC IMPLEMENTATION)
// Status: Working PoC - Quorum logic, TIS aggregation, Byzantine detection

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
#[cfg(test)]
use uuid::Uuid;

pub mod crypto;
pub mod rpc;
pub mod state;

use crate::mirror_audit::types::BiasKind;
use crypto::{sign, verify, Keypair, PublicKey, Signature};

// ============================================================================
// TYPES
// ============================================================================

pub type NodeId = String;
pub type RoundId = String;
pub type AuditId = String;
pub type Hash = String;

// ============================================================================
// AUDIT OBSERVATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditObservation {
    pub node_id: NodeId,
    pub audit_id: AuditId,
    pub round_id: RoundId,
    pub window_hours: u64,
    pub timestamp: DateTime<Utc>,
    pub forecast_hash: Hash,
    pub outcome_hash: Hash,
    pub tis_components: TISComponents,
    pub tis_overall: f64,
    pub bias_flags: Vec<BiasKind>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TISComponents {
    pub prediction_accuracy: f64,
    pub ethical_alignment: f64,
    pub coherence_stability: f64,
}

impl AuditObservation {
    /// Create canonical encoding for signature
    pub fn canonical_encoding(&self) -> Vec<u8> {
        let bias_str = self
            .bias_flags
            .iter()
            .map(|b| format!("{:?}", b))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{}|{}|{}|{}|{}|{}|{}|{:.6}|{:.6}|{:.6}|{:.6}|{}",
            self.node_id,
            self.audit_id,
            self.round_id,
            self.window_hours,
            self.timestamp.timestamp(),
            self.forecast_hash,
            self.outcome_hash,
            self.tis_overall,
            self.tis_components.prediction_accuracy,
            self.tis_components.ethical_alignment,
            self.tis_components.coherence_stability,
            bias_str
        )
        .into_bytes()
    }

    /// Sign observation with keypair
    pub fn sign_with(&mut self, keypair: &Keypair) -> Result<()> {
        let message = self.canonical_encoding();
        let signature = sign(&message, &keypair.secret)?;
        self.signature = signature.as_bytes().to_vec();
        Ok(())
    }

    /// Verify observation signature
    pub fn verify_signature(&self, public_key: &PublicKey) -> Result<bool> {
        let message = self.canonical_encoding();
        let signature = Signature::from_bytes(&self.signature)?;
        verify(&message, &signature, public_key)
    }
}

// ============================================================================
// CONSENSUS ROUND
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub round_id: RoundId,
    pub window_hours: u64,
    pub started_at: DateTime<Utc>,
    pub observations: Vec<AuditObservation>,
    pub status: RoundStatus,
    pub consensus_tis: Option<f64>,
    pub consensus_biases: Vec<BiasKind>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoundStatus {
    Pending,
    Complete,
    Failed,
}

// ============================================================================
// CONSENSUS RESULT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub round_id: RoundId,
    pub status: RoundStatus,
    pub participating_nodes: Vec<NodeId>,
    pub total_observations: usize,
    pub required_quorum: usize,
    pub consensus_tis: Option<f64>,
    pub consensus_biases: Vec<BiasKind>,
    pub signatures: Vec<ValidatorSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub node_id: NodeId,
    pub signature: Vec<u8>,
}

// ============================================================================
// BYZANTINE DETECTION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByzantineNode {
    pub node_id: NodeId,
    pub detected_at: DateTime<Utc>,
    pub reason: ByzantineReason,
    pub excluded_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByzantineReason {
    ConflictingObservations,
    ExcessiveDeviation,
    InvalidSignature,
}

// ============================================================================
// CONSENSUS ENGINE
// ============================================================================

pub struct ConsensusEngine {
    node_id: Option<NodeId>,
    validator_count: usize,
    rounds: Vec<ConsensusRound>,
    byzantine_nodes: Vec<ByzantineNode>,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        info!("Consensus engine initialized");
        Self {
            node_id: None,
            validator_count: 1,
            rounds: Vec::new(),
            byzantine_nodes: Vec::new(),
        }
    }

    pub fn set_node_id(&mut self, node_id: NodeId) {
        info!("Set node ID: {}", node_id);
        self.node_id = Some(node_id);
    }

    pub fn set_validator_count(&mut self, count: usize) {
        info!("Set validator count: {}", count);
        self.validator_count = count;
    }

    /// Calculate quorum threshold (majority)
    pub fn quorum_threshold(&self) -> usize {
        (self.validator_count + 1) / 2
    }

    /// Submit observation and process consensus
    pub fn submit_observation(&mut self, obs: AuditObservation) -> Result<bool> {
        debug!("Submitting observation: {}", obs.audit_id);

        let round_id = obs.round_id.clone();
        let window_hours = obs.window_hours;

        // Check for double-submit (Byzantine) before modifying state
        {
            let round = self.get_or_create_round(&round_id, window_hours);
            if round
                .observations
                .iter()
                .any(|o| o.node_id == obs.node_id && o.audit_id != obs.audit_id)
            {
                warn!(
                    "Byzantine behavior detected: double-submit from {}",
                    obs.node_id
                );
                self.mark_byzantine(
                    obs.node_id.clone(),
                    ByzantineReason::ConflictingObservations,
                );
                return Ok(false);
            }
        }

        // Add observation (get mutable reference in new scope)
        {
            let round = self.get_or_create_round(&round_id, window_hours);
            round.observations.push(obs);
        }

        // Check if quorum reached and compute consensus
        let should_compute = {
            let round = self.rounds.iter().find(|r| r.round_id == round_id).unwrap();
            self.has_quorum(round)
        };

        if should_compute {
            info!("Quorum reached for round {}", round_id);
            self.compute_consensus_for_round(&round_id)?;
        }

        Ok(true)
    }

    /// Get or create round
    fn get_or_create_round(&mut self, round_id: &str, window_hours: u64) -> &mut ConsensusRound {
        if let Some(idx) = self.rounds.iter().position(|r| r.round_id == round_id) {
            &mut self.rounds[idx]
        } else {
            let round = ConsensusRound {
                round_id: round_id.to_string(),
                window_hours,
                started_at: Utc::now(),
                observations: Vec::new(),
                status: RoundStatus::Pending,
                consensus_tis: None,
                consensus_biases: Vec::new(),
            };
            self.rounds.push(round);
            self.rounds.last_mut().unwrap()
        }
    }

    /// Check if round has reached quorum
    pub fn has_quorum(&self, round: &ConsensusRound) -> bool {
        let valid_obs = round
            .observations
            .iter()
            .filter(|o| !self.is_byzantine(&o.node_id))
            .count();
        valid_obs >= self.quorum_threshold()
    }

    /// Compute consensus for round by ID
    fn compute_consensus_for_round(&mut self, round_id: &str) -> Result<()> {
        let round_idx = self
            .rounds
            .iter()
            .position(|r| r.round_id == round_id)
            .ok_or_else(|| anyhow!("Round not found: {}", round_id))?;

        // Collect Byzantine nodes for filtering
        let byzantine_nodes: Vec<NodeId> = self
            .byzantine_nodes
            .iter()
            .map(|b| b.node_id.clone())
            .collect();

        let round = &mut self.rounds[round_idx];
        Self::compute_consensus_impl(round, &byzantine_nodes)
    }

    /// Compute consensus for round (static helper to avoid borrow conflicts)
    fn compute_consensus_impl(
        round: &mut ConsensusRound,
        byzantine_nodes: &[NodeId],
    ) -> Result<()> {
        debug!("Computing consensus for round {}", round.round_id);

        // Filter out Byzantine nodes
        let valid_obs: Vec<_> = round
            .observations
            .iter()
            .filter(|o| !byzantine_nodes.contains(&o.node_id))
            .collect();

        if valid_obs.is_empty() {
            round.status = RoundStatus::Failed;
            return Ok(());
        }

        // Compute weighted average TIS
        let mut tis_sum = 0.0;
        let mut weight_sum = 0.0;

        for obs in &valid_obs {
            let weight = 1.0; // Equal weighting for PoC
            tis_sum += obs.tis_overall * weight;
            weight_sum += weight;
        }

        round.consensus_tis = Some(tis_sum / weight_sum);

        // Aggregate biases (union with max confidence)
        let mut bias_map: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for obs in &valid_obs {
            for bias in &obs.bias_flags {
                let key = format!("{:?}", bias);
                *bias_map.entry(key).or_insert(0) += 1;
            }
        }

        // Include biases reported by majority
        let majority = valid_obs.len() / 2 + 1;
        for obs in &valid_obs {
            for bias in &obs.bias_flags {
                let key = format!("{:?}", bias);
                if bias_map.get(&key).copied().unwrap_or(0) >= majority
                    && !round.consensus_biases.contains(bias)
                {
                    round.consensus_biases.push(bias.clone());
                }
            }
        }

        round.status = RoundStatus::Complete;

        info!(
            "Consensus reached for round {}: TIS={:.3}, biases={}",
            round.round_id,
            round.consensus_tis.unwrap(),
            round.consensus_biases.len()
        );

        Ok(())
    }

    /// Mark node as Byzantine
    fn mark_byzantine(&mut self, node_id: NodeId, reason: ByzantineReason) {
        if self.is_byzantine(&node_id) {
            return;
        }

        warn!("Marking node {} as Byzantine: {:?}", node_id, reason);

        self.byzantine_nodes.push(ByzantineNode {
            node_id,
            detected_at: Utc::now(),
            reason,
            excluded_until: None,
        });
    }

    /// Check if node is Byzantine
    fn is_byzantine(&self, node_id: &str) -> bool {
        self.byzantine_nodes.iter().any(|b| b.node_id == node_id)
    }

    /// Get rounds
    pub fn get_rounds(&self) -> &[ConsensusRound] {
        &self.rounds
    }

    /// Get round by ID
    pub fn get_round(&self, round_id: &str) -> Option<&ConsensusRound> {
        self.rounds.iter().find(|r| r.round_id == round_id)
    }

    /// Get Byzantine nodes
    pub fn get_byzantine_nodes(&self) -> &[ByzantineNode] {
        &self.byzantine_nodes
    }

    /// Get consensus result for round
    pub fn get_consensus_result(&self, round_id: &str) -> Option<ConsensusResult> {
        let round = self.get_round(round_id)?;

        let participating_nodes: Vec<_> = round
            .observations
            .iter()
            .map(|o| o.node_id.clone())
            .collect();

        let signatures: Vec<_> = round
            .observations
            .iter()
            .map(|o| ValidatorSignature {
                node_id: o.node_id.clone(),
                signature: o.signature.clone(),
            })
            .collect();

        Some(ConsensusResult {
            round_id: round.round_id.clone(),
            status: round.status,
            participating_nodes,
            total_observations: round.observations.len(),
            required_quorum: self.quorum_threshold(),
            consensus_tis: round.consensus_tis,
            consensus_biases: round.consensus_biases.clone(),
            signatures,
        })
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::crypto::generate_keypair;

    fn create_test_observation(
        node_id: &str,
        round_id: &str,
        tis: f64,
        keypair: &Keypair,
    ) -> AuditObservation {
        let mut obs = AuditObservation {
            node_id: node_id.to_string(),
            audit_id: Uuid::new_v4().to_string(),
            round_id: round_id.to_string(),
            window_hours: 24,
            timestamp: Utc::now(),
            forecast_hash: "hash1".to_string(),
            outcome_hash: "hash2".to_string(),
            tis_components: TISComponents {
                prediction_accuracy: 0.85,
                ethical_alignment: 0.80,
                coherence_stability: 0.90,
            },
            tis_overall: tis,
            bias_flags: Vec::new(),
            signature: Vec::new(),
        };

        obs.sign_with(keypair).unwrap();
        obs
    }

    #[test]
    fn test_quorum_threshold() {
        let mut engine = ConsensusEngine::new();

        engine.set_validator_count(1);
        assert_eq!(engine.quorum_threshold(), 1);

        engine.set_validator_count(3);
        assert_eq!(engine.quorum_threshold(), 2);

        engine.set_validator_count(5);
        assert_eq!(engine.quorum_threshold(), 3);
    }

    #[test]
    fn test_consensus_with_quorum() {
        let mut engine = ConsensusEngine::new();
        engine.set_validator_count(3);

        let keypair1 = generate_keypair().unwrap();
        let keypair2 = generate_keypair().unwrap();
        let keypair3 = generate_keypair().unwrap();

        let round_id = "test_round_1";

        // Submit 3 observations
        engine
            .submit_observation(create_test_observation("node1", round_id, 0.80, &keypair1))
            .unwrap();
        engine
            .submit_observation(create_test_observation("node2", round_id, 0.85, &keypair2))
            .unwrap();
        engine
            .submit_observation(create_test_observation("node3", round_id, 0.90, &keypair3))
            .unwrap();

        // Check consensus
        let round = engine.get_round(round_id).unwrap();
        assert_eq!(round.status, RoundStatus::Complete);
        assert!(round.consensus_tis.is_some());

        let consensus_tis = round.consensus_tis.unwrap();
        assert!((consensus_tis - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_byzantine_double_submit() {
        let mut engine = ConsensusEngine::new();
        engine.set_validator_count(3);

        let keypair1 = generate_keypair().unwrap();
        let round_id = "test_round_2";

        // First submission - ok
        engine
            .submit_observation(create_test_observation("node1", round_id, 0.80, &keypair1))
            .unwrap();

        // Second submission from same node - Byzantine
        let result =
            engine.submit_observation(create_test_observation("node1", round_id, 0.90, &keypair1));

        assert_eq!(result.unwrap(), false);
        assert_eq!(engine.get_byzantine_nodes().len(), 1);
        assert_eq!(engine.get_byzantine_nodes()[0].node_id, "node1");
    }

    #[test]
    fn test_observation_signing_and_verification() {
        let keypair = generate_keypair().unwrap();
        let mut obs = create_test_observation("node1", "round1", 0.85, &keypair);

        // Verify signature
        let verified = obs.verify_signature(&keypair.public).unwrap();
        assert!(verified);

        // Tamper with observation
        obs.tis_overall = 0.95;

        // Verification should fail
        let verified = obs.verify_signature(&keypair.public).unwrap();
        assert!(!verified);
    }
}
