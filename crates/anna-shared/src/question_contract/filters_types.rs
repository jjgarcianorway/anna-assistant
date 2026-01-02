//! Type definitions for help text leakage filters.

use serde::{Deserialize, Serialize};

/// Type of content leakage detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeakageType {
    /// Tutorial-style instructions ("you can...", "try...").
    Tutorial,
    /// Debug steps ("to diagnose...", "check the logs...").
    DebugSteps,
    /// Command suggestions ("run `command`", code blocks).
    Commands,
    /// Unsolicited suggestions ("you should...", "consider...").
    Suggestions,
    /// Extra context not asked for.
    ExtraContext,
    /// Off-topic information.
    OffTopic,
}

impl LeakageType {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Tutorial => "tutorial",
            Self::DebugSteps => "debug_steps",
            Self::Commands => "commands",
            Self::Suggestions => "suggestions",
            Self::ExtraContext => "extra_context",
            Self::OffTopic => "off_topic",
        }
    }
}

/// Result of filtering an answer.
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// The filtered text.
    pub filtered_text: String,
    /// Leakage types found.
    pub leakages: Vec<DetectedLeakage>,
    /// Whether filtering was applied.
    pub was_filtered: bool,
}

impl FilterResult {
    /// Create result with no filtering needed.
    pub fn clean(text: String) -> Self {
        Self {
            filtered_text: text,
            leakages: Vec::new(),
            was_filtered: false,
        }
    }

    /// Create result with filtering applied.
    pub fn filtered(text: String, leakages: Vec<DetectedLeakage>) -> Self {
        Self {
            filtered_text: text,
            leakages,
            was_filtered: true,
        }
    }

    /// Check if any leakage was detected.
    pub fn has_leakage(&self) -> bool {
        !self.leakages.is_empty()
    }
}

/// A detected leakage instance.
#[derive(Debug, Clone)]
pub struct DetectedLeakage {
    /// Type of leakage.
    pub leakage_type: LeakageType,
    /// The text that was removed.
    pub removed_text: String,
    /// Pattern that matched.
    pub pattern: String,
}
