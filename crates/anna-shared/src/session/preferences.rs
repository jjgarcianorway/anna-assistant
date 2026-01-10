//! User preferences and contradiction tracking.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::anna_data_dir;
use super::types::DetailLevel;

/// Response style preferences
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum ResponseStyle {
    /// Concise, straight to the point
    Concise,
    /// Balanced explanation
    #[default]
    Balanced,
    /// Educational with context
    Educational,
}

/// v0.0.899: Persistent user preferences across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred response detail level
    pub detail_level: DetailLevel,

    /// Whether to show command output
    pub show_command_output: bool,

    /// Whether to show thinking/reasoning steps
    pub show_reasoning: bool,

    /// Preferred response style
    pub style: ResponseStyle,

    /// Topics user is frequently interested in
    pub favorite_topics: Vec<String>,

    /// Commands user prefers (learned from feedback)
    pub preferred_commands: Vec<String>,

    /// Last updated timestamp
    pub updated_at: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            detail_level: DetailLevel::Normal,
            show_command_output: true,
            show_reasoning: false,
            style: ResponseStyle::Balanced,
            favorite_topics: Vec::new(),
            preferred_commands: Vec::new(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl UserPreferences {
    fn path() -> PathBuf {
        anna_data_dir().join("user_preferences.json")
    }

    /// Load user preferences from disk
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save preferences to disk
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)?;
        Ok(())
    }

    /// Update preferences from user feedback
    pub fn update_from_signal(&mut self, signal: &str) {
        let sig = signal.to_lowercase();
        let mut changed = false;

        // Detail level signals
        if sig.contains("be brief") || sig.contains("shorter") || sig.contains("concise") {
            self.detail_level = DetailLevel::Minimal;
            self.style = ResponseStyle::Concise;
            changed = true;
        } else if sig.contains("explain more")
            || sig.contains("more detail")
            || sig.contains("elaborate")
        {
            self.detail_level = DetailLevel::Verbose;
            self.style = ResponseStyle::Educational;
            changed = true;
        }

        // Show/hide preferences
        if sig.contains("hide output") || sig.contains("no output") {
            self.show_command_output = false;
            changed = true;
        } else if sig.contains("show output") {
            self.show_command_output = true;
            changed = true;
        }

        if sig.contains("show reasoning") || sig.contains("show thinking") {
            self.show_reasoning = true;
            changed = true;
        } else if sig.contains("hide reasoning") || sig.contains("no thinking") {
            self.show_reasoning = false;
            changed = true;
        }

        if changed {
            self.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Record a topic the user asked about
    pub fn record_topic(&mut self, topic: &str) {
        if !topic.is_empty() && !self.favorite_topics.contains(&topic.to_string()) {
            self.favorite_topics.push(topic.to_string());
            if self.favorite_topics.len() > 20 {
                self.favorite_topics.remove(0);
            }
        }
    }

    /// Get prompt guidance based on preferences
    pub fn get_prompt_guidance(&self) -> String {
        let detail = match self.detail_level {
            DetailLevel::Minimal => "Be extremely concise. One-line answers preferred.",
            DetailLevel::Normal => "Be clear and direct.",
            DetailLevel::Verbose => "Provide detailed explanations with context.",
        };

        let style = match self.style {
            ResponseStyle::Concise => "Skip explanations, just give the answer.",
            ResponseStyle::Balanced => "",
            ResponseStyle::Educational => "Explain why, not just how.",
        };

        format!("{} {}", detail, style).trim().to_string()
    }
}

/// v0.0.899: Learned contradiction pattern to prevent repeating mistakes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionPattern {
    /// What topic/claim type this applies to
    pub claim_type: String,

    /// The incorrect assertion made
    pub wrong_assertion: String,

    /// The correct value from command output
    pub correct_value: String,

    /// Command that provided ground truth
    pub source_command: String,

    /// How many times this contradiction was detected
    pub occurrences: u32,

    /// When first seen
    pub first_seen: String,

    /// When last seen
    pub last_seen: String,
}

/// v0.0.899: Store for learned contradictions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContradictionStore {
    /// Known contradiction patterns
    pub patterns: Vec<ContradictionPattern>,
}

impl ContradictionStore {
    fn path() -> PathBuf {
        anna_data_dir().join("contradictions.json")
    }

    /// Load or create
    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)?;
        Ok(())
    }

    /// Record a new contradiction
    pub fn record(&mut self, claim_type: &str, wrong: &str, correct: &str, source_cmd: &str) {
        let now = chrono::Utc::now().to_rfc3339();

        // Check if we already know this pattern
        if let Some(existing) = self
            .patterns
            .iter_mut()
            .find(|p| p.claim_type == claim_type && p.wrong_assertion == wrong)
        {
            existing.occurrences += 1;
            existing.last_seen = now;
            return;
        }

        // New pattern
        self.patterns.push(ContradictionPattern {
            claim_type: claim_type.to_string(),
            wrong_assertion: wrong.to_string(),
            correct_value: correct.to_string(),
            source_command: source_cmd.to_string(),
            occurrences: 1,
            first_seen: now.clone(),
            last_seen: now,
        });

        // Keep max 100 patterns
        if self.patterns.len() > 100 {
            self.patterns.sort_by(|a, b| {
                b.occurrences
                    .cmp(&a.occurrences)
                    .then_with(|| b.last_seen.cmp(&a.last_seen))
            });
            self.patterns.truncate(100);
        }
    }

    /// Check if we've seen this type of claim be wrong before
    pub fn check_claim(&self, claim_type: &str, assertion: &str) -> Option<&ContradictionPattern> {
        self.patterns.iter().find(|p| {
            p.claim_type == claim_type
                && (p.wrong_assertion == assertion
                    || p.wrong_assertion.to_lowercase() == assertion.to_lowercase())
        })
    }

    /// Get correction guidance for answer generation
    pub fn get_correction_hints(&self, topic: &str) -> Vec<String> {
        self.patterns
            .iter()
            .filter(|p| p.claim_type.to_lowercase().contains(&topic.to_lowercase()))
            .take(3)
            .map(|p| {
                format!(
                    "Don't say '{}' about {} (correct: {})",
                    p.wrong_assertion, p.claim_type, p.correct_value
                )
            })
            .collect()
    }
}
