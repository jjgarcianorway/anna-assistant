//! Sentinel daemon main loop
//!
//! Phase 1.0: Persistent autonomous system administrator
//! Phase 1.1: Conscience-integrated ethical governance
//! Citation: [archwiki:System_maintenance]

use super::events::{create_default_playbooks, EventBus};
use super::state::{load_config, load_state, save_state};
use super::types::{
    SentinelAction, SentinelConfig, SentinelEvent, SentinelLogEntry, SentinelMetrics, SentinelState,
};
use crate::conscience::ConscienceDaemon;
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

const SENTINEL_LOG_DIR: &str = "/var/log/anna";
const SENTINEL_LOG_FILE: &str = "sentinel.jsonl";

/// Sentinel daemon
pub struct SentinelDaemon {
    /// Shared state
    state: Arc<RwLock<SentinelState>>,
    /// Configuration
    config: Arc<RwLock<SentinelConfig>>,
    /// Event bus
    event_bus: Arc<EventBus>,
    /// Conscience daemon (Phase 1.1)
    conscience: Option<Arc<ConscienceDaemon>>,
    /// Empathy kernel (Phase 1.2)
    empathy: Option<Arc<crate::empathy::EmpathyKernel>>,
    /// Start time
    start_time: Instant,
}

impl SentinelDaemon {
    /// Create new sentinel daemon
    pub async fn new() -> Result<Self> {
        info!("Initializing Sentinel Daemon");

        // Load or create state
        let state = load_state().await.unwrap_or_default();

        // Load or create configuration
        let config = load_config().await.unwrap_or_default();

        // Create event bus
        let event_bus = Arc::new(EventBus::new());

        // Register default playbooks
        for playbook in create_default_playbooks() {
            event_bus.register_playbook(playbook).await;
        }

        // Initialize conscience layer (Phase 1.1)
        let conscience = match ConscienceDaemon::new().await {
            Ok(c) => {
                let conscience_arc = Arc::new(c);
                event_bus.set_conscience(Arc::clone(&conscience_arc)).await;
                info!("Conscience layer initialized and integrated");
                Some(conscience_arc)
            }
            Err(e) => {
                warn!("Failed to initialize conscience layer: {}", e);
                warn!("Continuing without ethical evaluation");
                None
            }
        };

        // Initialize empathy kernel (Phase 1.2)
        let empathy = match crate::empathy::EmpathyKernel::new().await {
            Ok(e) => {
                let empathy_arc = Arc::new(e);
                info!("Empathy kernel initialized");
                Some(empathy_arc)
            }
            Err(e) => {
                warn!("Failed to initialize empathy kernel: {}", e);
                warn!("Continuing without contextual awareness");
                None
            }
        };

        info!(
            "Sentinel initialized (autonomous_mode={}, conscience={}, empathy={})",
            config.autonomous_mode,
            conscience.is_some(),
            empathy.is_some()
        );

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            config: Arc::new(RwLock::new(config)),
            event_bus,
            conscience,
            empathy,
            start_time: Instant::now(),
        })
    }

    /// Run the sentinel daemon
    pub async fn run(&self) -> Result<()> {
        info!("Starting Sentinel Daemon");

        // Spawn periodic schedulers
        self.spawn_health_check_scheduler().await;
        self.spawn_update_scan_scheduler().await;
        self.spawn_audit_scheduler().await;
        self.spawn_state_persistence_scheduler().await;

        // Spawn conscience introspection scheduler (Phase 1.1)
        if self.conscience.is_some() {
            self.spawn_introspection_scheduler().await;
        }

        // Spawn empathy digest scheduler (Phase 1.2)
        if self.empathy.is_some() {
            self.spawn_empathy_digest_scheduler().await;
        }

        // Spawn event processor
        let event_bus = Arc::clone(&self.event_bus);
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            let _ = event_bus
                .process_events(move |event| handle_event(event, Arc::clone(&state)))
                .await;
        });

        // Keep daemon running
        info!("Sentinel Daemon running");
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            // Update uptime
            let mut state = self.state.write().await;
            state.uptime_seconds = self.start_time.elapsed().as_secs();
        }
    }

    /// Spawn health check scheduler
    async fn spawn_health_check_scheduler(&self) {
        let event_sender = self.event_bus.sender();
        let config = Arc::clone(&self.config);

        tokio::spawn(async move {
            loop {
                let interval_secs = config.read().await.health_check_interval;
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;

                info!("Triggering scheduled health check");
                let _ = event_sender.send(SentinelEvent::HealthCheck);
            }
        });
    }

    /// Spawn update scan scheduler
    async fn spawn_update_scan_scheduler(&self) {
        let event_sender = self.event_bus.sender();
        let config = Arc::clone(&self.config);

        tokio::spawn(async move {
            loop {
                let interval_secs = config.read().await.update_scan_interval;
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;

                info!("Triggering scheduled update scan");
                let _ = event_sender.send(SentinelEvent::UpdateScan);
            }
        });
    }

    /// Spawn audit scheduler
    async fn spawn_audit_scheduler(&self) {
        let event_sender = self.event_bus.sender();
        let config = Arc::clone(&self.config);

        tokio::spawn(async move {
            loop {
                let interval_secs = config.read().await.audit_interval;
                tokio::time::sleep(Duration::from_secs(interval_secs)).await;

                info!("Triggering scheduled audit");
                let _ = event_sender.send(SentinelEvent::Audit);
            }
        });
    }

    /// Spawn state persistence scheduler
    async fn spawn_state_persistence_scheduler(&self) {
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Save every minute

            loop {
                interval.tick().await;

                let state_snapshot = state.read().await.clone();
                if let Err(e) = save_state(&state_snapshot).await {
                    warn!("Failed to save state: {}", e);
                }
            }
        });
    }

    /// Spawn conscience introspection scheduler (Phase 1.1)
    async fn spawn_introspection_scheduler(&self) {
        let conscience = match &self.conscience {
            Some(c) => Arc::clone(c),
            None => return,
        };

        tokio::spawn(async move {
            // Run every 6 hours
            let mut interval = interval(Duration::from_secs(6 * 60 * 60));

            loop {
                interval.tick().await;

                info!("Running scheduled conscience introspection");
                match conscience.introspect().await {
                    Ok(report) => {
                        info!(
                            "Introspection complete: {} decisions reviewed, {} violations",
                            report.decisions_reviewed,
                            report.violations.len()
                        );

                        // Log any recommendations
                        for rec in &report.recommendations {
                            info!("Introspection recommendation: {}", rec);
                        }
                    }
                    Err(e) => {
                        error!("Introspection failed: {}", e);
                    }
                }
            }
        });
    }

    /// Spawn empathy digest scheduler (Phase 1.2)
    async fn spawn_empathy_digest_scheduler(&self) {
        let empathy = match &self.empathy {
            Some(e) => Arc::clone(e),
            None => return,
        };

        tokio::spawn(async move {
            // Run every 7 days (weekly)
            let mut interval = interval(Duration::from_secs(7 * 24 * 60 * 60));

            loop {
                interval.tick().await;

                info!("Generating weekly empathy digest");
                match empathy.generate_weekly_digest().await {
                    Ok(digest) => {
                        info!("Weekly empathy digest generated successfully");
                        debug!("Digest preview:\n{}", &digest[..digest.len().min(500)]);
                    }
                    Err(e) => {
                        error!("Failed to generate empathy digest: {}", e);
                    }
                }

                // Also save state periodically
                if let Err(e) = empathy.save_state().await {
                    error!("Failed to save empathy state: {}", e);
                }
            }
        });
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> SentinelMetrics {
        let state = self.state.read().await;

        let total_events: u64 = state.event_counters.values().sum();
        let manual_commands = state
            .event_counters
            .get("ManualCommand")
            .copied()
            .unwrap_or(0);

        SentinelMetrics {
            uptime_seconds: self.start_time.elapsed().as_secs(),
            total_events,
            events_by_type: state.event_counters.clone(),
            automated_actions: state.event_counters.get("AutoRepair").copied().unwrap_or(0)
                + state.event_counters.get("AutoUpdate").copied().unwrap_or(0),
            manual_commands,
            health_checks: state
                .event_counters
                .get("HealthCheck")
                .copied()
                .unwrap_or(0),
            update_scans: state.event_counters.get("UpdateScan").copied().unwrap_or(0),
            audits: state.event_counters.get("Audit").copied().unwrap_or(0),
            current_health: state.last_health.status.clone(),
            error_rate: state.error_rate,
            drift_index: state.drift_index,
            last_transition: None, // TODO: Track this
        }
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: SentinelConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;

        // Save to disk
        super::state::save_config(&config).await?;

        // Emit config changed event
        let _ = self.event_bus.sender().send(SentinelEvent::ConfigChanged);

        info!("Configuration updated");
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> SentinelConfig {
        self.config.read().await.clone()
    }

    /// Get conscience daemon reference (Phase 1.1)
    pub fn get_conscience(&self) -> Option<Arc<ConscienceDaemon>> {
        self.conscience.clone()
    }

    /// Get empathy kernel reference (Phase 1.2)
    pub fn get_empathy(&self) -> Option<Arc<crate::empathy::EmpathyKernel>> {
        self.empathy.clone()
    }
}

/// Handle a sentinel event
fn handle_event(event: SentinelEvent, state: Arc<RwLock<SentinelState>>) -> Vec<SentinelAction> {
    // Increment event counter
    let event_name = format!("{:?}", &event)
        .split('{')
        .next()
        .unwrap_or("Unknown")
        .to_string();
    tokio::spawn(async move {
        let mut state = state.write().await;
        *state.event_counters.entry(event_name).or_insert(0) += 1;
    });

    // Determine actions based on event type
    match event {
        SentinelEvent::HealthCheck => {
            // Trigger health check via steward module
            vec![SentinelAction::LogWarning {
                message: "Health check triggered".to_string(),
            }]
        }
        SentinelEvent::UpdateScan => {
            // Trigger update scan
            vec![SentinelAction::LogWarning {
                message: "Update scan triggered".to_string(),
            }]
        }
        SentinelEvent::Audit => {
            // Trigger audit
            vec![SentinelAction::LogWarning {
                message: "Audit triggered".to_string(),
            }]
        }
        SentinelEvent::ServiceFailed { service } => {
            // Auto-restart if enabled
            vec![
                SentinelAction::RestartService {
                    service: service.clone(),
                },
                SentinelAction::SendNotification {
                    title: "Service Failed".to_string(),
                    body: format!("Service {} failed and was restarted", service),
                },
            ]
        }
        SentinelEvent::LogAnomaly { severity, message } => {
            if severity == "critical" {
                vec![SentinelAction::SendNotification {
                    title: "Critical Log Anomaly".to_string(),
                    body: message,
                }]
            } else {
                vec![SentinelAction::LogWarning {
                    message: format!("[{}] {}", severity, message),
                }]
            }
        }
        SentinelEvent::PackageDrift { added, removed } => {
            vec![
                SentinelAction::LogWarning {
                    message: format!(
                        "Package drift detected: {} added, {} removed",
                        added, removed
                    ),
                },
                SentinelAction::SyncDatabases,
            ]
        }
        SentinelEvent::StateTransition { from, to } => {
            vec![SentinelAction::SendNotification {
                title: "System State Changed".to_string(),
                body: format!("State: {} → {}", from, to),
            }]
        }
        SentinelEvent::ConfigChanged => {
            vec![SentinelAction::LogWarning {
                message: "Configuration reloaded".to_string(),
            }]
        }
        SentinelEvent::ManualCommand { .. } => {
            // Manual commands don't trigger automatic actions
            vec![SentinelAction::None]
        }
        SentinelEvent::AutoRepair { target } => {
            vec![SentinelAction::RunRepair { probe: target }]
        }
        SentinelEvent::AutoUpdate { package_count } => {
            vec![
                SentinelAction::SystemUpdate { dry_run: false },
                SentinelAction::SendNotification {
                    title: "System Updated".to_string(),
                    body: format!("{} packages updated", package_count),
                },
            ]
        }
    }
}

/// Write sentinel log entry
pub async fn write_log_entry(entry: SentinelLogEntry) -> Result<()> {
    let log_dir = Path::new(SENTINEL_LOG_DIR);
    create_dir_all(log_dir)
        .await
        .context("Failed to create sentinel log directory")?;

    let log_path = log_dir.join(SENTINEL_LOG_FILE);
    let json = serde_json::to_string(&entry)? + "\n";

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await
        .context("Failed to open sentinel log")?;

    file.write_all(json.as_bytes())
        .await
        .context("Failed to write sentinel log entry")?;

    file.sync_all()
        .await
        .context("Failed to sync sentinel log")?;

    Ok(())
}
