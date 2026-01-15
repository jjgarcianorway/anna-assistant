//! Session and cache persistence methods for StateInner.

use anna_shared::session::SessionStore;
use anna_shared::status::DaemonState;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};

use super::types::{CachedAnswer, StateInner, UpdateState};
use super::ANSWER_CACHE_TTL_SECS;

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
            recovery_status: anna_shared::status::RecoveryStatus::default(),
        }
    }

    /// Normalize a question for cache lookup (lowercase, trim, remove punctuation)
    pub(crate) fn normalize_question(question: &str) -> String {
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

    /// v0.3.28: Clear all in-memory state for reset command.
    /// This ensures consistency between daemon state and files after reset.
    pub fn clear_for_reset(&mut self) {
        self.answer_cache.clear();
        self.sessions = anna_shared::session::SessionStore::new();
        self.session_save_counter = 0;
        // Note: We don't reset uptime/started_at as those track daemon lifetime, not data
        debug!("StateInner cleared for reset");
    }

    /// Get or create a session for a client
    pub fn get_or_create_session(&mut self, client_id: &str) -> &mut anna_shared::session::Session {
        self.sessions.get_or_create(client_id)
    }

    /// Cleanup sessions and optionally persist to disk
    /// Saves every 5 interactions to avoid excessive disk writes
    /// v0.0.896: Also mines patterns for recurring issue detection
    pub fn cleanup_sessions(&mut self) {
        self.sessions.cleanup_old_sessions();

        // Increment counter and save periodically
        self.session_save_counter += 1;
        if self.session_save_counter >= 5 {
            self.session_save_counter = 0;

            // v0.0.896: Mine patterns from session history for better suggestions
            self.sessions.mine_patterns();

            if let Err(e) = self.sessions.save() {
                warn!("Failed to persist sessions: {}", e);
            }
        }
    }

    /// v0.0.896: Check if a question matches a recurring issue pattern
    pub fn check_recurring_issue(&self, question: &str) -> Option<String> {
        self.sessions.is_recurring_issue(question)
            .map(|issue| format!(
                "This looks like a recurring issue: {} ({} occurrences, last seen: {})",
                issue.description, issue.occurrences, issue.last_seen
            ))
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
}

impl Default for StateInner {
    fn default() -> Self {
        Self::new()
    }
}
