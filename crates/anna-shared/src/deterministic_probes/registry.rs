//! Probe registry and matching logic.

use crate::deterministic_probes::types::ProbeRule;

/// Registry of deterministic probe rules.
pub struct DeterministicProbeRegistry {
    rules: Vec<ProbeRule>,
}

impl DeterministicProbeRegistry {
    /// Create registry with all rules.
    pub fn new() -> Self {
        let mut rules = Vec::new();

        // Collect all rules from different categories
        rules.extend(super::rules_cpu::cpu_rules());
        rules.extend(super::rules_memory::memory_rules());
        rules.extend(super::rules_system::system_rules());
        rules.extend(super::rules_hardware::hardware_rules());
        rules.extend(super::rules_storage::storage_rules());
        rules.extend(super::rules_network::network_rules());
        rules.extend(super::rules_config::config_rules());
        rules.extend(super::rules_misc::misc_rules());

        Self { rules }
    }

    /// Find matching rule for a query.
    /// Returns the first rule where all keywords match and no negative keywords match.
    pub fn find_rule(&self, query: &str) -> Option<&ProbeRule> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        for rule in &self.rules {
            // Check all keywords present
            let all_keywords_match = rule
                .keywords
                .iter()
                .all(|kw| query_words.iter().any(|w| w.contains(kw)));

            if !all_keywords_match {
                continue;
            }

            // Check no negative keywords present
            let no_negative_match = rule
                .negative_keywords
                .iter()
                .all(|nkw| !query_words.iter().any(|w| w.contains(nkw)));

            if no_negative_match {
                return Some(rule);
            }
        }

        None
    }

    /// Get probes for a query. Returns None if no deterministic rule matches.
    pub fn get_probes(&self, query: &str) -> Option<Vec<&'static str>> {
        self.find_rule(query).map(|rule| rule.probes.to_vec())
    }

    /// Check if query should NEVER be treated as a package query.
    /// These are concept queries that happen to contain words that might be package names.
    pub fn is_concept_not_package(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Words that are concepts, not packages
        let concept_words = [
            "swap",
            "games",
            "apps",
            "tools",
            "utils",
            "drivers",
            "audio",
            "sound",
            "video",
            "network",
            "bluetooth",
            "wifi",
            "graphics",
            "display",
            "desktop",
            "fonts",
            "themes",
        ];

        // Package verbs that indicate actual package intent
        let package_verbs = [
            "install",
            "remove",
            "uninstall",
            "update",
            "upgrade",
            "pacman",
            "apt",
            "yum",
        ];

        let has_concept_word = concept_words.iter().any(|w| query_lower.contains(w));
        let has_package_verb = package_verbs.iter().any(|v| query_lower.contains(v));

        // If it has a concept word but no package verb, it's a concept query
        has_concept_word && !has_package_verb
    }
}

impl Default for DeterministicProbeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a query matches a deterministic probe rule.
/// Returns the probes if matched, None otherwise.
pub fn deterministic_probes_for_query(query: &str) -> Option<Vec<&'static str>> {
    DeterministicProbeRegistry::new().get_probes(query)
}

/// Check if query is a concept (not a package query).
pub fn is_concept_query(query: &str) -> bool {
    DeterministicProbeRegistry::new().is_concept_not_package(query)
}
