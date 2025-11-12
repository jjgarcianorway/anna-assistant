//! Consensus engine for distributed decision making
//!
//! Phase 1.3: Ethics-aware weighted voting
//! Citation: [archwiki:System_maintenance]

use super::types::{
    ConsensusDecision, ConsensusRecord, ConsensusVote, PeerId, TrustScore, VoteType,
};
use chrono::Utc;
use std::collections::HashMap;
use tokio::time::{timeout, Duration};
use tracing::{debug, info};
use uuid::Uuid;

/// Consensus engine for distributed voting
pub struct ConsensusEngine {
    /// Pending consensus requests
    pending: HashMap<String, ConsensusRequest>,
    /// Completed consensus records
    completed: Vec<ConsensusRecord>,
}

/// Internal consensus request tracking
struct ConsensusRequest {
    id: String,
    action: String,
    requester: PeerId,
    timeout_secs: u64,
    started_at: chrono::DateTime<chrono::Utc>,
    votes: HashMap<PeerId, ConsensusVote>,
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            completed: Vec::new(),
        }
    }

    /// Load from existing records
    pub fn from_records(records: Vec<ConsensusRecord>) -> Self {
        Self {
            pending: HashMap::new(),
            completed: records,
        }
    }

    /// Initiate new consensus request
    pub fn request_consensus(
        &mut self,
        action: String,
        requester: PeerId,
        timeout_secs: u64,
    ) -> String {
        let id = Uuid::new_v4().to_string();

        let request = ConsensusRequest {
            id: id.clone(),
            action,
            requester,
            timeout_secs,
            started_at: Utc::now(),
            votes: HashMap::new(),
        };

        self.pending.insert(id.clone(), request);

        info!("Consensus request initiated: {}", id);
        id
    }

    /// Submit vote for a consensus request
    pub fn submit_vote(&mut self, consensus_id: &str, vote: ConsensusVote) -> bool {
        if let Some(request) = self.pending.get_mut(consensus_id) {
            request.votes.insert(vote.peer_id.clone(), vote);
            debug!(
                "Vote recorded for consensus {}: {} votes",
                consensus_id,
                request.votes.len()
            );
            true
        } else {
            false
        }
    }

    /// Check if consensus is reached
    pub fn check_consensus(
        &mut self,
        consensus_id: &str,
        trust_scores: &HashMap<PeerId, TrustScore>,
        min_voters: usize,
    ) -> Option<ConsensusDecision> {
        let request = self.pending.get(consensus_id)?;

        // Check timeout
        let elapsed = Utc::now()
            .signed_duration_since(request.started_at)
            .num_seconds() as u64;

        if elapsed > request.timeout_secs {
            return Some(ConsensusDecision::Timeout);
        }

        // Need minimum number of votes
        if request.votes.len() < min_voters {
            return None;
        }

        // Calculate weighted votes
        let mut weighted_approve = 0.0;
        let mut weighted_reject = 0.0;
        let mut total_weight = 0.0;

        for vote in request.votes.values() {
            // Get trust weight for this peer
            let weight = trust_scores
                .get(&vote.peer_id)
                .map(|s| s.overall)
                .unwrap_or(0.5)
                * vote.weight;

            total_weight += weight;

            match vote.vote {
                VoteType::Approve => weighted_approve += weight,
                VoteType::Reject => weighted_reject += weight,
                VoteType::Abstain => {}
            }
        }

        if total_weight == 0.0 {
            return None;
        }

        // Decision threshold: >60% weighted approval
        let approval_ratio = weighted_approve / total_weight;

        if approval_ratio > 0.6 {
            Some(ConsensusDecision::Approved)
        } else if weighted_reject / total_weight > 0.4 {
            Some(ConsensusDecision::Rejected)
        } else {
            None // Not decided yet
        }
    }

    /// Finalize consensus and move to completed
    pub fn finalize_consensus(
        &mut self,
        consensus_id: &str,
        decision: ConsensusDecision,
    ) -> Option<ConsensusRecord> {
        let request = self.pending.remove(consensus_id)?;

        let reasoning = self.generate_reasoning(&request, decision);

        let record = ConsensusRecord {
            id: request.id,
            timestamp: Utc::now(),
            action: request.action,
            votes: request.votes,
            decision,
            reasoning,
        };

        self.completed.push(record.clone());

        // Keep only last 1000 records
        if self.completed.len() > 1000 {
            self.completed = self.completed.split_off(self.completed.len() - 1000);
        }

        info!("Consensus finalized: {} -> {:?}", consensus_id, decision);

        Some(record)
    }

    /// Generate reasoning explanation
    fn generate_reasoning(&self, request: &ConsensusRequest, decision: ConsensusDecision) -> String {
        let approve_count = request
            .votes
            .values()
            .filter(|v| v.vote == VoteType::Approve)
            .count();

        let reject_count = request
            .votes
            .values()
            .filter(|v| v.vote == VoteType::Reject)
            .count();

        let abstain_count = request
            .votes
            .values()
            .filter(|v| v.vote == VoteType::Abstain)
            .count();

        format!(
            "Consensus decision: {:?}\nVotes: {} approve, {} reject, {} abstain\nTotal participants: {}\nAction: {}",
            decision,
            approve_count,
            reject_count,
            abstain_count,
            request.votes.len(),
            request.action
        )
    }

    /// Get pending consensus requests
    pub fn get_pending(&self) -> Vec<String> {
        self.pending.keys().cloned().collect()
    }

    /// Get completed records
    pub fn get_completed(&self) -> &[ConsensusRecord] {
        &self.completed
    }

    /// Get specific consensus request
    pub fn get_request(&self, consensus_id: &str) -> Option<&ConsensusRequest> {
        self.pending.get(consensus_id)
    }

    /// Clean up timed-out requests
    pub fn cleanup_timeouts(&mut self) {
        let now = Utc::now();
        let mut to_remove = Vec::new();

        for (id, request) in &self.pending {
            let elapsed = now.signed_duration_since(request.started_at).num_seconds() as u64;

            if elapsed > request.timeout_secs {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            if let Some(request) = self.pending.remove(&id) {
                let record = ConsensusRecord {
                    id: request.id,
                    timestamp: Utc::now(),
                    action: request.action,
                    votes: request.votes,
                    decision: ConsensusDecision::Timeout,
                    reasoning: "Consensus timed out".to_string(),
                };

                self.completed.push(record);
            }
        }
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_creation() {
        let mut engine = ConsensusEngine::new();

        let id = engine.request_consensus(
            "test_action".to_string(),
            "requester_peer".to_string(),
            60,
        );

        assert!(engine.get_pending().contains(&id));
    }

    #[test]
    fn test_vote_submission() {
        let mut engine = ConsensusEngine::new();

        let id = engine.request_consensus(
            "test_action".to_string(),
            "requester_peer".to_string(),
            60,
        );

        let vote = ConsensusVote {
            peer_id: "voter1".to_string(),
            vote: VoteType::Approve,
            weight: 1.0,
            ethical_score: 0.8,
            reasoning: "Approved".to_string(),
            timestamp: Utc::now(),
        };

        assert!(engine.submit_vote(&id, vote));
    }

    #[test]
    fn test_consensus_decision() {
        let mut engine = ConsensusEngine::new();
        let mut trust_scores = HashMap::new();

        // Add trust scores for voters
        trust_scores.insert(
            "voter1".to_string(),
            TrustScore {
                peer_id: "voter1".to_string(),
                overall: 0.9,
                ..Default::default()
            },
        );

        trust_scores.insert(
            "voter2".to_string(),
            TrustScore {
                peer_id: "voter2".to_string(),
                overall: 0.8,
                ..Default::default()
            },
        );

        let id = engine.request_consensus(
            "test_action".to_string(),
            "requester_peer".to_string(),
            60,
        );

        // Two approval votes
        engine.submit_vote(
            &id,
            ConsensusVote {
                peer_id: "voter1".to_string(),
                vote: VoteType::Approve,
                weight: 1.0,
                ethical_score: 0.9,
                reasoning: "Good action".to_string(),
                timestamp: Utc::now(),
            },
        );

        engine.submit_vote(
            &id,
            ConsensusVote {
                peer_id: "voter2".to_string(),
                vote: VoteType::Approve,
                weight: 1.0,
                ethical_score: 0.85,
                reasoning: "Ethical".to_string(),
                timestamp: Utc::now(),
            },
        );

        let decision = engine.check_consensus(&id, &trust_scores, 2);
        assert_eq!(decision, Some(ConsensusDecision::Approved));
    }
}
