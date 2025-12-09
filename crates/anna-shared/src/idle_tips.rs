//! Idle-time learning tips (v0.0.240).
//!
//! When the user is idle in the REPL, Anna can surface helpful tips
//! about the system - things she noticed while monitoring.
//!
//! v0.0.240: Initial implementation.

use crate::user_profile::UserProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Idle tip categories - what kind of insight is this?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TipCategory {
    /// Performance insight (high CPU, memory, etc.)
    Performance,
    /// Security observation
    Security,
    /// Disk/storage related
    Storage,
    /// Service health
    Services,
    /// Network related
    Network,
    /// General system knowledge
    System,
    /// User habit patterns
    Patterns,
}

/// A single tip that can be shown during idle time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleTip {
    /// Unique identifier for deduplication
    pub id: String,
    /// Category of tip
    pub category: TipCategory,
    /// The tip text (Anna's voice)
    pub message: String,
    /// Optional actionable suggestion
    pub action_hint: Option<String>,
    /// When this tip was generated
    pub generated_at: DateTime<Utc>,
    /// Priority (higher = show first)
    pub priority: u8,
}

impl IdleTip {
    pub fn new(id: impl Into<String>, category: TipCategory, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            category,
            message: message.into(),
            action_hint: None,
            generated_at: Utc::now(),
            priority: 50,
        }
    }

    pub fn with_action(mut self, hint: impl Into<String>) -> Self {
        self.action_hint = Some(hint.into());
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Tracks which tips have been shown to avoid repetition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TipHistory {
    /// IDs of tips that have been shown
    pub shown_ids: HashSet<String>,
    /// Last time any tip was shown
    pub last_shown: Option<DateTime<Utc>>,
    /// Total tips shown this session
    pub session_count: u32,
}

impl TipHistory {
    /// Load tip history from disk
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save tip history to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = Self::path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    fn path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("anna").join("tip_history.json"))
    }

    /// Mark a tip as shown
    pub fn mark_shown(&mut self, tip_id: &str) {
        self.shown_ids.insert(tip_id.to_string());
        self.last_shown = Some(Utc::now());
        self.session_count += 1;
    }

    /// Check if a tip has been shown
    pub fn was_shown(&self, tip_id: &str) -> bool {
        self.shown_ids.contains(tip_id)
    }

    /// Reset for new day (partial reset - keep some memory)
    pub fn reset_daily(&mut self) {
        // Keep IDs from recent tips but reset session count
        self.session_count = 0;
    }
}

/// Queue of pending tips to show
#[derive(Debug, Default)]
pub struct TipQueue {
    tips: Vec<IdleTip>,
    history: TipHistory,
}

impl TipQueue {
    pub fn new() -> Self {
        Self {
            tips: Vec::new(),
            history: TipHistory::load(),
        }
    }

    /// Add a tip to the queue (if not already shown)
    pub fn push(&mut self, tip: IdleTip) {
        if !self.history.was_shown(&tip.id) {
            self.tips.push(tip);
            // Sort by priority (highest first)
            self.tips.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
    }

    /// Get the next tip to show (highest priority first)
    pub fn pop(&mut self) -> Option<IdleTip> {
        if self.tips.is_empty() {
            return None;
        }
        // Remove from front (highest priority after sort)
        let tip = self.tips.remove(0);
        self.history.mark_shown(&tip.id);
        let _ = self.history.save();
        Some(tip)
    }

    /// Check if there are tips waiting
    pub fn has_tips(&self) -> bool {
        !self.tips.is_empty()
    }

    /// How many tips are pending
    pub fn len(&self) -> usize {
        self.tips.len()
    }

    /// Get count shown this session
    pub fn shown_count(&self) -> u32 {
        self.history.session_count
    }
}

/// Get idle tips based on current system state
/// This would normally query the daemon, but for now returns contextual tips
pub fn get_contextual_tips() -> Vec<IdleTip> {
    let profile = UserProfile::load();
    let mut tips = Vec::new();

    // Learning mode tips
    if !profile.preferences.learning_mode {
        tips.push(IdleTip::new(
            "tip-learning-mode",
            TipCategory::System,
            "By the way, if you want me to explain why commands work, \
             just say \"enable learning mode\".",
        ));
    }

    // Auto-confirm tips
    if !profile.preferences.auto_confirm_low_risk {
        tips.push(IdleTip::new(
            "tip-auto-confirm",
            TipCategory::System,
            "I ask for confirmation on every change. If you'd like me to \
             auto-apply low-risk fixes, say \"enable auto-confirm\".",
        ).with_priority(30));
    }

    // Email tip if not set
    if profile.email.is_none() {
        tips.push(IdleTip::new(
            "tip-email-setup",
            TipCategory::System,
            "For long-running requests, I can email you when I'm done. \
             Just tell me your email address.",
        ).with_priority(40));
    }

    // Internal comms tip
    if !profile.preferences.show_internal_comms {
        tips.push(IdleTip::new(
            "tip-internal-comms",
            TipCategory::System,
            "Want to see the IT department discuss your requests? \
             Say \"show internal comms\" for the fly-on-wall experience.",
        ).with_priority(35));
    }

    tips
}

/// Format a tip for display in the REPL
pub fn format_tip(tip: &IdleTip, colors: &TipColors) -> String {
    let mut output = String::new();

    // v0.0.265: Use ASCII symbols instead of emojis
    let icon = match tip.category {
        TipCategory::Performance => "*",
        TipCategory::Security => "#",
        TipCategory::Storage => "@",
        TipCategory::Services => "+",
        TipCategory::Network => "~",
        TipCategory::System => "i",
        TipCategory::Patterns => "%",
    };

    output.push_str(&format!(
        "\n{}[tip]{} {} {}\n",
        colors.dim, colors.reset, icon, tip.message
    ));

    if let Some(ref hint) = tip.action_hint {
        output.push_str(&format!(
            "  {}→ {}{}\n",
            colors.dim, hint, colors.reset
        ));
    }

    output
}

/// Colors for tip formatting (passed from annactl)
pub struct TipColors {
    pub dim: &'static str,
    pub reset: &'static str,
}

impl Default for TipColors {
    fn default() -> Self {
        Self {
            dim: "\x1b[2m",
            reset: "\x1b[0m",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tip_creation() {
        let tip = IdleTip::new("test-tip", TipCategory::System, "Test message")
            .with_action("Do something")
            .with_priority(80);

        assert_eq!(tip.id, "test-tip");
        assert_eq!(tip.priority, 80);
        assert!(tip.action_hint.is_some());
    }

    #[test]
    fn test_tip_queue() {
        let mut queue = TipQueue::new();
        queue.tips.clear(); // Clear any loaded state
        queue.history = TipHistory::default(); // Fresh history for test

        queue.push(IdleTip::new("tip1", TipCategory::System, "First").with_priority(50));
        queue.push(IdleTip::new("tip2", TipCategory::System, "Second").with_priority(80));

        // After push, tips are sorted by priority descending
        // So order in vec is: [tip2 (80), tip1 (50)]
        // pop() removes from front, so highest priority comes first
        assert_eq!(queue.len(), 2);
        let next = queue.pop();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, "tip2"); // Higher priority first
    }

    #[test]
    fn test_history_dedup() {
        let mut history = TipHistory::default();
        history.mark_shown("tip1");

        assert!(history.was_shown("tip1"));
        assert!(!history.was_shown("tip2"));
    }
}
