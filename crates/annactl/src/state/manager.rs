//! State Manager - Manage application state
//!
//! Beta.200: Core state management logic
//!
//! Responsibilities:
//! - Manage conversation history
//! - Track user preferences (language, etc.)
//! - Persist state to disk
//! - Provide clean state access

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred language (default: "en")
    pub language: String,

    /// Show startup summary
    pub show_startup_summary: bool,

    /// Enable telemetry caching
    pub cache_telemetry: bool,

    /// Telemetry cache TTL in seconds
    pub cache_ttl: u64,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            show_startup_summary: true,
            cache_telemetry: true,
            cache_ttl: 5,
        }
    }
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message role (user or assistant)
    pub role: String,

    /// Message content
    pub content: String,

    /// Timestamp
    pub timestamp: i64,
}

/// Application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// User preferences
    pub preferences: UserPreferences,

    /// Conversation history (for TUI)
    pub conversation_history: Vec<ConversationMessage>,

    /// Last query timestamp
    pub last_query_time: Option<i64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            preferences: UserPreferences::default(),
            conversation_history: Vec::new(),
            last_query_time: None,
        }
    }
}

/// State manager for application state
pub struct StateManager {
    /// Current state
    state: AppState,

    /// State file path
    state_file: PathBuf,
}

impl StateManager {
    /// Create a new state manager
    ///
    /// Beta.200: Loads state from ~/.config/anna/state.json
    pub fn new() -> Result<Self> {
        let state_file = Self::get_state_file_path()?;

        // Try to load existing state
        let state = if state_file.exists() {
            Self::load_from_file(&state_file)?
        } else {
            AppState::default()
        };

        Ok(Self { state, state_file })
    }

    /// Get state file path
    fn get_state_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        let anna_dir = config_dir.join("anna");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&anna_dir)?;

        Ok(anna_dir.join("state.json"))
    }

    /// Load state from file
    fn load_from_file(path: &PathBuf) -> Result<AppState> {
        let contents = std::fs::read_to_string(path)?;
        let state: AppState = serde_json::from_str(&contents)?;
        Ok(state)
    }

    /// Save state to file
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.state)?;
        std::fs::write(&self.state_file, json)?;
        Ok(())
    }

    /// Get user preferences
    pub fn get_preferences(&self) -> &UserPreferences {
        &self.state.preferences
    }

    /// Update user preferences
    pub fn update_preferences(&mut self, preferences: UserPreferences) -> Result<()> {
        self.state.preferences = preferences;
        self.save()
    }

    /// Add message to conversation history
    pub fn add_message(&mut self, role: String, content: String) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();

        let message = ConversationMessage {
            role,
            content,
            timestamp,
        };

        self.state.conversation_history.push(message);
        self.state.last_query_time = Some(timestamp);

        self.save()
    }

    /// Get conversation history
    pub fn get_conversation_history(&self) -> &[ConversationMessage] {
        &self.state.conversation_history
    }

    /// Clear conversation history
    pub fn clear_conversation_history(&mut self) -> Result<()> {
        self.state.conversation_history.clear();
        self.save()
    }

    /// Get last query time
    pub fn get_last_query_time(&self) -> Option<i64> {
        self.state.last_query_time
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            state: AppState::default(),
            state_file: PathBuf::from("/tmp/anna_state.json"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_preferences() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.language, "en");
        assert!(prefs.show_startup_summary);
        assert!(prefs.cache_telemetry);
        assert_eq!(prefs.cache_ttl, 5);
    }

    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert_eq!(state.preferences.language, "en");
        assert!(state.conversation_history.is_empty());
        assert!(state.last_query_time.is_none());
    }

    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_add_message() {
        let mut manager = StateManager::default();
        let result = manager.add_message("user".to_string(), "test message".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.get_conversation_history().len(), 1);
    }

    #[test]
    fn test_clear_history() {
        let mut manager = StateManager::default();
        manager.add_message("user".to_string(), "test".to_string()).ok();
        manager.clear_conversation_history().ok();
        assert_eq!(manager.get_conversation_history().len(), 0);
    }
}
