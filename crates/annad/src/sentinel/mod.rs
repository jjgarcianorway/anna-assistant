//! Sentinel subsystem for continuous system governance
//!
//! Phase 1.0: Autonomous persistent daemon framework
//! Citation: [archwiki:System_maintenance]
//!
//! The Sentinel is Anna's autonomous core - a persistent daemon that continuously
//! monitors system health, responds to events, and maintains system integrity without
//! user intervention.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                 Sentinel Daemon                          │
//! │                                                          │
//! │  ┌──────────────┐         ┌──────────────┐             │
//! │  │  Schedulers  │────────►│  Event Bus   │             │
//! │  │              │         │              │             │
//! │  │ • Health     │         │  Playbooks   │             │
//! │  │ • Updates    │         │  Handlers    │             │
//! │  │ • Audits     │         │              │             │
//! │  └──────────────┘         └──────┬───────┘             │
//! │                                  │                      │
//! │  ┌──────────────┐                │                      │
//! │  │    State     │◄───────────────┘                      │
//! │  │  Manager     │                                       │
//! │  │              │                                       │
//! │  │ /var/lib/    │                                       │
//! │  │  anna/       │                                       │
//! │  └──────────────┘                                       │
//! │                                                          │
//! │  ┌──────────────────────────────────────┐              │
//! │  │         Module Integration           │              │
//! │  │  • Health   • Repair                 │              │
//! │  │  • Steward  • Recovery               │              │
//! │  │  • Install  • Audit                  │              │
//! │  └──────────────────────────────────────┘              │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Periodic Monitoring**: Automated health checks, update scans, and security audits
//! - **Event-Driven Response**: React to service failures, log anomalies, and package drift
//! - **State Persistence**: Maintains system state snapshot in `/var/lib/anna/state.json`
//! - **Response Playbooks**: Configurable automated responses to system events
//! - **Adaptive Scheduling**: Adjust check frequencies based on system stability
//! - **Ethics Layer**: Prevents destructive operations on user data
//!
//! # Safety Guarantees
//!
//! - All automated actions require explicit configuration
//! - Dry-run mode for all mutation operations
//! - Append-only audit logging with integrity verification
//! - Never modifies `/home` or `/data` directories
//! - Watchdog integration for daemon resilience

pub mod daemon;
pub mod events;
pub mod state;
pub mod types;

pub use daemon::{SentinelDaemon, write_log_entry};
pub use events::{create_default_playbooks, EventBus};
pub use state::{calculate_diff, load_config, load_state, save_config, save_state, StateDiff};
pub use types::{
    HealthSnapshot, ResponsePlaybook, SentinelAction, SentinelConfig, SentinelEvent,
    SentinelLogEntry, SentinelMetrics, SentinelState, StateTransition,
};

use anyhow::Result;

/// Initialize sentinel subsystem
pub async fn initialize() -> Result<SentinelDaemon> {
    SentinelDaemon::new().await
}

/// Check if sentinel is enabled in configuration
pub async fn is_enabled() -> bool {
    match load_config().await {
        Ok(config) => config.autonomous_mode,
        Err(_) => false,
    }
}
