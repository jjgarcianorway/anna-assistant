//! Gossip protocol for peer discovery and event propagation
//!
//! Phase 1.3: Peer-to-peer communication layer
//! Citation: [archwiki:System_maintenance]

use super::crypto::{sign_message, verify_signature};
use super::types::{GossipMessage, PeerId, PeerInfo};
use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// Gossip engine for peer-to-peer communication
pub struct GossipEngine {
    /// This node's peer ID
    node_id: PeerId,
    /// Private key for signing
    private_key: String,
    /// Known peers
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    /// Recently seen message IDs (for deduplication)
    seen_messages: Arc<RwLock<HashSet<String>>>,
    /// UDP socket for gossip
    socket: Option<Arc<UdpSocket>>,
    /// Message handlers
    handlers: Arc<RwLock<Vec<Box<dyn MessageHandler + Send + Sync>>>>,
}

/// Message handler trait
pub trait MessageHandler {
    fn handle_message(&self, message: &GossipMessage, from: SocketAddr) -> Result<()>;
}

impl GossipEngine {
    /// Create new gossip engine
    pub fn new(node_id: PeerId, private_key: String) -> Self {
        Self {
            node_id,
            private_key,
            peers: Arc::new(RwLock::new(HashMap::new())),
            seen_messages: Arc::new(RwLock::new(HashSet::new())),
            socket: None,
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start listening for gossip messages
    pub async fn start(&mut self, listen_addr: SocketAddr) -> Result<()> {
        let socket = UdpSocket::bind(listen_addr).await?;
        info!("Gossip engine listening on {}", listen_addr);

        let socket_arc = Arc::new(socket);
        self.socket = Some(Arc::clone(&socket_arc));

        // Spawn listener task
        let peers = Arc::clone(&self.peers);
        let seen_messages = Arc::clone(&self.seen_messages);
        let handlers = Arc::clone(&self.handlers);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];

            loop {
                match socket_arc.recv_from(&mut buf).await {
                    Ok((len, from)) => {
                        if let Ok(json) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(message) = serde_json::from_str::<GossipMessage>(json) {
                                Self::handle_received_message(
                                    message,
                                    from,
                                    &peers,
                                    &seen_messages,
                                    &handlers,
                                )
                                .await;
                            } else {
                                debug!("Failed to parse gossip message from {}", from);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving gossip message: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle received gossip message
    async fn handle_received_message(
        message: GossipMessage,
        from: SocketAddr,
        peers: &Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        seen_messages: &Arc<RwLock<HashSet<String>>>,
        handlers: &Arc<RwLock<Vec<Box<dyn MessageHandler + Send + Sync>>>>,
    ) {
        // Generate message ID for deduplication
        let message_id = Self::generate_message_id(&message);

        // Check if we've seen this message before
        {
            let mut seen = seen_messages.write().await;
            if !seen.insert(message_id.clone()) {
                debug!("Ignoring duplicate message {}", message_id);
                return;
            }

            // Limit seen messages cache size
            if seen.len() > 10000 {
                seen.clear();
            }
        }

        // Verify signature based on message type
        let verified = Self::verify_message_signature(&message);
        if !verified {
            warn!("Invalid signature for message from {}", from);
            return;
        }

        debug!("Received gossip message: {:?}", message);

        // Update peer info if applicable
        match &message {
            GossipMessage::PeerAnnounce { peer_info, .. } => {
                let mut peer_map = peers.write().await;
                let mut info = peer_info.clone();
                info.last_seen = Utc::now();
                info.connected = true;
                peer_map.insert(info.id.clone(), info);
            }
            GossipMessage::Heartbeat { peer_id, .. } => {
                let mut peer_map = peers.write().await;
                if let Some(peer) = peer_map.get_mut(peer_id) {
                    peer.last_seen = Utc::now();
                    peer.connected = true;
                }
            }
            _ => {}
        }

        // Call registered handlers
        let handlers_vec = handlers.read().await;
        for handler in handlers_vec.iter() {
            if let Err(e) = handler.handle_message(&message, from) {
                error!("Handler error: {}", e);
            }
        }
    }

    /// Generate unique message ID
    fn generate_message_id(message: &GossipMessage) -> String {
        // Simple ID based on message content hash
        format!("{:?}", message).chars().take(32).collect()
    }

    /// Verify message signature
    fn verify_message_signature(message: &GossipMessage) -> bool {
        // Extract signature and verify based on message type
        match message {
            GossipMessage::PeerAnnounce {
                peer_info,
                signature,
            } => {
                let payload = format!("{:?}", peer_info);
                verify_signature(&payload, signature, &peer_info.public_key)
            }
            GossipMessage::Heartbeat {
                peer_id,
                signature,
                ..
            } => {
                let payload = format!("{}", peer_id);
                verify_signature(&payload, signature, "placeholder_key")
            }
            _ => true, // Other messages verified by handlers
        }
    }

    /// Send gossip message to a peer
    pub async fn send_to_peer(&self, message: &GossipMessage, peer_addr: SocketAddr) -> Result<()> {
        if let Some(socket) = &self.socket {
            let json = serde_json::to_string(message)?;
            socket.send_to(json.as_bytes(), peer_addr).await?;
            debug!("Sent gossip message to {}", peer_addr);
        }
        Ok(())
    }

    /// Broadcast message to all known peers
    pub async fn broadcast(&self, message: &GossipMessage) -> Result<()> {
        let peers = self.peers.read().await;

        for peer in peers.values() {
            if peer.connected {
                if let Err(e) = self.send_to_peer(message, peer.address).await {
                    warn!("Failed to send to {}: {}", peer.id, e);
                }
            }
        }

        Ok(())
    }

    /// Announce this node to the network
    pub async fn announce_self(&self, peer_info: PeerInfo) -> Result<()> {
        let payload = format!("{:?}", peer_info);
        let signature = sign_message(&payload, &self.private_key);

        let message = GossipMessage::PeerAnnounce {
            peer_info,
            signature,
        };

        self.broadcast(&message).await
    }

    /// Send heartbeat
    pub async fn send_heartbeat(&self) -> Result<()> {
        let payload = format!("{}", self.node_id);
        let signature = sign_message(&payload, &self.private_key);

        let message = GossipMessage::Heartbeat {
            peer_id: self.node_id.clone(),
            timestamp: Utc::now(),
            signature,
        };

        self.broadcast(&message).await
    }

    /// Register message handler
    pub async fn register_handler(&self, handler: Box<dyn MessageHandler + Send + Sync>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }

    /// Add peer manually
    pub async fn add_peer(&self, peer: PeerInfo) {
        let mut peers = self.peers.write().await;
        peers.insert(peer.id.clone(), peer);
    }

    /// Get all known peers
    pub async fn get_peers(&self) -> HashMap<PeerId, PeerInfo> {
        self.peers.read().await.clone()
    }

    /// Get connected peer count
    pub async fn connected_peer_count(&self) -> usize {
        self.peers
            .read()
            .await
            .values()
            .filter(|p| p.connected)
            .count()
    }

    /// Spawn periodic heartbeat task
    pub async fn spawn_heartbeat_task(&self, interval_secs: u64) {
        let node_id = self.node_id.clone();
        let private_key = self.private_key.clone();
        let peers = Arc::clone(&self.peers);
        let socket = self.socket.as_ref().map(Arc::clone);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                if let Some(ref sock) = socket {
                    let payload = format!("{}", node_id);
                    let signature = sign_message(&payload, &private_key);

                    let message = GossipMessage::Heartbeat {
                        peer_id: node_id.clone(),
                        timestamp: Utc::now(),
                        signature,
                    };

                    if let Ok(json) = serde_json::to_string(&message) {
                        let peer_list = peers.read().await;
                        for peer in peer_list.values() {
                            if peer.connected {
                                let _ = sock.send_to(json.as_bytes(), peer.address).await;
                            }
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gossip_engine_creation() {
        let engine = GossipEngine::new("test_node".to_string(), "test_key".to_string());
        assert_eq!(engine.node_id, "test_node");
    }

    #[tokio::test]
    async fn test_add_peer() {
        let engine = GossipEngine::new("test_node".to_string(), "test_key".to_string());

        let peer = PeerInfo {
            id: "peer1".to_string(),
            name: "Test Peer".to_string(),
            address: "127.0.0.1:8742".parse().unwrap(),
            public_key: "test_pub_key".to_string(),
            discovered_at: Utc::now(),
            last_seen: Utc::now(),
            protocol_version: "1.3.0".to_string(),
            connected: false,
        };

        engine.add_peer(peer.clone()).await;

        let peers = engine.get_peers().await;
        assert!(peers.contains_key("peer1"));
    }
}
