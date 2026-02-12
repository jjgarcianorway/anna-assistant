//! Suggestion data types.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

const SUGGESTIONS_FILE: &str = "/var/lib/anna/suggestions.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionPriority {
    Low,      // Nice to have
    Medium,   // Should consider
    High,     // Important
    Critical, // Needs attention
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: String,
    pub priority: SuggestionPriority,
    pub title: String,
    pub description: String,
    pub reasoning: String,
    pub action: Option<String>, // Optional action user can take
    pub created_at: String,
    pub shown_count: u32,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionsState {
    pub suggestions: Vec<Suggestion>,
    pub last_scan: String,
}

impl Default for SuggestionsState {
    fn default() -> Self {
        Self {
            suggestions: Vec::new(),
            last_scan: Utc::now().to_rfc3339(),
        }
    }
}

impl SuggestionsState {
    /// Load suggestions from disk
    pub fn load() -> Self {
        let path = PathBuf::from(SUGGESTIONS_FILE);
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save suggestions to disk
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(SUGGESTIONS_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add a new suggestion if it doesn't already exist
    pub fn add(&mut self, suggestion: Suggestion) {
        // Don't add if already exists
        if !self.suggestions.iter().any(|s| s.id == suggestion.id) {
            info!("New suggestion: {}", suggestion.title);
            self.suggestions.push(suggestion);
        }
    }

    /// Get active (non-dismissed) suggestions
    pub fn active_suggestions(&self) -> Vec<&Suggestion> {
        self.suggestions
            .iter()
            .filter(|s| !s.dismissed)
            .collect()
    }

    /// Mark suggestion as shown
    pub fn mark_shown(&mut self, id: &str) {
        if let Some(s) = self.suggestions.iter_mut().find(|s| s.id == id) {
            s.shown_count += 1;
        }
    }

    /// Dismiss a suggestion
    pub fn dismiss(&mut self, id: &str) {
        if let Some(s) = self.suggestions.iter_mut().find(|s| s.id == id) {
            s.dismissed = true;
        }
    }
}

/// Format suggestions for user display
pub fn format_suggestions(suggestions: &[&Suggestion]) -> String {
    if suggestions.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("\n💡 Proactive Suggestions:\n");

    for (i, suggestion) in suggestions.iter().enumerate() {
        let priority_icon = match suggestion.priority {
            SuggestionPriority::Critical => "🔴",
            SuggestionPriority::High => "🟠",
            SuggestionPriority::Medium => "🟡",
            SuggestionPriority::Low => "🔵",
        };

        output.push_str(&format!("\n{} {}. {}\n", priority_icon, i + 1, suggestion.title));
        output.push_str(&format!("   {}\n", suggestion.description));

        if let Some(action) = &suggestion.action {
            output.push_str(&format!("   → {}\n", action));
        }
    }

    output.push('\n');
    output
}
