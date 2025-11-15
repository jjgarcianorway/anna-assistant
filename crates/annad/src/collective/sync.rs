//! State synchronization across collective network
//!
//! Phase 1.3: Cross-empathy sync and distributed state
//! Citation: [archwiki:System_maintenance]

use super::types::{NetworkEmpathyState, PeerId};
use std::collections::HashMap;
use tracing::debug;

/// State synchronization manager
pub struct SyncManager {
    /// Network empathy states from all peers
    empathy_states: HashMap<PeerId, NetworkEmpathyState>,
    /// Last sync timestamp per peer
    last_sync: HashMap<PeerId, chrono::DateTime<chrono::Utc>>,
}

impl SyncManager {
    /// Create new sync manager
    pub fn new() -> Self {
        Self {
            empathy_states: HashMap::new(),
            last_sync: HashMap::new(),
        }
    }

    /// Load from existing states
    pub fn from_states(states: HashMap<PeerId, NetworkEmpathyState>) -> Self {
        let last_sync = states
            .iter()
            .map(|(id, state)| (id.clone(), state.timestamp))
            .collect();

        Self {
            empathy_states: states,
            last_sync,
        }
    }

    /// Update empathy state for a peer
    pub fn update_peer_state(&mut self, state: NetworkEmpathyState) {
        let peer_id = state.peer_id.clone();

        debug!(
            "Syncing empathy state from {}: empathy={:.2}, strain={:.2}",
            peer_id, state.empathy_index, state.strain_index
        );

        self.last_sync.insert(peer_id.clone(), chrono::Utc::now());
        self.empathy_states.insert(peer_id, state);
    }

    /// Get empathy state for a peer
    pub fn get_peer_state(&self, peer_id: &PeerId) -> Option<&NetworkEmpathyState> {
        self.empathy_states.get(peer_id)
    }

    /// Get all empathy states
    pub fn get_all_states(&self) -> &HashMap<PeerId, NetworkEmpathyState> {
        &self.empathy_states
    }

    /// Calculate network-wide average empathy
    pub fn calculate_network_empathy(&self) -> f64 {
        if self.empathy_states.is_empty() {
            return 0.0;
        }

        let sum: f64 = self
            .empathy_states
            .values()
            .map(|s| s.empathy_index)
            .sum();

        sum / self.empathy_states.len() as f64
    }

    /// Calculate network-wide average strain
    pub fn calculate_network_strain(&self) -> f64 {
        if self.empathy_states.is_empty() {
            return 0.0;
        }

        let sum: f64 = self
            .empathy_states
            .values()
            .map(|s| s.strain_index)
            .sum();

        sum / self.empathy_states.len() as f64
    }

    /// Calculate average resonance for a stakeholder across network
    pub fn calculate_network_resonance(&self, stakeholder: &str) -> f64 {
        if self.empathy_states.is_empty() {
            return 0.0;
        }

        let sum: f64 = self
            .empathy_states
            .values()
            .map(|s| match stakeholder {
                "user" => s.user_resonance,
                "system" => s.system_resonance,
                "environment" => s.environment_resonance,
                _ => 0.0,
            })
            .sum();

        sum / self.empathy_states.len() as f64
    }

    /// Get peers under high strain
    pub fn get_strained_peers(&self, threshold: f64) -> Vec<PeerId> {
        self.empathy_states
            .iter()
            .filter(|(_, state)| state.strain_index > threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get network health score (0.0-1.0)
    pub fn calculate_network_health(&self) -> f64 {
        if self.empathy_states.is_empty() {
            return 0.0;
        }

        // Network health based on:
        // - Average empathy (higher is better)
        // - Low strain (lower is better)
        // - Number of recent syncs (recency matters)

        let avg_empathy = self.calculate_network_empathy();
        let avg_strain = self.calculate_network_strain();

        // Recent syncs (within last 10 minutes)
        let now = chrono::Utc::now();
        let recent_syncs = self
            .last_sync
            .values()
            .filter(|&&ts| {
                let duration = now.signed_duration_since(ts);
                duration.num_minutes() < 10
            })
            .count();

        let sync_ratio = recent_syncs as f64 / self.empathy_states.len() as f64;

        // Weighted health score
        let health = avg_empathy * 0.4 + (1.0 - avg_strain) * 0.4 + sync_ratio * 0.2;

        health.max(0.0).min(1.0)
    }

    /// Remove stale peer states (no sync for >1 hour)
    pub fn cleanup_stale_states(&mut self) {
        let now = chrono::Utc::now();
        let mut to_remove = Vec::new();

        for (peer_id, &last_sync_time) in &self.last_sync {
            let duration = now.signed_duration_since(last_sync_time);
            if duration.num_hours() > 1 {
                to_remove.push(peer_id.clone());
            }
        }

        for peer_id in to_remove {
            debug!("Removing stale empathy state for {}", peer_id);
            self.empathy_states.remove(&peer_id);
            self.last_sync.remove(&peer_id);
        }
    }

    /// Get sync statistics
    pub fn get_sync_stats(&self) -> SyncStats {
        let now = chrono::Utc::now();

        let recent_syncs = self
            .last_sync
            .values()
            .filter(|&&ts| {
                let duration = now.signed_duration_since(ts);
                duration.num_minutes() < 5
            })
            .count();

        let oldest_sync = self
            .last_sync
            .values()
            .min()
            .map(|&ts| now.signed_duration_since(ts).num_seconds())
            .unwrap_or(0);

        SyncStats {
            total_peers: self.empathy_states.len(),
            recent_syncs,
            oldest_sync_secs: oldest_sync,
            network_empathy: self.calculate_network_empathy(),
            network_strain: self.calculate_network_strain(),
            network_health: self.calculate_network_health(),
        }
    }
}

/// Sync statistics
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub total_peers: usize,
    pub recent_syncs: usize,
    pub oldest_sync_secs: i64,
    pub network_empathy: f64,
    pub network_strain: f64,
    pub network_health: f64,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use approx::assert_relative_eq;

    #[test]
    fn test_sync_manager_creation() {
        let manager = SyncManager::new();
        assert_eq!(manager.calculate_network_empathy(), 0.0);
    }

    #[test]
    fn test_update_peer_state() {
        let mut manager = SyncManager::new();

        let state = NetworkEmpathyState {
            peer_id: "peer1".to_string(),
            empathy_index: 0.8,
            strain_index: 0.3,
            user_resonance: 0.7,
            system_resonance: 0.9,
            environment_resonance: 0.6,
            timestamp: Utc::now(),
        };

        manager.update_peer_state(state);

        assert_eq!(manager.calculate_network_empathy(), 0.8);
        assert_eq!(manager.calculate_network_strain(), 0.3);
    }

    #[test]
    fn test_network_averages() {
        let mut manager = SyncManager::new();

        manager.update_peer_state(NetworkEmpathyState {
            peer_id: "peer1".to_string(),
            empathy_index: 0.8,
            strain_index: 0.2,
            user_resonance: 0.9,
            system_resonance: 0.8,
            environment_resonance: 0.7,
            timestamp: Utc::now(),
        });

        manager.update_peer_state(NetworkEmpathyState {
            peer_id: "peer2".to_string(),
            empathy_index: 0.6,
            strain_index: 0.4,
            user_resonance: 0.7,
            system_resonance: 0.6,
            environment_resonance: 0.5,
            timestamp: Utc::now(),
        });

        assert_relative_eq!(manager.calculate_network_empathy(), 0.7, epsilon = 1e-10);
        assert_relative_eq!(manager.calculate_network_strain(), 0.3, epsilon = 1e-10);
        assert_relative_eq!(manager.calculate_network_resonance("user"), 0.8, epsilon = 1e-10);
    }

    #[test]
    fn test_strained_peers() {
        let mut manager = SyncManager::new();

        manager.update_peer_state(NetworkEmpathyState {
            peer_id: "peer1".to_string(),
            empathy_index: 0.5,
            strain_index: 0.8, // High strain
            user_resonance: 0.5,
            system_resonance: 0.5,
            environment_resonance: 0.5,
            timestamp: Utc::now(),
        });

        manager.update_peer_state(NetworkEmpathyState {
            peer_id: "peer2".to_string(),
            empathy_index: 0.7,
            strain_index: 0.2, // Low strain
            user_resonance: 0.7,
            system_resonance: 0.7,
            environment_resonance: 0.7,
            timestamp: Utc::now(),
        });

        let strained = manager.get_strained_peers(0.6);
        assert_eq!(strained.len(), 1);
        assert!(strained.contains(&"peer1".to_string()));
    }
}
