//! Deeper context memory for cross-session continuity (v0.0.246).
//!
//! Remembers user patterns, frequently used queries, learned preferences,
//! and provides natural continuity across sessions.
//!
//! v0.0.246: Initial implementation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// User interaction pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPattern {
    /// Topic/domain of interaction
    pub topic: String,
    /// How many times this topic was discussed
    pub count: u32,
    /// Last time this topic came up
    pub last_seen: DateTime<Utc>,
    /// Typical time of day user asks about this
    pub typical_hours: Vec<u8>,
    /// Related queries made with this topic
    pub related_queries: Vec<String>,
}

impl InteractionPattern {
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            count: 1,
            last_seen: Utc::now(),
            typical_hours: vec![],
            related_queries: vec![],
        }
    }

    pub fn bump(&mut self, hour: u8, query: Option<String>) {
        self.count += 1;
        self.last_seen = Utc::now();
        if !self.typical_hours.contains(&hour) {
            self.typical_hours.push(hour);
        }
        if let Some(q) = query {
            if !self.related_queries.contains(&q) && self.related_queries.len() < 5 {
                self.related_queries.push(q);
            }
        }
    }

    /// Is this a frequent topic for this user?
    pub fn is_frequent(&self) -> bool {
        self.count >= 3
    }

    /// Was this discussed recently (within 24 hours)?
    pub fn is_recent(&self) -> bool {
        let elapsed = Utc::now() - self.last_seen;
        elapsed.num_hours() < 24
    }
}

/// Learned preference from user behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPreference {
    /// What preference was learned
    pub preference: String,
    /// Confidence level (0.0-1.0)
    pub confidence: f32,
    /// How it was learned
    pub source: String,
    /// When it was learned
    pub learned_at: DateTime<Utc>,
}

/// Context continuity item - something to remember across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuityItem {
    /// What happened
    pub description: String,
    /// When it happened
    pub when: DateTime<Utc>,
    /// Optional action hint
    pub action: Option<String>,
    /// Priority for recall
    pub priority: u8,
}

/// Main context memory store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextMemory {
    /// Interaction patterns by topic
    pub patterns: HashMap<String, InteractionPattern>,
    /// Learned preferences
    pub preferences: Vec<LearnedPreference>,
    /// Continuity items (things to remember)
    pub continuity: Vec<ContinuityItem>,
    /// User's primary editor (if detected)
    pub preferred_editor: Option<String>,
    /// User's shell preference (if detected)
    pub preferred_shell: Option<String>,
    /// Frequently mentioned paths
    pub frequent_paths: Vec<String>,
    /// Commands user has mastered (no longer needs explanation)
    pub mastered_commands: Vec<String>,
    /// Last meaningful interaction
    pub last_interaction: Option<DateTime<Utc>>,
}

impl ContextMemory {
    /// Load from disk
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save to disk
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
        dirs::data_dir().map(|d| d.join("anna").join("context_memory.json"))
    }

    /// Record an interaction with a topic
    pub fn record_interaction(&mut self, topic: &str, query: Option<&str>) {
        let hour = Utc::now().format("%H").to_string().parse().unwrap_or(12);

        if let Some(pattern) = self.patterns.get_mut(topic) {
            pattern.bump(hour, query.map(String::from));
        } else {
            let mut pattern = InteractionPattern::new(topic);
            pattern.typical_hours.push(hour);
            if let Some(q) = query {
                pattern.related_queries.push(q.to_string());
            }
            self.patterns.insert(topic.to_string(), pattern);
        }

        self.last_interaction = Some(Utc::now());
    }

    /// Learn a preference from user behavior
    pub fn learn_preference(&mut self, preference: &str, source: &str, confidence: f32) {
        // Update existing or add new
        if let Some(existing) = self.preferences.iter_mut().find(|p| p.preference == preference) {
            // Increase confidence if we see it again
            existing.confidence = (existing.confidence + confidence) / 2.0;
            existing.learned_at = Utc::now();
        } else {
            self.preferences.push(LearnedPreference {
                preference: preference.to_string(),
                confidence,
                source: source.to_string(),
                learned_at: Utc::now(),
            });
        }

        // Keep only top 10 preferences
        self.preferences.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        self.preferences.truncate(10);
    }

    /// Add a continuity item to remember
    pub fn remember(&mut self, description: &str, action: Option<&str>, priority: u8) {
        self.continuity.push(ContinuityItem {
            description: description.to_string(),
            when: Utc::now(),
            action: action.map(String::from),
            priority,
        });

        // Keep only last 20 items
        self.continuity.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.continuity.truncate(20);
    }

    /// Mark a command as mastered (user knows it well)
    pub fn mark_mastered(&mut self, command: &str) {
        if !self.mastered_commands.contains(&command.to_string()) {
            self.mastered_commands.push(command.to_string());
        }
    }

    /// Record detected editor preference
    pub fn detect_editor(&mut self, editor: &str) {
        self.preferred_editor = Some(editor.to_string());
    }

    /// Get frequent topics for this user
    pub fn frequent_topics(&self) -> Vec<&InteractionPattern> {
        self.patterns.values().filter(|p| p.is_frequent()).collect()
    }

    /// Get recent topics for continuity
    pub fn recent_topics(&self) -> Vec<&InteractionPattern> {
        self.patterns.values().filter(|p| p.is_recent()).collect()
    }

    /// Get high-priority continuity items
    pub fn pending_continuity(&self) -> Vec<&ContinuityItem> {
        self.continuity.iter().filter(|c| c.priority >= 50).collect()
    }

    /// Generate a continuity message for greeting
    pub fn continuity_greeting(&self) -> Option<String> {
        // Check for recent high-priority items
        let pending = self.pending_continuity();
        if let Some(item) = pending.first() {
            let elapsed = Utc::now() - item.when;
            if elapsed.num_hours() < 48 {
                return Some(format!("By the way, {}", item.description));
            }
        }

        // Check for frequent recent topics
        let recent = self.recent_topics();
        if let Some(topic) = recent.first() {
            if topic.count >= 3 {
                return Some(format!(
                    "You've been asking about {} a lot - want me to keep an eye on it?",
                    topic.topic
                ));
            }
        }

        None
    }

    /// Should we explain this command or has user mastered it?
    pub fn should_explain(&self, command: &str) -> bool {
        !self.mastered_commands.contains(&command.to_string())
    }
}

/// Generate context-aware response hints
pub fn response_hints(memory: &ContextMemory, current_topic: &str) -> Vec<String> {
    let mut hints = Vec::new();

    // Check if user frequently asks about this topic
    if let Some(pattern) = memory.patterns.get(current_topic) {
        if pattern.is_frequent() && !pattern.related_queries.is_empty() {
            hints.push(format!(
                "You've asked about {} {} times before",
                current_topic, pattern.count
            ));
        }
    }

    // Check for editor preference
    if let Some(ref editor) = memory.preferred_editor {
        if current_topic.contains("edit") || current_topic.contains("config") {
            hints.push(format!("Using your preferred editor: {}", editor));
        }
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_pattern() {
        let mut pattern = InteractionPattern::new("vim");
        assert_eq!(pattern.count, 1);

        pattern.bump(14, Some("how to exit vim".to_string()));
        assert_eq!(pattern.count, 2);
        assert!(!pattern.related_queries.is_empty());
    }

    #[test]
    fn test_context_memory_record() {
        let mut memory = ContextMemory::default();
        memory.record_interaction("vim", Some("how to exit vim"));
        memory.record_interaction("vim", Some("vim copy paste"));
        memory.record_interaction("vim", None);

        assert!(memory.patterns.contains_key("vim"));
        assert_eq!(memory.patterns.get("vim").unwrap().count, 3);
    }

    #[test]
    fn test_learned_preference() {
        let mut memory = ContextMemory::default();
        memory.learn_preference("prefers nvim over vim", "mentioned nvim", 0.7);
        memory.learn_preference("prefers nvim over vim", "opened nvim", 0.8);

        assert_eq!(memory.preferences.len(), 1);
        assert!(memory.preferences[0].confidence > 0.7);
    }

    #[test]
    fn test_mastered_commands() {
        let mut memory = ContextMemory::default();
        memory.mark_mastered("ls");
        memory.mark_mastered("cd");

        assert!(!memory.should_explain("ls"));
        assert!(memory.should_explain("rsync"));
    }

    #[test]
    fn test_frequent_topics() {
        let mut memory = ContextMemory::default();
        memory.record_interaction("disk", None);
        memory.record_interaction("disk", None);
        memory.record_interaction("disk", None);
        memory.record_interaction("network", None);

        let frequent = memory.frequent_topics();
        assert_eq!(frequent.len(), 1);
        assert_eq!(frequent[0].topic, "disk");
    }
}
