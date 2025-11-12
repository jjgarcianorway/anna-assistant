//! Collective Mind - Distributed cooperation between Anna instances
//!
//! Phase 1.3: Multi-node cooperation, consensus, and shared learning
//! Citation: [archwiki:System_maintenance]

pub mod consensus;
pub mod crypto;
pub mod gossip;
pub mod introspect;
pub mod sync;
pub mod trust;
pub mod types;

use anyhow::{Context as AnyhowContext, Result};
use chrono::Utc;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use consensus::ConsensusEngine;
use crypto::KeyPair;
use gossip::GossipEngine;
use introspect::IntrospectionManager;
use sync::SyncManager;
use trust::TrustLedger;
use types::*;

/// Collective Mind daemon
pub struct CollectiveMind {
    /// Configuration
    config: CollectiveConfig,
    /// Current state
    state: Arc<RwLock<CollectiveState>>,
    /// Gossip engine
    gossip: Arc<RwLock<GossipEngine>>,
    /// Trust ledger
    trust: Arc<RwLock<TrustLedger>>,
    /// Consensus engine
    consensus: Arc<RwLock<ConsensusEngine>>,
    /// Sync manager
    sync: Arc<RwLock<SyncManager>>,
    /// Introspection manager
    introspect: Arc<RwLock<IntrospectionManager>>,
    /// This node's keypair
    keypair: KeyPair,
}

impl CollectiveMind {
    /// Create new collective mind instance
    pub async fn new() -> Result<Self> {
        Self::with_config(CollectiveConfig::default()).await
    }

    /// Create with custom configuration
    pub async fn with_config(config: CollectiveConfig) -> Result<Self> {
        info!("Initializing Collective Mind v1.3.0");

        // Load or generate keypair
        let keypair = Self::load_or_generate_keypair().await?;

        // Load or create state
        let mut state = Self::load_or_create_state().await?;
        state.node_id = keypair.peer_id();

        // Initialize components
        let gossip = GossipEngine::new(state.node_id.clone(), keypair.private_key.clone().unwrap_or_default());
        let trust = TrustLedger::from_scores(state.trust_ledger.clone());
        let consensus = ConsensusEngine::from_records(state.consensus_history.clone());
        let sync = SyncManager::from_states(state.network_empathy.clone());
        let introspect = IntrospectionManager::new();

        info!(
            "Collective Mind initialized - Node ID: {}, Peers: {}",
            state.node_id,
            state.peers.len()
        );

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            gossip: Arc::new(RwLock::new(gossip)),
            trust: Arc::new(RwLock::new(trust)),
            consensus: Arc::new(RwLock::new(consensus)),
            sync: Arc::new(RwLock::new(sync)),
            introspect: Arc::new(RwLock::new(introspect)),
            keypair,
        })
    }

    /// Start the collective mind daemon
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Collective Mind is disabled in configuration");
            return Ok(());
        }

        info!("Starting Collective Mind daemon");

        // Start gossip engine
        let listen_addr: SocketAddr = format!("{}:{}", self.config.listen_addr, self.config.listen_port)
            .parse()
            .context("Invalid listen address")?;

        {
            let mut gossip = self.gossip.write().await;
            gossip.start(listen_addr).await?;

            // Announce self to network
            let peer_info = self.create_self_peer_info().await;
            gossip.announce_self(peer_info).await?;

            // Start heartbeat task
            gossip.spawn_heartbeat_task(self.config.heartbeat_interval_secs).await;
        }

        // Spawn periodic tasks
        self.spawn_empathy_sync_task().await;
        self.spawn_trust_decay_task().await;
        self.spawn_consensus_cleanup_task().await;
        self.spawn_state_persistence_task().await;

        info!("Collective Mind daemon started on {}", listen_addr);

        Ok(())
    }

    /// Create peer info for this node
    async fn create_self_peer_info(&self) -> PeerInfo {
        PeerInfo {
            id: self.keypair.peer_id(),
            name: format!("anna-{}", &self.keypair.peer_id()[..8]),
            address: format!("{}:{}", self.config.listen_addr, self.config.listen_port)
                .parse()
                .unwrap(),
            public_key: self.keypair.public_key.clone(),
            discovered_at: Utc::now(),
            last_seen: Utc::now(),
            protocol_version: "1.3.0".to_string(),
            connected: true,
        }
    }

    /// Spawn empathy synchronization task
    async fn spawn_empathy_sync_task(&self) {
        let gossip = Arc::clone(&self.gossip);
        let sync = Arc::clone(&self.sync);
        let interval_secs = self.config.empathy_sync_interval_secs;
        let node_id = self.keypair.peer_id();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                // Note: In full implementation, would fetch empathy state from empathy kernel
                // For now, this is a placeholder

                debug!("Empathy sync task tick");
            }
        });
    }

    /// Spawn trust decay task
    async fn spawn_trust_decay_task(&self) {
        let trust = Arc::clone(&self.trust);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(3600)); // Every hour

            loop {
                ticker.tick().await;

                let mut ledger = trust.write().await;
                ledger.apply_decay();
                debug!("Applied trust decay");
            }
        });
    }

    /// Spawn consensus cleanup task
    async fn spawn_consensus_cleanup_task(&self) {
        let consensus = Arc::clone(&self.consensus);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60)); // Every minute

            loop {
                ticker.tick().await;

                let mut engine = consensus.write().await;
                engine.cleanup_timeouts();
            }
        });
    }

    /// Spawn state persistence task
    async fn spawn_state_persistence_task(&self) {
        let state = Arc::clone(&self.state);
        let trust = Arc::clone(&self.trust);
        let consensus = Arc::clone(&self.consensus);
        let sync = Arc::clone(&self.sync);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(300)); // Every 5 minutes

            loop {
                ticker.tick().await;

                if let Err(e) = Self::save_state_internal(&state, &trust, &consensus, &sync).await {
                    error!("Failed to save collective state: {}", e);
                }
            }
        });
    }

    /// Get current status
    pub async fn get_status(&self) -> CollectiveStatus {
        let state = self.state.read().await;
        let gossip = self.gossip.read().await;
        let sync = self.sync.read().await;

        CollectiveStatus {
            enabled: self.config.enabled,
            node_id: state.node_id.clone(),
            connected_peers: gossip.connected_peer_count().await,
            total_peers: state.peers.len(),
            avg_network_empathy: sync.calculate_network_empathy(),
            avg_network_strain: sync.calculate_network_strain(),
            recent_decisions: state.consensus_history.len(),
            network_health: sync.calculate_network_health(),
        }
    }

    /// Get trust details for a peer
    pub async fn get_peer_trust(&self, peer_id: &PeerId) -> Option<PeerTrustDetails> {
        let state = self.state.read().await;
        let trust = self.trust.read().await;

        let peer_info = state.peers.get(peer_id)?;
        let trust_score = trust.get_score(peer_id);

        // Capture values before move
        let recent_messages = trust_score.messages_received;
        let last_interaction = trust_score.updated_at;

        Some(PeerTrustDetails {
            peer_info: peer_info.clone(),
            trust_score,
            recent_messages,
            last_interaction,
        })
    }

    /// Get explanation for a consensus decision
    pub async fn get_consensus_explanation(&self, consensus_id: &str) -> Option<ConsensusExplanation> {
        let state = self.state.read().await;
        let consensus = self.consensus.read().await;

        // Check completed records
        let record = consensus.get_completed().iter().find(|r| r.id == consensus_id)?;

        // Calculate approval metrics
        let total_votes = record.votes.len();
        if total_votes == 0 {
            return None;
        }

        let approve_count = record.votes.values().filter(|v| v.vote == VoteType::Approve).count();
        let approval_percentage = (approve_count as f64 / total_votes as f64) * 100.0;

        // Calculate weighted approval
        let trust = self.trust.read().await;
        let mut weighted_approve = 0.0;
        let mut total_weight = 0.0;

        for vote in record.votes.values() {
            let weight = trust.get_score(&vote.peer_id).overall * vote.weight;
            total_weight += weight;
            if vote.vote == VoteType::Approve {
                weighted_approve += weight;
            }
        }

        let weighted_approval = if total_weight > 0.0 {
            (weighted_approve / total_weight) * 100.0
        } else {
            0.0
        };

        // Find dissenting peers
        let dissenting_peers: Vec<PeerId> = record
            .votes
            .values()
            .filter(|v| v.vote == VoteType::Reject)
            .map(|v| v.peer_id.clone())
            .collect();

        Some(ConsensusExplanation {
            record: record.clone(),
            approval_percentage,
            weighted_approval,
            dissenting_peers,
            reasoning_trail: record.reasoning.clone(),
        })
    }

    /// Load or generate keypair
    async fn load_or_generate_keypair() -> Result<KeyPair> {
        let key_path = Path::new("/var/lib/anna/keys/collective.pem");

        if key_path.exists() {
            match KeyPair::load(key_path).await {
                Ok(keypair) => {
                    info!("Loaded existing collective keypair");
                    return Ok(keypair);
                }
                Err(e) => {
                    warn!("Failed to load keypair: {}, generating new", e);
                }
            }
        }

        info!("Generating new collective keypair");
        let keypair = KeyPair::generate()?;
        keypair.save(key_path).await?;
        Ok(keypair)
    }

    /// Load or create state
    async fn load_or_create_state() -> Result<CollectiveState> {
        let state_path = Path::new("/var/lib/anna/collective/state.json");

        if state_path.exists() {
            match fs::read_to_string(state_path).await {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(state) => {
                        info!("Loaded collective state from {:?}", state_path);
                        return Ok(state);
                    }
                    Err(e) => {
                        warn!("Failed to parse collective state: {}, using default", e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read collective state: {}, using default", e);
                }
            }
        }

        info!("Creating new collective state");
        Ok(CollectiveState::default())
    }

    /// Save state to disk
    pub async fn save_state(&self) -> Result<()> {
        Self::save_state_internal(&self.state, &self.trust, &self.consensus, &self.sync).await
    }

    /// Internal state save helper
    async fn save_state_internal(
        state: &Arc<RwLock<CollectiveState>>,
        trust: &Arc<RwLock<TrustLedger>>,
        consensus: &Arc<RwLock<ConsensusEngine>>,
        sync: &Arc<RwLock<SyncManager>>,
    ) -> Result<()> {
        let mut state_guard = state.write().await;

        // Update from components
        state_guard.trust_ledger = trust.read().await.get_all_scores().clone();
        state_guard.consensus_history = consensus.read().await.get_completed().to_vec();
        state_guard.network_empathy = sync.read().await.get_all_states().clone();
        state_guard.timestamp = Utc::now();

        // Write to disk
        let state_path = Path::new("/var/lib/anna/collective/state.json");
        fs::create_dir_all(state_path.parent().unwrap()).await?;

        let json = serde_json::to_string_pretty(&*state_guard)?;
        fs::write(state_path, json).await?;

        debug!("Collective state saved to {:?}", state_path);
        Ok(())
    }

    /// Get node ID
    pub fn get_node_id(&self) -> PeerId {
        self.keypair.peer_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collective_mind_creation() {
        let result = CollectiveMind::new().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_status() {
        let collective = CollectiveMind::new().await.unwrap();
        let status = collective.get_status().await;

        assert!(!status.enabled); // Disabled by default
        assert!(!status.node_id.is_empty());
    }
}
