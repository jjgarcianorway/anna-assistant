//! Daemon state management.

use anna_shared::status::{DaemonState, UpdateCheckState};
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Shared daemon state handle
#[derive(Clone)]
pub struct SharedState {
    inner: Arc<RwLock<StateInner>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StateInner::new())),
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, StateInner> {
        self.inner.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, StateInner> {
        self.inner.write().await
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

/// Inner state
pub struct StateInner {
    pub state: DaemonState,
    pub started_at: Instant,
    pub ollama_running: bool,
    pub model: Option<String>,
    pub last_error: Option<String>,
    pub update: UpdateState,
    pub gpu: Option<String>,
    pub vram_mb: Option<u64>,
}

impl StateInner {
    pub fn new() -> Self {
        Self {
            state: DaemonState::Starting,
            started_at: Instant::now(),
            ollama_running: false,
            model: None,
            last_error: None,
            update: UpdateState::default(),
            gpu: None,
            vram_mb: None,
        }
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    pub fn to_status(&self) -> anna_shared::status::DaemonStatus {
        anna_shared::status::DaemonStatus {
            state: self.state,
            version: VERSION.to_string(),
            ollama_running: self.ollama_running,
            model: self.model.clone(),
            uptime_secs: self.uptime_secs(),
            gpu: self.gpu.clone(),
            vram_mb: self.vram_mb,
        }
    }
}

impl Default for StateInner {
    fn default() -> Self {
        Self::new()
    }
}

/// Update state
pub struct UpdateState {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub last_check_at: Option<DateTime<Utc>>,
    pub next_check_at: Option<DateTime<Utc>>,
    pub latest_version: Option<String>,
    pub latest_checked_at: Option<DateTime<Utc>>,
    pub update_available: bool,
    pub check_state: UpdateCheckState,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: DEFAULT_UPDATE_CHECK_INTERVAL,
            last_check_at: None,
            next_check_at: None,
            latest_version: None,
            latest_checked_at: None,
            update_available: false,
            check_state: UpdateCheckState::NeverChecked,
        }
    }
}
