//! Distributed introspection - cross-node ethical audits
//!
//! Phase 1.3: Peer introspection requests and sharing
//! Citation: [archwiki:System_maintenance]

use super::types::{IntrospectionRequest, IntrospectionResponse, PeerId};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// Distributed introspection manager
pub struct IntrospectionManager {
    /// Pending introspection requests
    pending_requests: Arc<RwLock<HashMap<String, IntrospectionRequest>>>,
    /// Completed introspection responses
    responses: Arc<RwLock<HashMap<String, IntrospectionResponse>>>,
}

impl IntrospectionManager {
    /// Create new introspection manager
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Request introspection from a peer
    pub async fn request_introspection(
        &self,
        target_peer: PeerId,
        requester: PeerId,
        query: String,
    ) -> String {
        let request_id = Uuid::new_v4().to_string();

        let request = IntrospectionRequest {
            id: request_id.clone(),
            requester,
            target_peer: target_peer.clone(),
            query: query.clone(),
            timestamp: Utc::now(),
        };

        self.pending_requests
            .write()
            .await
            .insert(request_id.clone(), request);

        info!(
            "Introspection request {} sent to peer {} for query: {}",
            request_id, target_peer, query
        );

        request_id
    }

    /// Handle incoming introspection request
    pub async fn handle_request(
        &self,
        request_id: String,
        requester: PeerId,
        query: String,
    ) -> IntrospectionResponse {
        info!(
            "Received introspection request {} from {}: {}",
            request_id, requester, query
        );

        // Perform introspection based on query type
        let data = match query.as_str() {
            "conscience_status" => {
                // Query conscience layer status
                self.introspect_conscience().await
            }
            "empathy_pulse" => {
                // Query empathy kernel pulse
                self.introspect_empathy().await
            }
            "system_health" => {
                // Query system health
                self.introspect_health().await
            }
            _ => {
                format!("Unknown introspection query: {}", query)
            }
        };

        let response = IntrospectionResponse {
            request_id: request_id.clone(),
            responder: "self".to_string(), // Will be replaced with actual node ID
            data,
            timestamp: Utc::now(),
        };

        // Store response
        self.responses
            .write()
            .await
            .insert(request_id.clone(), response.clone());

        debug!("Introspection response {} generated", request_id);

        response
    }

    /// Get response for a request
    pub async fn get_response(&self, request_id: &str) -> Option<IntrospectionResponse> {
        self.responses.read().await.get(request_id).cloned()
    }

    /// Get all pending requests
    pub async fn get_pending_requests(&self) -> Vec<IntrospectionRequest> {
        self.pending_requests
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Complete a request (remove from pending)
    pub async fn complete_request(&self, request_id: &str) {
        self.pending_requests.write().await.remove(request_id);
        debug!("Introspection request {} completed", request_id);
    }

    /// Introspect conscience layer
    async fn introspect_conscience(&self) -> String {
        // Placeholder: In full implementation, would query actual conscience state
        format!(
            "Conscience Status:\n  Pending Actions: 0\n  Last Review: N/A\n  Ethics Score: 0.95"
        )
    }

    /// Introspect empathy kernel
    async fn introspect_empathy(&self) -> String {
        // Placeholder: In full implementation, would query actual empathy state
        format!(
            "Empathy Pulse:\n  Empathy Index: 0.75\n  Strain Index: 0.25\n  User Resonance: 0.80"
        )
    }

    /// Introspect system health
    async fn introspect_health(&self) -> String {
        // Placeholder: In full implementation, would query actual health metrics
        format!(
            "System Health:\n  CPU: 15%\n  Memory: 2.3GB\n  Load Average: 0.8\n  Uptime: 3 days"
        )
    }

    /// Cleanup old responses (older than 1 hour)
    pub async fn cleanup_old_responses(&self) {
        let now = Utc::now();
        let mut responses = self.responses.write().await;
        let mut to_remove = Vec::new();

        for (id, response) in responses.iter() {
            let age = now.signed_duration_since(response.timestamp);
            if age.num_hours() > 1 {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            responses.remove(&id);
            debug!("Removed old introspection response: {}", id);
        }
    }
}

impl Default for IntrospectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_introspection_request() {
        let manager = IntrospectionManager::new();

        let request_id = manager
            .request_introspection(
                "peer1".to_string(),
                "self".to_string(),
                "conscience_status".to_string(),
            )
            .await;

        let pending = manager.get_pending_requests().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, request_id);
    }

    #[tokio::test]
    async fn test_introspection_response() {
        let manager = IntrospectionManager::new();

        let response = manager
            .handle_request(
                "test_request".to_string(),
                "peer1".to_string(),
                "empathy_pulse".to_string(),
            )
            .await;

        assert_eq!(response.request_id, "test_request");
        assert!(response.data.contains("Empathy Pulse"));

        let stored = manager.get_response("test_request").await;
        assert!(stored.is_some());
    }

    #[tokio::test]
    async fn test_complete_request() {
        let manager = IntrospectionManager::new();

        let request_id = manager
            .request_introspection(
                "peer1".to_string(),
                "self".to_string(),
                "system_health".to_string(),
            )
            .await;

        manager.complete_request(&request_id).await;

        let pending = manager.get_pending_requests().await;
        assert_eq!(pending.len(), 0);
    }
}
