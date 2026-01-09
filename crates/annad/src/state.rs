//! Daemon state management.

use anna_shared::session::Session;
use anna_shared::status::{DaemonState, UpdateCheckState};
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
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

    /// Wait for active connections to drain before restart
    /// Returns true if drained, false if timeout
    pub async fn wait_for_connections_to_drain(&self, timeout_secs: u64) -> bool {
        use tokio::time::{sleep, Duration};

        // Signal that restart is pending
        {
            let mut state = self.write().await;
            state.restart_pending = true;
        }

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let active = {
                let state = self.read().await;
                state.active_connections
            };

            if active == 0 {
                tracing::info!("All connections drained, safe to restart");
                return true;
            }

            if start.elapsed() > timeout {
                tracing::warn!("Timeout waiting for {} connections to drain, restarting anyway", active);
                return false;
            }

            tracing::info!("Waiting for {} active connection(s) to finish before restart...", active);
            sleep(Duration::from_secs(2)).await;
        }
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
    /// Active sessions by client ID
    pub sessions: HashMap<String, Session>,
    /// Number of active connections (for graceful shutdown)
    pub active_connections: u32,
    /// Flag indicating restart is pending (clients should finish quickly)
    pub restart_pending: bool,
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
            sessions: HashMap::new(),
            active_connections: 0,
            restart_pending: false,
        }
    }

    /// Get or create a session for a client
    pub fn get_or_create_session(&mut self, client_id: &str) -> &mut Session {
        self.sessions.entry(client_id.to_string()).or_insert_with(Session::new)
    }

    /// Cleanup sessions older than 1 hour
    pub fn cleanup_sessions(&mut self) {
        let now = chrono::Utc::now();
        self.sessions.retain(|_, session| {
            if let Ok(last_activity) = chrono::DateTime::parse_from_rfc3339(&session.last_activity) {
                let duration = now.signed_duration_since(last_activity);
                duration.num_hours() < 1
            } else {
                true // Keep if we can't parse the timestamp
            }
        });
    }

    /// Increment active connection count
    pub fn connection_started(&mut self) {
        self.active_connections = self.active_connections.saturating_add(1);
    }

    /// Decrement active connection count
    pub fn connection_ended(&mut self) {
        self.active_connections = self.active_connections.saturating_sub(1);
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
