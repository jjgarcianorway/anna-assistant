//! Collective Mind type definitions
//!
//! Phase 1.3: Multi-node cooperation and distributed consensus
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Peer ID - SHA256 hash of public key
pub type PeerId = String;

/// Collective Mind state persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveState {
    /// State version for migration tracking
    pub version: u64,
    /// Last updated timestamp
    pub timestamp: DateTime<Utc>,
    /// This node's peer ID
    pub node_id: PeerId,
    /// Known peers in the network
    pub peers: HashMap<PeerId, PeerInfo>,
    /// Trust ledger for all peers
    pub trust_ledger: HashMap<PeerId, TrustScore>,
    /// Recent consensus decisions
    pub consensus_history: Vec<ConsensusRecord>,
    /// Synchronized empathy/strain indices from network
    pub network_empathy: HashMap<PeerId, NetworkEmpathyState>,
}

impl Default for CollectiveState {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: Utc::now(),
            node_id: String::new(),
            peers: HashMap::new(),
            trust_ledger: HashMap::new(),
            consensus_history: Vec::new(),
            network_empathy: HashMap::new(),
        }
    }
}

/// Information about a peer node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub id: PeerId,
    /// Display name
    pub name: String,
    /// Network address
    pub address: SocketAddr,
    /// Public key (hex encoded)
    pub public_key: String,
    /// When peer was first discovered
    pub discovered_at: DateTime<Utc>,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Protocol version
    pub protocol_version: String,
    /// Is this peer currently connected?
    pub connected: bool,
}

/// Trust score for a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    /// Peer ID
    pub peer_id: PeerId,
    /// Overall trust score (0.0-1.0)
    pub overall: f64,
    /// Honesty score - message integrity
    pub honesty: f64,
    /// Reliability score - uptime and responsiveness
    pub reliability: f64,
    /// Ethical alignment score
    pub ethical_alignment: f64,
    /// Total messages received
    pub messages_received: u64,
    /// Messages validated successfully
    pub messages_validated: u64,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl Default for TrustScore {
    fn default() -> Self {
        Self {
            peer_id: String::new(),
            overall: 0.5, // Start with neutral trust
            honesty: 0.5,
            reliability: 0.5,
            ethical_alignment: 0.5,
            messages_received: 0,
            messages_validated: 0,
            updated_at: Utc::now(),
        }
    }
}

/// Consensus record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRecord {
    /// Consensus ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Action being considered
    pub action: String,
    /// Votes from peers
    pub votes: HashMap<PeerId, ConsensusVote>,
    /// Final decision
    pub decision: ConsensusDecision,
    /// Reasoning
    pub reasoning: String,
}

/// Consensus vote from a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    /// Peer ID
    pub peer_id: PeerId,
    /// Vote (approve/reject/abstain)
    pub vote: VoteType,
    /// Weight (based on trust score)
    pub weight: f64,
    /// Ethical score for this action
    pub ethical_score: f64,
    /// Reasoning
    pub reasoning: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Vote type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    Approve,
    Reject,
    Abstain,
}

/// Consensus decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusDecision {
    Approved,
    Rejected,
    Pending,
    Timeout,
}

/// Network empathy state from a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEmpathyState {
    /// Peer ID
    pub peer_id: PeerId,
    /// Empathy index
    pub empathy_index: f64,
    /// Strain index
    pub strain_index: f64,
    /// User resonance
    pub user_resonance: f64,
    /// System resonance
    pub system_resonance: f64,
    /// Environment resonance
    pub environment_resonance: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Gossip message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GossipMessage {
    /// Peer announcement
    PeerAnnounce {
        peer_info: PeerInfo,
        signature: String,
    },
    /// Heartbeat
    Heartbeat {
        peer_id: PeerId,
        timestamp: DateTime<Utc>,
        signature: String,
    },
    /// Empathy sync
    EmpathySync {
        peer_id: PeerId,
        state: NetworkEmpathyState,
        signature: String,
    },
    /// Consensus request
    ConsensusRequest {
        id: String,
        action: String,
        requester: PeerId,
        timeout_secs: u64,
        signature: String,
    },
    /// Consensus response
    ConsensusResponse {
        id: String,
        vote: ConsensusVote,
        signature: String,
    },
    /// Introspection request
    IntrospectionRequest {
        id: String,
        requester: PeerId,
        query: String,
        signature: String,
    },
    /// Introspection response
    IntrospectionResponse {
        id: String,
        data: String,
        signature: String,
    },
}

/// Collective configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveConfig {
    /// Enable collective mind
    pub enabled: bool,
    /// Listen address
    pub listen_addr: String,
    /// Listen port
    pub listen_port: u16,
    /// Gossip interval (seconds)
    pub gossip_interval_secs: u64,
    /// Heartbeat interval (seconds)
    pub heartbeat_interval_secs: u64,
    /// Empathy sync interval (seconds)
    pub empathy_sync_interval_secs: u64,
    /// Consensus timeout (seconds)
    pub consensus_timeout_secs: u64,
    /// Minimum trust threshold for consensus participation
    pub min_trust_threshold: f64,
    /// Maximum peers to maintain
    pub max_peers: usize,
}

impl Default for CollectiveConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            listen_addr: "0.0.0.0".to_string(),
            listen_port: 8742, // ANNA on phone keypad = 2662, but use 8742 for "TNRA"
            gossip_interval_secs: 60,
            heartbeat_interval_secs: 30,
            empathy_sync_interval_secs: 300, // 5 minutes
            consensus_timeout_secs: 60,
            min_trust_threshold: 0.3,
            max_peers: 50,
        }
    }
}

/// Collective status for CLI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveStatus {
    /// Is collective mind enabled?
    pub enabled: bool,
    /// Node ID
    pub node_id: PeerId,
    /// Number of connected peers
    pub connected_peers: usize,
    /// Total known peers
    pub total_peers: usize,
    /// Average network empathy
    pub avg_network_empathy: f64,
    /// Average network strain
    pub avg_network_strain: f64,
    /// Recent consensus decisions
    pub recent_decisions: usize,
    /// Network health (0.0-1.0)
    pub network_health: f64,
}

/// Peer trust details for CLI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerTrustDetails {
    /// Peer info
    pub peer_info: PeerInfo,
    /// Trust score
    pub trust_score: TrustScore,
    /// Recent interactions
    pub recent_messages: u64,
    /// Last interaction time
    pub last_interaction: DateTime<Utc>,
}

/// Consensus explanation for CLI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusExplanation {
    /// Consensus record
    pub record: ConsensusRecord,
    /// Approval percentage
    pub approval_percentage: f64,
    /// Weighted approval
    pub weighted_approval: f64,
    /// Dissenting peers
    pub dissenting_peers: Vec<PeerId>,
    /// Full reasoning trail
    pub reasoning_trail: String,
}

/// Introspection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionRequest {
    /// Request ID
    pub id: String,
    /// Requester peer ID
    pub requester: PeerId,
    /// Target peer ID
    pub target_peer: PeerId,
    /// Query type (conscience_status, empathy_pulse, system_health)
    pub query: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Introspection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    /// Request ID
    pub request_id: String,
    /// Responder peer ID
    pub responder: PeerId,
    /// Response data
    pub data: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
