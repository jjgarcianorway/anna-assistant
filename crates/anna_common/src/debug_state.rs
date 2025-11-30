//! Debug State Module for v0.89.0
//!
//! Persistent debug mode toggle controlled via natural language.
//! No LLM required - handled entirely by Brain fast path.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Debug state storage directory
/// v0.91.0: Changed to user-local ~/.config/anna for CLI writability
pub const DEBUG_STATE_DIR: &str = ".config/anna";

/// Debug state file
pub const DEBUG_STATE_FILE: &str = "debug_state.json";

// ============================================================================
// Debug State Model
// ============================================================================

/// Persistent debug mode state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct DebugState {
    /// Whether debug mode is enabled
    pub enabled: bool,
    /// When the state was last changed
    pub last_changed_at: Option<DateTime<Utc>>,
    /// Reason for the last change (e.g., "user_command")
    pub last_changed_reason: Option<String>,
}


impl DebugState {
    /// Create a new debug state
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the path to the debug state file
    /// Uses $HOME/.config/anna/debug_state.json for user writability
    fn file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(DEBUG_STATE_DIR).join(DEBUG_STATE_FILE)
    }

    /// Load debug state from disk
    pub fn load() -> Self {
        let path = Self::file_path();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save debug state to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::file_path();
        // Ensure directory exists
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    /// Enable debug mode
    pub fn enable(&mut self, reason: &str) -> std::io::Result<()> {
        self.enabled = true;
        self.last_changed_at = Some(Utc::now());
        self.last_changed_reason = Some(reason.to_string());
        self.save()
    }

    /// Disable debug mode
    pub fn disable(&mut self, reason: &str) -> std::io::Result<()> {
        self.enabled = false;
        self.last_changed_at = Some(Utc::now());
        self.last_changed_reason = Some(reason.to_string());
        self.save()
    }

    /// Format status string for display
    pub fn format_status(&self) -> String {
        if self.enabled {
            "Debug mode is currently enabled.".to_string()
        } else {
            "Debug mode is currently disabled.".to_string()
        }
    }

    /// Format enable confirmation
    pub fn format_enable_message() -> String {
        "Debug mode is now enabled for this machine.".to_string()
    }

    /// Format disable confirmation
    pub fn format_disable_message() -> String {
        "Debug mode is now disabled for this machine.".to_string()
    }
}

// ============================================================================
// Global Accessors (for convenience)
// ============================================================================

/// Check if debug mode is enabled (reads from disk each time)
pub fn debug_is_enabled() -> bool {
    DebugState::load().enabled
}

/// Set debug mode enabled state
pub fn debug_set_enabled(enabled: bool, reason: &str) -> std::io::Result<()> {
    let mut state = DebugState::load();
    if enabled {
        state.enable(reason)
    } else {
        state.disable(reason)
    }
}

/// Get current debug state (for status display)
pub fn debug_get_state() -> DebugState {
    DebugState::load()
}

// ============================================================================
// Debug Intent Classification
// ============================================================================

/// Debug-related intent from user question
#[derive(Debug, Clone, PartialEq)]
pub enum DebugIntent {
    /// User wants to enable debug mode
    Enable,
    /// User wants to disable debug mode
    Disable,
    /// User wants to check debug mode status
    Status,
    /// Not a debug-related question
    None,
}

impl DebugIntent {
    /// Classify a question for debug intent
    pub fn classify(question: &str) -> Self {
        let q = question.to_lowercase();
        let q = q.trim();

        // Check for debug-related keywords
        let has_debug = q.contains("debug");
        let _has_mode = q.contains("mode") || has_debug; // "debug" alone is enough

        if has_debug {
            // IMPORTANT: Check Disable FIRST because:
            // - "deactivate" contains "activate"
            // - "disable" contains no enable substring, but safer to check first
            if q.contains("disable") || (q.contains("turn") && q.contains("off"))
                || q.contains("deactivate") || q.contains("stop debug")
            {
                return Self::Disable;
            }

            // Status patterns - check BEFORE enable because "enabled" contains "enable"
            if q.contains("enabled?") || q.contains("enabled") && q.contains("?")
                || q.contains("on?") || q.contains("status")
                || q.contains("state") || q.starts_with("is ")
                || q.contains("are you in debug") || q.contains("what is")
            {
                return Self::Status;
            }

            // Enable patterns - check last
            if q.contains("enable") || (q.contains("turn") && q.contains("on"))
                || q.contains("activate") || q.contains("start debug")
            {
                return Self::Enable;
            }
        }

        Self::None
    }

    /// Check if this is a debug-related intent
    pub fn is_debug_intent(&self) -> bool {
        *self != Self::None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_state_default() {
        let state = DebugState::default();
        assert!(!state.enabled);
        assert!(state.last_changed_at.is_none());
        assert!(state.last_changed_reason.is_none());
    }

    #[test]
    fn test_debug_intent_enable() {
        assert_eq!(DebugIntent::classify("enable debug mode"), DebugIntent::Enable);
        assert_eq!(DebugIntent::classify("turn debug mode on"), DebugIntent::Enable);
        assert_eq!(DebugIntent::classify("activate debug"), DebugIntent::Enable);
        assert_eq!(DebugIntent::classify("Anna, start debug mode"), DebugIntent::Enable);
        assert_eq!(DebugIntent::classify("turn on debug"), DebugIntent::Enable);
    }

    #[test]
    fn test_debug_intent_disable() {
        assert_eq!(DebugIntent::classify("disable debug mode"), DebugIntent::Disable);
        assert_eq!(DebugIntent::classify("turn debug mode off"), DebugIntent::Disable);
        assert_eq!(DebugIntent::classify("deactivate debug"), DebugIntent::Disable);
        assert_eq!(DebugIntent::classify("stop debug mode"), DebugIntent::Disable);
        assert_eq!(DebugIntent::classify("turn off debug"), DebugIntent::Disable);
    }

    #[test]
    fn test_debug_intent_status() {
        assert_eq!(DebugIntent::classify("is debug mode enabled?"), DebugIntent::Status);
        assert_eq!(DebugIntent::classify("is debug on?"), DebugIntent::Status);
        assert_eq!(DebugIntent::classify("what is your debug mode state?"), DebugIntent::Status);
        assert_eq!(DebugIntent::classify("are you in debug mode?"), DebugIntent::Status);
        assert_eq!(DebugIntent::classify("debug status"), DebugIntent::Status);
    }

    #[test]
    fn test_debug_intent_none() {
        assert_eq!(DebugIntent::classify("how much RAM do I have?"), DebugIntent::None);
        assert_eq!(DebugIntent::classify("what is the weather?"), DebugIntent::None);
        assert_eq!(DebugIntent::classify("tell me about debugging"), DebugIntent::None);
    }

    #[test]
    fn test_format_messages() {
        assert!(DebugState::format_enable_message().contains("enabled"));
        assert!(DebugState::format_disable_message().contains("disabled"));

        let mut state = DebugState::default();
        assert!(state.format_status().contains("disabled"));
        state.enabled = true;
        assert!(state.format_status().contains("enabled"));
    }
}
