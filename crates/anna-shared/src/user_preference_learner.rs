//! User Preference Learner - Phase 98
//!
//! Learns user preferences from interactions.
//! Adapts Anna's behavior based on observed patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Preference category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PreferenceCategory {
    #[default]
    Communication,
    Technical,
    Schedule,
    Notification,
    Display,
    Privacy,
}

impl PreferenceCategory {
    pub fn name(&self) -> &'static str {
        match self {
            PreferenceCategory::Communication => "Communication",
            PreferenceCategory::Technical => "Technical",
            PreferenceCategory::Schedule => "Schedule",
            PreferenceCategory::Notification => "Notification",
            PreferenceCategory::Display => "Display",
            PreferenceCategory::Privacy => "Privacy",
        }
    }
}

/// Confidence in learned preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LearnConfidence {
    #[default]
    Low,
    Medium,
    High,
    Confirmed,
}

impl LearnConfidence {
    pub fn name(&self) -> &'static str {
        match self {
            LearnConfidence::Low => "Low",
            LearnConfidence::Medium => "Medium",
            LearnConfidence::High => "High",
            LearnConfidence::Confirmed => "Confirmed",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            LearnConfidence::Low => 25,
            LearnConfidence::Medium => 50,
            LearnConfidence::High => 75,
            LearnConfidence::Confirmed => 100,
        }
    }
}

/// A learned preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPreference {
    /// Preference key
    pub key: String,
    /// Preference value
    pub value: String,
    /// Category
    pub category: PreferenceCategory,
    /// Confidence level
    pub confidence: LearnConfidence,
    /// Times observed
    pub observations: u64,
    /// Last observed timestamp
    pub last_observed: u64,
    /// User confirmed
    pub user_confirmed: bool,
}

/// User preference learner
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPreferenceLearner {
    /// All learned preferences
    pub preferences: Vec<LearnedPreference>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Count by confidence
    pub by_confidence: HashMap<String, u64>,
    /// Total observations
    pub total_observations: u64,
    /// User confirmations
    pub user_confirmations: u64,
}

impl UserPreferenceLearner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Learn a preference from observation
    pub fn learn(&mut self, key: &str, value: &str, category: PreferenceCategory, timestamp: u64) {
        let found = self.preferences.iter().position(|p| p.key == key);

        if let Some(idx) = found {
            // Update existing
            self.preferences[idx].observations += 1;
            self.preferences[idx].last_observed = timestamp;
            self.preferences[idx].value = value.to_string();

            // Increase confidence with more observations
            let obs = self.preferences[idx].observations;
            let old_confidence = self.preferences[idx].confidence;
            let new_confidence = match obs {
                0..=2 => LearnConfidence::Low,
                3..=5 => LearnConfidence::Medium,
                6..=10 => LearnConfidence::High,
                _ => LearnConfidence::High,
            };

            if new_confidence != old_confidence {
                if let Some(count) = self.by_confidence.get_mut(old_confidence.name()) {
                    *count = count.saturating_sub(1);
                }
                *self.by_confidence.entry(new_confidence.name().to_string()).or_insert(0) += 1;
                self.preferences[idx].confidence = new_confidence;
            }
        } else {
            // New preference
            let pref = LearnedPreference {
                key: key.to_string(),
                value: value.to_string(),
                category,
                confidence: LearnConfidence::Low,
                observations: 1,
                last_observed: timestamp,
                user_confirmed: false,
            };
            *self.by_category.entry(category.name().to_string()).or_insert(0) += 1;
            *self.by_confidence.entry(LearnConfidence::Low.name().to_string()).or_insert(0) += 1;
            self.preferences.push(pref);
        }
        self.total_observations += 1;
    }

    /// Confirm a preference (user explicitly confirmed)
    pub fn confirm(&mut self, key: &str) -> bool {
        let found = self.preferences.iter().position(|p| p.key == key);
        if let Some(idx) = found {
            if !self.preferences[idx].user_confirmed {
                let old_confidence = self.preferences[idx].confidence;
                if let Some(count) = self.by_confidence.get_mut(old_confidence.name()) {
                    *count = count.saturating_sub(1);
                }
                *self.by_confidence.entry(LearnConfidence::Confirmed.name().to_string()).or_insert(0) += 1;

                self.preferences[idx].confidence = LearnConfidence::Confirmed;
                self.preferences[idx].user_confirmed = true;
                self.user_confirmations += 1;
            }
            true
        } else {
            false
        }
    }

    /// Get preference by key
    pub fn get(&self, key: &str) -> Option<&LearnedPreference> {
        self.preferences.iter().find(|p| p.key == key)
    }

    /// Get preferences by category
    pub fn by_pref_category(&self, category: PreferenceCategory) -> Vec<&LearnedPreference> {
        self.preferences.iter().filter(|p| p.category == category).collect()
    }

    /// Get high-confidence preferences
    pub fn high_confidence(&self) -> Vec<&LearnedPreference> {
        self.preferences
            .iter()
            .filter(|p| {
                p.confidence == LearnConfidence::High || p.confidence == LearnConfidence::Confirmed
            })
            .collect()
    }

    /// Get confirmed preferences
    pub fn confirmed(&self) -> Vec<&LearnedPreference> {
        self.preferences.iter().filter(|p| p.user_confirmed).collect()
    }

    /// Total preference count
    pub fn total_count(&self) -> usize {
        self.preferences.len()
    }

    /// High confidence count
    pub fn high_confidence_count(&self) -> usize {
        self.high_confidence().len()
    }
}

/// Format preference learner for display
pub fn format_preference_learner(learner: &UserPreferenceLearner) -> String {
    let mut lines = vec!["=== User Preference Learner ===".to_string()];
    lines.push(String::new());

    if learner.preferences.is_empty() {
        lines.push("No preferences learned yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total preferences: {}", learner.total_count()));
    lines.push(format!("High confidence: {}", learner.high_confidence_count()));
    lines.push(format!("User confirmed: {}", learner.user_confirmations));
    lines.push(format!("Total observations: {}", learner.total_observations));

    // By category
    if !learner.by_category.is_empty() {
        lines.push(String::new());
        lines.push("By category:".to_string());
        for (cat, count) in &learner.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    // High confidence preferences
    let high = learner.high_confidence();
    if !high.is_empty() {
        lines.push(String::new());
        lines.push("High confidence preferences:".to_string());
        for pref in high.iter().take(10) {
            let confirmed = if pref.user_confirmed { " [✓]" } else { "" };
            lines.push(format!("  {} = {}{}", pref.key, pref.value, confirmed));
        }
    }

    lines.join("\n")
}

/// Format preference learner compact
pub fn format_preference_learner_compact(learner: &UserPreferenceLearner) -> String {
    format!(
        "Preferences: {} learned | {} high confidence | {} confirmed",
        learner.total_count(),
        learner.high_confidence_count(),
        learner.user_confirmations
    )
}

/// Format preference learner one-line
pub fn format_preference_learner_oneline(learner: &UserPreferenceLearner) -> String {
    format!(
        "{} preferences ({} confirmed)",
        learner.total_count(),
        learner.user_confirmations
    )
}

/// Check if query is about preferences
pub fn is_preference_learner_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "preference",
        "preferences",
        "learned about me",
        "what do you know about me",
        "my settings",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about preferences
pub fn preference_learner_fun_fact(learner: &UserPreferenceLearner) -> String {
    if learner.preferences.is_empty() {
        return "No preferences learned yet!".to_string();
    }

    let facts = [
        format!("Anna has learned {} preferences about you.", learner.total_count()),
        format!("{} preferences are high confidence.", learner.high_confidence_count()),
        format!("You've confirmed {} preferences.", learner.user_confirmations),
    ];

    facts[learner.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_category() {
        assert_eq!(PreferenceCategory::Communication.name(), "Communication");
        assert_eq!(PreferenceCategory::Technical.name(), "Technical");
    }

    #[test]
    fn test_learn_confidence() {
        assert_eq!(LearnConfidence::High.name(), "High");
        assert_eq!(LearnConfidence::High.score(), 75);
    }

    #[test]
    fn test_learn_preference() {
        let mut learner = UserPreferenceLearner::new();
        learner.learn("verbosity", "verbose", PreferenceCategory::Communication, 1000);

        assert_eq!(learner.total_count(), 1);
        assert!(learner.get("verbosity").is_some());
    }

    #[test]
    fn test_confidence_increase() {
        let mut learner = UserPreferenceLearner::new();
        // Learn same preference multiple times
        for i in 0..6 {
            learner.learn("editor", "vim", PreferenceCategory::Technical, i);
        }

        let pref = learner.get("editor").unwrap();
        assert_eq!(pref.confidence, LearnConfidence::High);
        assert_eq!(pref.observations, 6);
    }

    #[test]
    fn test_confirm_preference() {
        let mut learner = UserPreferenceLearner::new();
        learner.learn("shell", "zsh", PreferenceCategory::Technical, 1000);
        learner.confirm("shell");

        let pref = learner.get("shell").unwrap();
        assert!(pref.user_confirmed);
        assert_eq!(pref.confidence, LearnConfidence::Confirmed);
    }

    #[test]
    fn test_by_category() {
        let mut learner = UserPreferenceLearner::new();
        learner.learn("editor", "vim", PreferenceCategory::Technical, 1000);
        learner.learn("verbosity", "brief", PreferenceCategory::Communication, 1000);

        assert_eq!(learner.by_pref_category(PreferenceCategory::Technical).len(), 1);
        assert_eq!(learner.by_pref_category(PreferenceCategory::Communication).len(), 1);
    }

    #[test]
    fn test_high_confidence() {
        let mut learner = UserPreferenceLearner::new();
        learner.learn("editor", "vim", PreferenceCategory::Technical, 1000);
        learner.confirm("editor");

        assert_eq!(learner.high_confidence().len(), 1);
    }

    #[test]
    fn test_format_learner() {
        let mut learner = UserPreferenceLearner::new();
        learner.learn("editor", "vim", PreferenceCategory::Technical, 1000);

        let output = format_preference_learner(&learner);
        assert!(output.contains("User Preference Learner"));
        assert!(output.contains("Total preferences: 1"));
    }

    #[test]
    fn test_is_preference_query() {
        assert!(is_preference_learner_query("show my preferences"));
        assert!(is_preference_learner_query("what do you know about me?"));
        assert!(!is_preference_learner_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut learner = UserPreferenceLearner::new();
        learner.learn("editor", "vim", PreferenceCategory::Technical, 1000);

        let fact = preference_learner_fun_fact(&learner);
        assert!(!fact.is_empty());
    }
}
