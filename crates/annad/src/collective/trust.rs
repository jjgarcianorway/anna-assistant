//! Trust ledger for peer reputation management
//!
//! Phase 1.3: Track honesty, reliability, and ethical alignment
//! Citation: [archwiki:System_maintenance]

use super::types::{PeerId, TrustScore};
use chrono::Utc;
use std::collections::HashMap;
use tracing::debug;

/// Trust ledger manages reputation scores for all peers
pub struct TrustLedger {
    /// Trust scores indexed by peer ID
    scores: HashMap<PeerId, TrustScore>,
    /// Decay rate for old scores (per day)
    decay_rate: f64,
}

impl TrustLedger {
    /// Create new trust ledger
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            decay_rate: 0.01, // 1% decay per day toward neutral (0.5)
        }
    }

    /// Load ledger from saved scores
    pub fn from_scores(scores: HashMap<PeerId, TrustScore>) -> Self {
        Self {
            scores,
            decay_rate: 0.01,
        }
    }

    /// Get trust score for a peer
    pub fn get_score(&self, peer_id: &PeerId) -> TrustScore {
        self.scores
            .get(peer_id)
            .cloned()
            .unwrap_or_else(|| TrustScore {
                peer_id: peer_id.clone(),
                ..Default::default()
            })
    }

    /// Record successful message validation
    pub fn record_valid_message(&mut self, peer_id: &PeerId) {
        let mut score = self.get_score(peer_id);

        score.messages_received += 1;
        score.messages_validated += 1;

        // Increase honesty score
        score.honesty = (score.honesty + 0.05).min(1.0);

        // Recalculate overall score
        score.overall = self.calculate_overall(&score);
        score.updated_at = Utc::now();

        let honesty = score.honesty;
        self.scores.insert(peer_id.clone(), score);

        debug!("Trust increased for {}: honesty={:.2}", peer_id, honesty);
    }

    /// Record invalid message (signature failed, replay attack, etc.)
    pub fn record_invalid_message(&mut self, peer_id: &PeerId) {
        let mut score = self.get_score(peer_id);

        score.messages_received += 1;

        // Significantly decrease honesty score
        score.honesty = (score.honesty - 0.20).max(0.0);

        // Recalculate overall score
        score.overall = self.calculate_overall(&score);
        score.updated_at = Utc::now();

        let honesty = score.honesty;
        self.scores.insert(peer_id.clone(), score);

        debug!("Trust decreased for {}: honesty={:.2}", peer_id, honesty);
    }

    /// Record peer heartbeat (affects reliability)
    pub fn record_heartbeat(&mut self, peer_id: &PeerId) {
        let mut score = self.get_score(peer_id);

        // Increase reliability score slightly
        score.reliability = (score.reliability + 0.01).min(1.0);

        // Recalculate overall score
        score.overall = self.calculate_overall(&score);
        score.updated_at = Utc::now();

        self.scores.insert(peer_id.clone(), score);
    }

    /// Record missed heartbeat (affects reliability)
    pub fn record_missed_heartbeat(&mut self, peer_id: &PeerId) {
        let mut score = self.get_score(peer_id);

        // Decrease reliability score
        score.reliability = (score.reliability - 0.05).max(0.0);

        // Recalculate overall score
        score.overall = self.calculate_overall(&score);
        score.updated_at = Utc::now();

        self.scores.insert(peer_id.clone(), score);
    }

    /// Record ethical alignment from consensus participation
    pub fn record_ethical_consensus(&mut self, peer_id: &PeerId, alignment_score: f64) {
        let mut score = self.get_score(peer_id);

        // Weighted average with existing score
        score.ethical_alignment = score.ethical_alignment * 0.9 + alignment_score * 0.1;

        // Recalculate overall score
        score.overall = self.calculate_overall(&score);
        score.updated_at = Utc::now();

        let ethical = score.ethical_alignment;
        self.scores.insert(peer_id.clone(), score);

        debug!("Ethical alignment updated for {}: {:.2}", peer_id, ethical);
    }

    /// Calculate overall trust score from components
    fn calculate_overall(&self, score: &TrustScore) -> f64 {
        // Weighted average: honesty most important, then ethical alignment, then reliability
        let weights = (0.5, 0.3, 0.2);
        score.honesty * weights.0
            + score.ethical_alignment * weights.1
            + score.reliability * weights.2
    }

    /// Apply time-based decay to all scores
    pub fn apply_decay(&mut self) {
        let now = Utc::now();
        let decay_rate = self.decay_rate;

        for score in self.scores.values_mut() {
            // Calculate days since last update
            let duration = now.signed_duration_since(score.updated_at);
            let days = duration.num_days() as f64;

            if days > 0.0 {
                // Decay toward neutral (0.5)
                let decay_factor = 1.0 - (decay_rate * days).min(0.5);

                score.honesty = 0.5 + (score.honesty - 0.5) * decay_factor;
                score.reliability = 0.5 + (score.reliability - 0.5) * decay_factor;
                score.ethical_alignment = 0.5 + (score.ethical_alignment - 0.5) * decay_factor;

                // Calculate overall inline to avoid borrow issues
                let weights = (0.5, 0.3, 0.2);
                score.overall = score.honesty * weights.0
                    + score.ethical_alignment * weights.1
                    + score.reliability * weights.2;
            }
        }
    }

    /// Get all peer scores
    pub fn get_all_scores(&self) -> &HashMap<PeerId, TrustScore> {
        &self.scores
    }

    /// Get high-trust peers (above threshold)
    pub fn get_trusted_peers(&self, min_trust: f64) -> Vec<PeerId> {
        self.scores
            .iter()
            .filter(|(_, score)| score.overall >= min_trust)
            .map(|(peer_id, _)| peer_id.clone())
            .collect()
    }

    /// Remove peer from ledger
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.scores.remove(peer_id);
    }
}

impl Default for TrustLedger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_score_increase() {
        let mut ledger = TrustLedger::new();
        let peer_id = "test_peer".to_string();

        let initial = ledger.get_score(&peer_id).honesty;

        ledger.record_valid_message(&peer_id);
        let after = ledger.get_score(&peer_id).honesty;

        assert!(after > initial);
    }

    #[test]
    fn test_trust_score_decrease() {
        let mut ledger = TrustLedger::new();
        let peer_id = "test_peer".to_string();

        // First establish some trust
        for _ in 0..5 {
            ledger.record_valid_message(&peer_id);
        }

        let before = ledger.get_score(&peer_id).honesty;

        ledger.record_invalid_message(&peer_id);
        let after = ledger.get_score(&peer_id).honesty;

        assert!(after < before);
    }

    #[test]
    fn test_trusted_peers_filter() {
        let mut ledger = TrustLedger::new();

        let peer1 = "peer1".to_string();
        let peer2 = "peer2".to_string();

        // Peer 1: high trust
        for _ in 0..10 {
            ledger.record_valid_message(&peer1);
        }

        // Peer 2: low trust
        ledger.record_invalid_message(&peer2);

        let trusted = ledger.get_trusted_peers(0.6);
        assert!(trusted.contains(&peer1));
        assert!(!trusted.contains(&peer2));
    }
}
