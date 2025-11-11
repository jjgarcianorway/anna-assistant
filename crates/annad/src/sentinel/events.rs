//! Sentinel event bus and handler
//!
//! Phase 1.0: Unified event system for all subsystems
//! Phase 1.1: Conscience-integrated action execution
//! Citation: [archwiki:System_maintenance]

use super::types::{SentinelEvent, SentinelAction, ResponsePlaybook};
use crate::conscience::ConscienceDaemon;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

/// Event bus for sentinel
pub struct EventBus {
    sender: mpsc::UnboundedSender<SentinelEvent>,
    receiver: Arc<RwLock<mpsc::UnboundedReceiver<SentinelEvent>>>,
    playbooks: Arc<RwLock<Vec<ResponsePlaybook>>>,
    conscience: Arc<RwLock<Option<Arc<ConscienceDaemon>>>>,
}

impl EventBus {
    /// Create new event bus
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver: Arc::new(RwLock::new(receiver)),
            playbooks: Arc::new(RwLock::new(Vec::new())),
            conscience: Arc::new(RwLock::new(None)),
        }
    }

    /// Set conscience daemon for ethical evaluation
    pub async fn set_conscience(&self, conscience: Arc<ConscienceDaemon>) {
        info!("Conscience layer integrated with event bus");
        *self.conscience.write().await = Some(conscience);
    }

    /// Get sender handle for publishing events
    pub fn sender(&self) -> mpsc::UnboundedSender<SentinelEvent> {
        self.sender.clone()
    }

    /// Register a response playbook
    pub async fn register_playbook(&self, playbook: ResponsePlaybook) {
        let mut playbooks = self.playbooks.write().await;
        info!("Registering playbook: {}", playbook.name);
        playbooks.push(playbook);
    }

    /// Process events from the bus
    pub async fn process_events<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(SentinelEvent) -> Vec<SentinelAction> + Send,
    {
        let mut receiver = self.receiver.write().await;

        loop {
            match receiver.recv().await {
                Some(event) => {
                    info!("Processing event: {:?}", event);

                    // Get actions from handler
                    let actions = handler(event.clone());

                    // Execute actions
                    for action in actions {
                        if let Err(e) = self.execute_action(action).await {
                            warn!("Failed to execute action: {}", e);
                        }
                    }

                    // Check playbooks for additional actions
                    let playbook_actions = self.check_playbooks(&event).await;
                    for action in playbook_actions {
                        if let Err(e) = self.execute_action(action).await {
                            warn!("Failed to execute playbook action: {}", e);
                        }
                    }
                }
                None => {
                    info!("Event bus closed");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Check playbooks for matching triggers
    async fn check_playbooks(&self, event: &SentinelEvent) -> Vec<SentinelAction> {
        let playbooks = self.playbooks.read().await;
        let mut actions = Vec::new();

        for playbook in playbooks.iter() {
            if playbook.triggers.contains(event) {
                info!("Playbook triggered: {}", playbook.name);

                if playbook.requires_confirmation {
                    info!("Playbook requires confirmation, skipping automatic execution");
                    continue;
                }

                actions.extend(playbook.actions.clone());
            }
        }

        actions
    }

    /// Execute a sentinel action (with conscience evaluation)
    async fn execute_action(&self, action: SentinelAction) -> Result<()> {
        // Phase 1.1: Conscience pre-execution hook
        let conscience_opt = {
            let guard = self.conscience.read().await;
            guard.clone()
        };

        if let Some(conscience) = conscience_opt {
            match conscience.evaluate_action(&action).await {
                Ok((should_execute, decision_id)) => {
                    if !should_execute {
                        info!(
                            "Conscience blocked action: {:?} (decision: {})",
                            action, decision_id
                        );
                        return Ok(());
                    }
                    info!(
                        "Conscience approved action: {:?} (decision: {})",
                        action, decision_id
                    );

                    // Execute the action
                    let result = self.execute_action_impl(&action).await;

                    // Update conscience with execution result
                    let result_str = match &result {
                        Ok(_) => "Success".to_string(),
                        Err(e) => format!("Error: {}", e),
                    };

                    if let Err(e) = conscience.update_outcome(&decision_id, result_str).await {
                        warn!("Failed to update conscience outcome: {}", e);
                    }

                    result
                }
                Err(e) => {
                    warn!("Conscience evaluation failed: {}", e);
                    // Fail-safe: don't execute if conscience evaluation fails
                    Err(e)
                }
            }
        } else {
            // No conscience layer, execute directly
            self.execute_action_impl(&action).await
        }
    }

    /// Execute action implementation (actual work)
    async fn execute_action_impl(&self, action: &SentinelAction) -> Result<()> {
        match action {
            SentinelAction::None => {
                // No-op
            }
            SentinelAction::RestartService { service } => {
                info!("Executing: Restart service {}", service);
                // This would call into the repair module
                // For now, just log the action
            }
            SentinelAction::SyncDatabases => {
                info!("Executing: Sync package databases");
                // This would call into the repair module
            }
            SentinelAction::SystemUpdate { dry_run } => {
                info!("Executing: System update (dry_run={})", dry_run);
                // This would call into the steward update module
            }
            SentinelAction::RunRepair { probe } => {
                info!("Executing: Run repair for {}", probe);
                // This would call into the repair module
            }
            SentinelAction::LogWarning { message } => {
                warn!("Sentinel warning: {}", message);
            }
            SentinelAction::SendNotification { title, body } => {
                info!("Sending notification: {} - {}", title, body);
                // This would send a desktop notification
                let _ = std::process::Command::new("notify-send")
                    .arg("--app-name=Anna Assistant")
                    .arg("--icon=dialog-information")
                    .arg(&title)
                    .arg(&body)
                    .spawn();
            }
        }

        Ok(())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default response playbooks
pub fn create_default_playbooks() -> Vec<ResponsePlaybook> {
    vec![
        // Service failure auto-restart
        ResponsePlaybook {
            name: "auto-restart-failed-services".to_string(),
            triggers: vec![SentinelEvent::ServiceFailed {
                service: String::new(), // Matches any service
            }],
            actions: vec![
                SentinelAction::LogWarning {
                    message: "Service failure detected".to_string(),
                },
                SentinelAction::SendNotification {
                    title: "Service Failure".to_string(),
                    body: "Anna detected a failed service".to_string(),
                },
            ],
            requires_confirmation: false,
        },
        // Package drift notification
        ResponsePlaybook {
            name: "notify-package-drift".to_string(),
            triggers: vec![SentinelEvent::PackageDrift {
                added: 0,
                removed: 0,
            }],
            actions: vec![
                SentinelAction::LogWarning {
                    message: "Package drift detected".to_string(),
                },
                SentinelAction::SendNotification {
                    title: "Package Drift".to_string(),
                    body: "System packages changed unexpectedly".to_string(),
                },
            ],
            requires_confirmation: false,
        },
        // Log anomaly notification
        ResponsePlaybook {
            name: "notify-log-anomaly".to_string(),
            triggers: vec![SentinelEvent::LogAnomaly {
                severity: "critical".to_string(),
                message: String::new(),
            }],
            actions: vec![
                SentinelAction::SendNotification {
                    title: "Critical Log Anomaly".to_string(),
                    body: "Anna detected critical system errors".to_string(),
                },
            ],
            requires_confirmation: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new();
        let sender = bus.sender();

        // Should be able to send events
        assert!(sender.send(SentinelEvent::HealthCheck).is_ok());
    }

    #[tokio::test]
    async fn test_playbook_registration() {
        let bus = EventBus::new();
        let playbook = ResponsePlaybook {
            name: "test".to_string(),
            triggers: vec![SentinelEvent::HealthCheck],
            actions: vec![SentinelAction::None],
            requires_confirmation: false,
        };

        bus.register_playbook(playbook).await;

        let playbooks = bus.playbooks.read().await;
        assert_eq!(playbooks.len(), 1);
    }
}
