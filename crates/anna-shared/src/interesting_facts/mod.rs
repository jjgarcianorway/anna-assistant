//! Interesting facts for greeting personalization (v0.0.291).
//!
//! Generates data-driven facts about system, hardware, user patterns,
//! and performance for the LLM translator to naturalize into greetings.
//!
//! Key principle: All facts derived from actual data, no hardcoded content.
//!
//! v0.0.291: Refactored into modules for maintainability.

mod generators;
mod types;

use crate::event_log::{AggregatedEvents, EventLog};
use crate::learning_progress::{compute_learning_progress, LearningProgress};
use crate::snapshot::SystemSnapshot;
use crate::system_telemetry::TelemetryStore;
use serde::{Deserialize, Serialize};

// Re-export types
pub use generators::{growth_facts, hardware_facts, performance_facts, user_pattern_facts};
pub use types::{FactCategory, InterestingFact};

/// Collection of interesting facts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterestingFacts {
    pub facts: Vec<InterestingFact>,
}

impl InterestingFacts {
    /// Generate facts from all available data sources
    pub fn generate(
        snapshot: Option<&SystemSnapshot>,
        telemetry: Option<&TelemetryStore>,
        events: Option<&AggregatedEvents>,
        progress: Option<&LearningProgress>,
    ) -> Self {
        let mut facts = Vec::new();

        // Hardware/uptime facts
        if let Some(snap) = snapshot {
            facts.extend(generators::hardware_facts(snap));
        }

        // Performance trend facts
        if let Some(tel) = telemetry {
            facts.extend(generators::performance_facts(tel));
        }

        // User pattern facts
        if let Some(agg) = events {
            facts.extend(generators::user_pattern_facts(agg));
        }

        // Anna's growth facts
        if let Some(prog) = progress {
            facts.extend(generators::growth_facts(prog));
        }

        // Sort by priority
        facts.sort_by_key(|f| f.priority);

        Self { facts }
    }

    /// Load all data and generate facts
    pub fn from_current_state(snapshot: &SystemSnapshot) -> Self {
        let event_log = EventLog::new(EventLog::default_path(), 10000);
        let events = event_log.aggregate().ok();
        let telemetry = TelemetryStore::load_if_exists();
        let progress = compute_learning_progress();

        Self::generate(
            Some(snapshot),
            telemetry.as_ref(),
            events.as_ref(),
            Some(&progress),
        )
    }

    /// Get top N facts for greeting
    pub fn top(&self, n: usize) -> Vec<&InterestingFact> {
        self.facts.iter().take(n).collect()
    }

    /// Get facts as strings for LLM context
    pub fn as_strings(&self, max: usize) -> Vec<String> {
        self.facts
            .iter()
            .take(max)
            .map(|f| f.fact.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_facts() {
        let facts = InterestingFacts::generate(None, None, None, None);
        assert!(facts.facts.is_empty());
    }

    #[test]
    fn test_hardware_uptime_fact() {
        let mut snapshot = SystemSnapshot::default();
        // Set boot time to 8 days ago
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        snapshot.boot_time_secs = now - (8 * 86400);

        let facts = hardware_facts(&snapshot);
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.fact.contains("8 days")));
    }

    #[test]
    fn test_milestone_detection() {
        let mut events = AggregatedEvents::default();
        events.total_requests = 100;
        events.verified_count = 95;

        let facts = user_pattern_facts(&events);
        assert!(facts.iter().any(|f| f.fact.contains("100 requests")));
    }

    #[test]
    fn test_growth_facts() {
        let mut progress = LearningProgress::default();
        progress.recipes_total = 50;
        progress.self_sufficiency = 0.6;
        progress.strong_areas = vec!["storage".to_string(), "network".to_string()];

        let facts = growth_facts(&progress);
        assert!(!facts.is_empty());
        assert!(facts.iter().any(|f| f.fact.contains("50 recipes")));
    }

    #[test]
    fn test_facts_sorted_by_priority() {
        let mut snapshot = SystemSnapshot::default();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        snapshot.boot_time_secs = now - (8 * 86400);
        snapshot.memory_total_bytes = 16 * 1024 * 1024 * 1024;
        snapshot.memory_used_bytes = 4 * 1024 * 1024 * 1024;

        let facts = InterestingFacts::generate(Some(&snapshot), None, None, None);

        // Check facts are sorted by priority (ascending)
        for window in facts.facts.windows(2) {
            assert!(window[0].priority <= window[1].priority);
        }
    }
}
