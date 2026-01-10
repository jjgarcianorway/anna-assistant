//! SessionStore - Persistent storage for sessions.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::patterns::CrossSessionPatterns;
use super::types::Session;
use crate::config::anna_data_dir;

/// Persistent storage for sessions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStore {
    /// Sessions by ID
    pub sessions: HashMap<String, Session>,
    /// Last save timestamp
    pub last_saved: Option<String>,
    /// Cross-session patterns (v0.0.889)
    #[serde(default)]
    pub patterns: CrossSessionPatterns,
}

impl SessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            last_saved: None,
            patterns: CrossSessionPatterns::default(),
        }
    }

    /// Load sessions from disk
    pub fn load() -> Result<Self> {
        let path = sessions_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let mut store: SessionStore = serde_json::from_str(&content)?;
            store.cleanup_old_sessions();
            Ok(store)
        } else {
            Ok(Self::new())
        }
    }

    /// Save sessions to disk
    pub fn save(&mut self) -> Result<()> {
        let path = sessions_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.last_saved = Some(chrono::Utc::now().to_rfc3339());
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get or create a session
    pub fn get_or_create(&mut self, session_id: &str) -> &mut Session {
        self.sessions
            .entry(session_id.to_string())
            .or_insert_with(|| {
                let mut session = Session::new();
                session.id = session_id.to_string();
                session
            })
    }

    /// Remove sessions older than 24 hours
    pub fn cleanup_old_sessions(&mut self) {
        let now = chrono::Utc::now();
        self.sessions.retain(|_, session| {
            if let Ok(last_activity) =
                chrono::DateTime::parse_from_rfc3339(&session.last_activity)
            {
                let duration = now.signed_duration_since(last_activity);
                duration.num_hours() < 24
            } else {
                false
            }
        });
    }

    /// Get number of active sessions
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

/// Get sessions file path
pub fn sessions_path() -> PathBuf {
    anna_data_dir().join("sessions.json")
}
