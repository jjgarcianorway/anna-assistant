//! Daemon state management.

use std::sync::Arc;
use std::time::Instant;

use anna_shared::ledger::Ledger;
use anna_shared::status::{DaemonState, DaemonStatus, HardwareInfo, ModelInfo, OllamaStatus};
use anna_shared::VERSION;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// Shared daemon state
pub struct DaemonState_ {
    pub state: DaemonState,
    pub started_at: Instant,
    pub last_update_check: Option<DateTime<Utc>>,
    pub next_update_check: Option<DateTime<Utc>>,
    pub ollama: OllamaStatus,
    pub model: Option<ModelInfo>,
    pub hardware: HardwareInfo,
    pub ledger: Ledger,
    #[allow(dead_code)] // Reserved for future error reporting
    pub last_error: Option<String>,
}

impl DaemonState_ {
    pub fn new() -> Self {
        Self {
            state: DaemonState::Starting,
            started_at: Instant::now(),
            last_update_check: None,
            next_update_check: None,
            ollama: OllamaStatus::default(),
            model: None,
            hardware: HardwareInfo::default(),
            ledger: Ledger::new(),
            last_error: None,
        }
    }

    pub fn to_status(&self) -> DaemonStatus {
        DaemonStatus {
            version: VERSION.to_string(),
            state: self.state.clone(),
            uptime_seconds: self.started_at.elapsed().as_secs(),
            last_update_check: self.last_update_check,
            next_update_check: self.next_update_check,
            ollama: self.ollama.clone(),
            model: self.model.clone(),
            hardware: self.hardware.clone(),
            ledger: self.ledger.summary(),
        }
    }
}

/// Thread-safe shared state handle
pub type SharedState = Arc<RwLock<DaemonState_>>;

pub fn create_shared_state() -> SharedState {
    Arc::new(RwLock::new(DaemonState_::new()))
}
