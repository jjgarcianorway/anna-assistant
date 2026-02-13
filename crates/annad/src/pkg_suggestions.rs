//! Package suggestions from usage patterns.
//!
//! Design: Anna observes which topics the user repeatedly asks about,
//! then asks the LLM (grounded in Arch Wiki) what tools would help.
//! No hardcoded topic→package mappings — the LLM + wiki decides.
//!
//! Stored in /var/lib/anna/pkg_suggestions.json.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

use anna_shared::config::anna_data_dir;

/// A suggested package with rationale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSuggestion {
    /// Package name
    pub package: String,
    /// Why Anna is suggesting this (grounded in usage patterns + wiki)
    pub rationale: String,
    /// The recurring topic that triggered this
    pub trigger_topic: String,
    /// Number of times user asked about this topic
    pub topic_frequency: u32,
    /// Whether user accepted/rejected this suggestion
    pub accepted: Option<bool>,
    /// Unix timestamp
    pub suggested_at: u64,
}

/// Persistent store
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuggestionStore {
    pub suggestions: Vec<PackageSuggestion>,
    /// Topics the user has asked about and their frequency
    pub topic_counts: std::collections::HashMap<String, u32>,
}

impl SuggestionStore {
    fn path() -> PathBuf {
        anna_data_dir().join("pkg_suggestions.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, s);
        }
    }

    /// Record that user asked about a topic. Returns true if threshold crossed.
    pub fn record_topic(&mut self, topic: &str) -> bool {
        let count = self.topic_counts.entry(topic.to_string()).or_insert(0);
        *count += 1;
        *count >= 3 // Suggest after 3 occurrences
    }

    /// Get topics that recur but have no accepted suggestion yet.
    pub fn pending_topics(&self) -> Vec<(String, u32)> {
        let already_suggested: std::collections::HashSet<&str> = self.suggestions.iter()
            .filter(|s| s.accepted != Some(false))
            .map(|s| s.trigger_topic.as_str())
            .collect();

        self.topic_counts.iter()
            .filter(|(topic, count)| **count >= 3 && !already_suggested.contains(topic.as_str()))
            .map(|(t, c)| (t.clone(), *c))
            .collect()
    }

    /// Mark a suggestion as accepted or rejected.
    pub fn mark(&mut self, package: &str, accepted: bool) {
        for s in self.suggestions.iter_mut() {
            if s.package == package {
                s.accepted = Some(accepted);
            }
        }
        self.save();
    }

    /// Pending suggestions not yet shown to user.
    pub fn unshown_suggestions(&self) -> Vec<&PackageSuggestion> {
        self.suggestions.iter().filter(|s| s.accepted.is_none()).collect()
    }
}

/// Extract topic keywords from a question for pattern matching.
/// This is intentionally minimal — just normalize and extract nouns.
pub fn extract_topic(question: &str) -> String {
    let q = question.to_lowercase();
    let stop_words = ["what", "how", "why", "when", "where", "is", "are", "does",
        "do", "can", "will", "my", "the", "a", "an", "to", "in", "on", "of", "for"];

    q.split_whitespace()
        .filter(|w| !stop_words.contains(w) && w.len() >= 3)
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check for new package suggestions based on accumulated patterns.
/// Uses LLM + Arch Wiki to decide what tool would help, never hardcodes.
pub async fn check_for_suggestions(model: &str, question: &str) {
    let topic = extract_topic(question);
    if topic.is_empty() {
        return;
    }

    let mut store = SuggestionStore::load();
    if !store.record_topic(&topic) {
        store.save();
        return; // Not frequent enough yet
    }

    // Already have an unrejected suggestion for this topic?
    if store.suggestions.iter().any(|s| s.trigger_topic == topic && s.accepted != Some(false)) {
        store.save();
        return;
    }

    info!("Topic '{}' recurring ({} times), generating package suggestion", topic,
        store.topic_counts.get(&topic).unwrap_or(&0));

    // Search wiki for this topic
    let wiki_context = anna_shared::wiki::search::keyword_search_text(&topic, 1200)
        .unwrap_or_default();

    // Ask LLM: what Arch Linux package would help this user?
    let prompt = format!(
        "An Arch Linux user has asked about '{}' {} times. \
        Based on this pattern and the following Arch Wiki content, \
        suggest ONE specific Arch Linux package (from pacman or AUR) \
        that would help them.\n\n\
        Wiki content:\n{}\n\n\
        Respond with ONLY:\n\
        PACKAGE: <package-name>\n\
        REASON: <one sentence why this package fits their usage pattern>\n\
        If no package would clearly help, respond: NONE",
        topic,
        store.topic_counts.get(&topic).unwrap_or(&0),
        wiki_context
    );

    match crate::ollama::chat_with_timeout(model, &prompt, 25).await {
        Ok(response) => {
            parse_and_store_suggestion(response, topic, &mut store);
            store.save();
        }
        Err(e) => warn!("LLM error generating package suggestion: {}", e),
    }
}

fn parse_and_store_suggestion(response: String, topic: String, store: &mut SuggestionStore) {
    let response = response.trim();
    if response.starts_with("NONE") || response.is_empty() {
        return;
    }

    let mut package = String::new();
    let mut reason = String::new();

    for line in response.lines() {
        if let Some(rest) = line.strip_prefix("PACKAGE:") {
            package = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("REASON:") {
            reason = rest.trim().to_string();
        }
    }

    if package.is_empty() || package.contains(' ') {
        // LLM gave invalid package name
        return;
    }

    let frequency = store.topic_counts.get(&topic).cloned().unwrap_or(0);
    info!("Package suggestion: {} for topic '{}'", package, topic);

    store.suggestions.push(PackageSuggestion {
        package,
        rationale: reason,
        trigger_topic: topic,
        topic_frequency: frequency,
        accepted: None,
        suggested_at: unix_now(),
    });
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Format pending suggestions for briefing.
pub fn pending_suggestions_briefing() -> String {
    let store = SuggestionStore::load();
    let pending = store.unshown_suggestions();
    if pending.is_empty() {
        return String::new();
    }

    let mut out = "## Package Suggestions (from your usage patterns)\n".to_string();
    for s in pending.iter().take(3) {
        out.push_str(&format!(
            "- Install `{}`: {} (you asked about '{}' {} times)\n",
            s.package, s.rationale, s.trigger_topic, s.topic_frequency
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_topic() {
        let t = extract_topic("what is my bandwidth usage?");
        assert!(t.contains("bandwidth") || t.contains("usage"));
    }

    #[test]
    fn test_record_topic_threshold() {
        let mut store = SuggestionStore::default();
        assert!(!store.record_topic("network"));
        assert!(!store.record_topic("network"));
        assert!(store.record_topic("network")); // 3rd = true
    }

    #[test]
    fn test_pending_topics() {
        let mut store = SuggestionStore::default();
        *store.topic_counts.entry("monitoring".to_string()).or_insert(0) = 5;
        let pending = store.pending_topics();
        assert!(!pending.is_empty());
    }
}
