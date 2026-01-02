//! Probe recommendation and suggestion logic.
//! Provides intelligent probe suggestions based on learned patterns.

use std::collections::HashMap;

use super::store::ProbeLearningStore;
use super::types::QueryCategory;
use super::utils::extract_keywords;

impl ProbeLearningStore {
    /// Get probe recommendations for a category (sorted by effectiveness)
    pub fn get_recommended_probes(&self, category: &QueryCategory) -> Vec<(String, f32)> {
        let mut recommendations: Vec<(String, f32)> = self
            .effectiveness
            .get(category)
            .map(|m| {
                m.iter()
                    .map(|(probe_id, eff)| (probe_id.clone(), eff.score))
                    .collect()
            })
            .unwrap_or_default();

        recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        recommendations
    }

    /// Get probe suggestions based on query keywords
    /// v0.0.371: Now uses semantic matching via canonicalization
    pub fn suggest_probes_for_query(&self, query: &str) -> Vec<(String, u32)> {
        use super::utils::canonicalize;

        let keywords = extract_keywords(query);

        if keywords.is_empty() {
            return vec![];
        }

        let mut probe_scores: HashMap<String, u32> = HashMap::new();

        // Canonicalize query keywords for better matching
        let canonical_keywords: Vec<String> = keywords.iter().map(|k| canonicalize(k)).collect();

        for (stored_keyword, stats) in &self.keyword_probes {
            // Check both exact and canonical matches
            let stored_canonical = canonicalize(stored_keyword);
            let matches =
                keywords.contains(stored_keyword) || canonical_keywords.contains(&stored_canonical);

            if matches {
                for (probe, count) in &stats.effective_probes {
                    *probe_scores.entry(probe.clone()).or_insert(0) += count;
                }
            }
        }

        let mut suggestions: Vec<_> = probe_scores.into_iter().collect();
        suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        suggestions.truncate(5);

        suggestions
    }

    /// v0.0.377: Get recent high-quality keywords for a category (for specialist hints)
    pub fn recent_success_hints(&self, category: &QueryCategory) -> Vec<String> {
        self.successful_patterns
            .iter()
            .rev() // Most recent first
            .filter(|p| &p.category == category && p.quality >= 4)
            .take(3)
            .flat_map(|p| p.keywords.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .take(6) // Max 6 keywords
            .collect()
    }
}
