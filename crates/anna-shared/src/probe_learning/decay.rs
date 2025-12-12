//! Learning decay system (v0.0.331).
//!
//! Decay old learning data so recent experiences have more weight.

use super::store::ProbeLearningStore;
use super::types::DecayResult;

/// Time constants for decay
const DECAY_INTERVAL_SECS: u64 = 7 * 24 * 60 * 60; // 1 week
const PATTERN_MAX_AGE_SECS: u64 = 30 * 24 * 60 * 60; // 30 days
const NEGATIVE_MAX_AGE_SECS: u64 = 14 * 24 * 60 * 60; // 14 days

impl ProbeLearningStore {
    /// Apply decay to old learning data
    /// Should be called periodically (e.g., on load or weekly)
    pub fn apply_decay(&mut self) -> DecayResult {
        let now = now_secs();

        // Only decay if it's been more than a week since last decay
        if now - self.last_decay_time < DECAY_INTERVAL_SECS {
            return DecayResult::skipped();
        }

        // Remove old successful patterns (older than 30 days)
        let old_pattern_count = self.successful_patterns.len();
        self.successful_patterns
            .retain(|p| now - p.timestamp < PATTERN_MAX_AGE_SECS);
        let mut patterns_removed = old_pattern_count - self.successful_patterns.len();

        // Remove old negative patterns (older than 14 days)
        let old_negative_count = self.negative_patterns.len();
        self.negative_patterns
            .retain(|p| now - p.timestamp < NEGATIVE_MAX_AGE_SECS);
        patterns_removed += old_negative_count - self.negative_patterns.len();

        // Decay keyword counts (reduce by 20%, remove if too low)
        let old_keyword_count = self.keyword_probes.len();
        for stats in self.keyword_probes.values_mut() {
            stats.success_count = (stats.success_count * 80) / 100;
            for count in stats.effective_probes.values_mut() {
                *count = (*count * 80) / 100;
            }
            stats.effective_probes.retain(|_, c| *c >= 1);
        }
        self.keyword_probes
            .retain(|_, stats| stats.success_count >= 1 && !stats.effective_probes.is_empty());
        let keywords_decayed = old_keyword_count - self.keyword_probes.len();

        let mut probes_decayed = 0;

        // Decay probe effectiveness (reduce counts by 20%)
        for category_map in self.effectiveness.values_mut() {
            for eff in category_map.values_mut() {
                eff.uses = (eff.uses * 80) / 100;
                eff.helpful = (eff.helpful * 80) / 100;
                eff.not_helpful = (eff.not_helpful * 80) / 100;
                eff.failures = (eff.failures * 80) / 100;
                eff.compute_score();
                if eff.uses > 0 {
                    probes_decayed += 1;
                }
            }
            category_map.retain(|_, eff| eff.uses >= 1);
        }
        self.effectiveness.retain(|_, m| !m.is_empty());

        self.last_decay_time = now;

        DecayResult {
            applied: true,
            patterns_removed,
            keywords_decayed,
            probes_decayed,
        }
    }
}

/// Get current Unix timestamp in seconds
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
