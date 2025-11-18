//! Sentinel types and data structures
//!
//! Phase 1.0: Persistent autonomous daemon types
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sentinel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelConfig {
    /// Enable autonomous operations
    pub autonomous_mode: bool,
    /// Health check interval (seconds)
    pub health_check_interval: u64,
    /// Update scan interval (seconds)
    pub update_scan_interval: u64,
    /// Audit interval (seconds)
    pub audit_interval: u64,
    /// Auto-repair failed services
    pub auto_repair_services: bool,
    /// Auto-update packages (with approval threshold)
    pub auto_update: bool,
    /// Maximum packages to auto-update without confirmation
    pub auto_update_threshold: u32,
    /// Enable adaptive scheduling
    pub adaptive_scheduling: bool,
}

impl Default for SentinelConfig {
    fn default() -> Self {
        Self {
            autonomous_mode: false,
            health_check_interval: 300, // 5 minutes
            update_scan_interval: 3600, // 1 hour
            audit_interval: 86400,      // 24 hours
            auto_repair_services: false,
            auto_update: false,
            auto_update_threshold: 5,
            adaptive_scheduling: true,
        }
    }
}

/// Sentinel state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelState {
    /// State version (incremented on each update)
    pub version: u64,
    /// Last update timestamp
    pub timestamp: DateTime<Utc>,
    /// System state (iso_live, configured, degraded, etc.)
    pub system_state: String,
    /// Last health status
    pub last_health: HealthSnapshot,
    /// Last update check
    pub last_update_check: Option<DateTime<Utc>>,
    /// Last audit
    pub last_audit: Option<DateTime<Utc>>,
    /// Sentinel uptime (seconds)
    pub uptime_seconds: u64,
    /// Event counters
    pub event_counters: HashMap<String, u64>,
    /// Error rate (errors per hour)
    pub error_rate: f64,
    /// System drift index (0.0 = stable, 1.0 = critical drift)
    pub drift_index: f64,
}

impl Default for SentinelState {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: Utc::now(),
            system_state: "unknown".to_string(),
            last_health: HealthSnapshot::default(),
            last_update_check: None,
            last_audit: None,
            uptime_seconds: 0,
            event_counters: HashMap::new(),
            error_rate: 0.0,
            drift_index: 0.0,
        }
    }
}

/// Health snapshot for state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Overall status (healthy, degraded, critical)
    pub status: String,
    /// Failed services count
    pub failed_services: u32,
    /// Available updates count
    pub available_updates: u32,
    /// Log issues count
    pub log_issues: u32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl Default for HealthSnapshot {
    fn default() -> Self {
        Self {
            status: "unknown".to_string(),
            failed_services: 0,
            available_updates: 0,
            log_issues: 0,
            timestamp: Utc::now(),
        }
    }
}

/// Sentinel event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SentinelEvent {
    /// Periodic health check triggered
    HealthCheck,
    /// Periodic update scan triggered
    UpdateScan,
    /// Periodic audit triggered
    Audit,
    /// Service failure detected
    ServiceFailed { service: String },
    /// Log anomaly detected
    LogAnomaly { severity: String, message: String },
    /// Package drift detected (installed packages changed unexpectedly)
    PackageDrift { added: u32, removed: u32 },
    /// System state transition
    StateTransition { from: String, to: String },
    /// Configuration changed
    ConfigChanged,
    /// Manual command received
    ManualCommand { command: String },
    /// Automated repair triggered
    AutoRepair { target: String },
    /// Automated update triggered
    AutoUpdate { package_count: u32 },
}

/// Sentinel response action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentinelAction {
    /// No action needed
    None,
    /// Restart failed service
    RestartService { service: String },
    /// Sync package databases
    SyncDatabases,
    /// Perform system update
    SystemUpdate { dry_run: bool },
    /// Run repair playbook
    RunRepair { probe: String },
    /// Log warning
    LogWarning { message: String },
    /// Send notification
    SendNotification { title: String, body: String },
}

/// Sentinel metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelMetrics {
    /// Sentinel uptime (seconds)
    pub uptime_seconds: u64,
    /// Total events processed
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    /// Automated actions taken
    pub automated_actions: u64,
    /// Manual commands received
    pub manual_commands: u64,
    /// Health checks performed
    pub health_checks: u64,
    /// Update scans performed
    pub update_scans: u64,
    /// Audits performed
    pub audits: u64,
    /// Current health status
    pub current_health: String,
    /// Error rate (errors per hour)
    pub error_rate: f64,
    /// System drift index
    pub drift_index: f64,
    /// Last state transition
    pub last_transition: Option<StateTransition>,
}

/// State transition record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// From state
    pub from: String,
    /// To state
    pub to: String,
    /// Reason
    pub reason: String,
}

/// Response playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePlaybook {
    /// Playbook name
    pub name: String,
    /// Event triggers
    pub triggers: Vec<SentinelEvent>,
    /// Actions to perform
    pub actions: Vec<SentinelAction>,
    /// Require confirmation?
    pub requires_confirmation: bool,
}

/// Sentinel log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelLogEntry {
    /// Timestamp (ISO 8601)
    pub ts: DateTime<Utc>,
    /// Event type
    pub event: String,
    /// Action taken
    pub action: String,
    /// Success status
    pub success: bool,
    /// Details
    pub details: String,
    /// Arch Wiki citation
    pub citation: String,
}

impl SentinelLogEntry {
    /// Create new log entry
    pub fn new(
        event: String,
        action: String,
        success: bool,
        details: String,
        citation: String,
    ) -> Self {
        Self {
            ts: Utc::now(),
            event,
            action,
            success,
            details,
            citation,
        }
    }
}
