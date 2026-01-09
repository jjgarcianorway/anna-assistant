//! Daemon state management.

use anna_shared::session::{Session, SessionStore};
use anna_shared::status::{DaemonState, UpdateCheckState};
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache expiration time in seconds (5 minutes)
const ANSWER_CACHE_TTL_SECS: u64 = 300;

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
    /// Persistent session storage
    pub sessions: SessionStore,
    /// Number of active connections (for graceful shutdown)
    pub active_connections: u32,
    /// Flag indicating restart is pending (clients should finish quickly)
    pub restart_pending: bool,
    /// Counter for periodic session saves
    session_save_counter: u32,
    /// Answer cache for identical questions (normalized question -> (answer, timestamp))
    answer_cache: HashMap<String, CachedAnswer>,
}

/// A cached answer with timestamp
#[derive(Clone)]
pub struct CachedAnswer {
    pub answer: String,
    pub cached_at: Instant,
}

impl StateInner {
    pub fn new() -> Self {
        // Load persisted sessions from disk
        let sessions = match SessionStore::load() {
            Ok(store) => {
                let count = store.active_count();
                if count > 0 {
                    info!("Loaded {} persisted sessions from disk", count);
                }
                store
            }
            Err(e) => {
                warn!("Failed to load persisted sessions: {}", e);
                SessionStore::new()
            }
        };

        Self {
            state: DaemonState::Starting,
            started_at: Instant::now(),
            ollama_running: false,
            model: None,
            last_error: None,
            update: UpdateState::default(),
            gpu: None,
            vram_mb: None,
            sessions,
            active_connections: 0,
            restart_pending: false,
            session_save_counter: 0,
            answer_cache: HashMap::new(),
        }
    }

    /// Normalize a question for cache lookup (lowercase, trim, remove punctuation)
    fn normalize_question(question: &str) -> String {
        question
            .to_lowercase()
            .trim()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get a cached answer if available and not expired
    pub fn get_cached_answer(&self, question: &str) -> Option<String> {
        let key = Self::normalize_question(question);
        if let Some(cached) = self.answer_cache.get(&key) {
            if cached.cached_at.elapsed().as_secs() < ANSWER_CACHE_TTL_SECS {
                debug!("Cache hit for question: {}", question);
                return Some(cached.answer.clone());
            }
        }
        None
    }

    /// Cache an answer for a question
    pub fn cache_answer(&mut self, question: &str, answer: &str) {
        let key = Self::normalize_question(question);
        self.answer_cache.insert(key, CachedAnswer {
            answer: answer.to_string(),
            cached_at: Instant::now(),
        });

        // Cleanup old entries periodically (keep max 100)
        if self.answer_cache.len() > 100 {
            self.cleanup_answer_cache();
        }
    }

    /// Remove expired cache entries
    fn cleanup_answer_cache(&mut self) {
        self.answer_cache.retain(|_, cached| {
            cached.cached_at.elapsed().as_secs() < ANSWER_CACHE_TTL_SECS
        });
    }

    /// Get or create a session for a client
    pub fn get_or_create_session(&mut self, client_id: &str) -> &mut Session {
        self.sessions.get_or_create(client_id)
    }

    /// Cleanup sessions and optionally persist to disk
    /// Saves every 5 interactions to avoid excessive disk writes
    pub fn cleanup_sessions(&mut self) {
        self.sessions.cleanup_old_sessions();

        // Increment counter and save periodically
        self.session_save_counter += 1;
        if self.session_save_counter >= 5 {
            self.session_save_counter = 0;
            if let Err(e) = self.sessions.save() {
                warn!("Failed to persist sessions: {}", e);
            }
        }
    }

    /// Force save sessions to disk (called on shutdown)
    pub fn save_sessions(&mut self) {
        if let Err(e) = self.sessions.save() {
            warn!("Failed to save sessions on shutdown: {}", e);
        } else {
            info!("Sessions persisted to disk");
        }
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
