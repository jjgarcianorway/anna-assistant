//! Global Daemon State - Shared state across RPC handlers
//!
//! Holds policy engine, event dispatcher, telemetry snapshot, and learning cache

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

use crate::policy::PolicyEngine;
use crate::events::{EventDispatcher, Event, EventType, EventSeverity};
use crate::learning::LearningCache;

/// Telemetry snapshot - cheap runtime metrics
#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    pub disk_free_pct: f64,
    pub last_quickscan_hours: f64,
    pub uptime_minutes: f64,
    pub last_updated: u64,
}

impl TelemetrySnapshot {
    pub fn new() -> Self {
        Self {
            disk_free_pct: 100.0,
            last_quickscan_hours: 0.0,
            uptime_minutes: 0.0,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Update disk free percentage (cheap query)
    pub fn update_disk_free(&mut self) {
        // Simplified: read from df or statfs
        // For now, use a placeholder value
        self.disk_free_pct = 75.0; // Would be real df output
        self.update_timestamp();
    }

    /// Update last quickscan time
    pub fn update_quickscan(&mut self) {
        // This would read from state file
        self.last_quickscan_hours = 2.5; // Would be real calculation
        self.update_timestamp();
    }

    /// Update uptime
    pub fn update_uptime(&mut self, start_time: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.uptime_minutes = ((now - start_time) as f64) / 60.0;
        self.update_timestamp();
    }

    fn update_timestamp(&mut self) {
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Global daemon state
pub struct DaemonState {
    pub policy_engine: Arc<PolicyEngine>,
    pub event_dispatcher: Arc<EventDispatcher>,
    pub telemetry: Arc<Mutex<TelemetrySnapshot>>,
    pub learning_cache: Arc<Mutex<LearningCache>>,
    pub start_time: u64,
}

impl DaemonState {
    /// Create new daemon state with all subsystems initialized
    pub fn new() -> Result<Self> {
        let policy_engine = Arc::new(PolicyEngine::new("/etc/anna/policies.d"));

        // Load policies from disk
        if let Err(e) = policy_engine.load_policies() {
            tracing::warn!("Failed to load policies: {}", e);
        }

        let event_dispatcher = Arc::new(EventDispatcher::new(policy_engine.clone()));
        let telemetry = Arc::new(Mutex::new(TelemetrySnapshot::new()));

        let learning_cache = Arc::new(Mutex::new(LearningCache::new("/var/lib/anna/learning.json")));

        // Try to load learning cache, ignore error if not exists
        if let Err(e) = learning_cache.lock().unwrap().load() {
            tracing::info!("Learning cache not loaded: {}", e);
        }

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            policy_engine,
            event_dispatcher,
            telemetry,
            learning_cache,
            start_time,
        })
    }

    /// Emit bootstrap events on daemon startup
    pub fn emit_bootstrap_events(&self) -> Result<()> {
        // Event 1: DaemonStarted
        let event1 = Event::new(
            EventType::SystemStartup,
            EventSeverity::Info,
            "annad",
            "Anna Assistant Daemon started"
        ).with_metadata(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "start_time": self.start_time,
        }));
        self.event_dispatcher.dispatch(event1)?;

        // Event 2: DoctorBootstrap
        let event2 = Event::new(
            EventType::Custom("DoctorBootstrap".to_string()),
            EventSeverity::Info,
            "installer",
            "Doctor repair bootstrap completed"
        ).with_metadata(serde_json::json!({
            "phase": "post-install",
        }));
        self.event_dispatcher.dispatch(event2)?;

        // Event 3: ConfigLoaded
        let event3 = Event::new(
            EventType::ConfigChange,
            EventSeverity::Info,
            "config",
            "Configuration loaded successfully"
        ).with_metadata(serde_json::json!({
            "config_files": ["/etc/anna/config.toml"],
        }));
        self.event_dispatcher.dispatch(event3)?;

        tracing::info!("[BOOT] Bootstrap events emitted (3 events)");
        Ok(())
    }

    /// Reload policies from disk
    pub fn reload_policies(&self) -> Result<usize> {
        self.policy_engine.reload()?;
        let count = self.policy_engine.rule_count();
        tracing::info!("[POLICY] Reloaded {} policy rules", count);
        Ok(count)
    }

    /// Update telemetry snapshot
    pub fn update_telemetry(&self) {
        let mut telemetry = self.telemetry.lock().unwrap();
        telemetry.update_disk_free();
        telemetry.update_quickscan();
        telemetry.update_uptime(self.start_time);
    }
}
