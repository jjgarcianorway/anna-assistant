//! Feedback recording and pattern tracking.
//! Handles user feedback, usage tracking, and negative pattern detection.

use super::store::ProbeLearningStore;
use super::types::{NegativePattern, QueryCategory, SuccessfulPattern};
use super::utils::extract_keywords;

/// Get current Unix timestamp in seconds
pub(super) fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl ProbeLearningStore {
    /// Record probe usage for a query
    pub fn record_usage(&mut self, category: QueryCategory, probe_id: &str, failed: bool) {
        let category_map = self.effectiveness.entry(category).or_default();
        let probe = category_map.entry(probe_id.to_string()).or_default();
        probe.uses += 1;
        if failed {
            probe.failures += 1;
        }
        probe.compute_score();
    }

    /// Record feedback (helpful or not)
    pub fn record_feedback(
        &mut self,
        category: QueryCategory,
        probes: &[String],
        helpful: bool,
        query: Option<&str>,
        failure_reason: Option<&str>,
    ) {
        let category_map = self.effectiveness.entry(category.clone()).or_default();

        for probe_id in probes {
            let probe = category_map.entry(probe_id.to_string()).or_default();
            if helpful {
                probe.helpful += 1;
            } else {
                probe.not_helpful += 1;
            }
            probe.compute_score();
        }

        // Record negative pattern for learning
        if !helpful {
            if let (Some(q), Some(reason)) = (query, failure_reason) {
                self.negative_patterns.push(NegativePattern {
                    query: q.to_string(),
                    category,
                    probes_used: probes.to_vec(),
                    failure_reason: reason.to_string(),
                    timestamp: now_secs(),
                });

                // Keep only last 100 negative patterns
                if self.negative_patterns.len() > 100 {
                    self.negative_patterns.remove(0);
                }
            }
        }
    }

    /// Check if a query+probe combo has been problematic before
    pub fn is_known_bad_combo(&self, query: &str, probes: &[String]) -> Option<&str> {
        let q_lower = query.to_lowercase();
        for pattern in &self.negative_patterns {
            let pattern_words: Vec<&str> = pattern.query.split_whitespace().collect();
            let query_words: Vec<&str> = q_lower.split_whitespace().collect();
            let overlap = pattern_words
                .iter()
                .filter(|w| query_words.contains(w))
                .count();

            if overlap >= 2 && probes.iter().any(|p| pattern.probes_used.contains(p)) {
                return Some(&pattern.failure_reason);
            }
        }
        None
    }

    /// Record a successful query pattern
    pub fn record_success(
        &mut self,
        query: &str,
        probes: &[String],
        quality: u8,
        category: QueryCategory,
    ) {
        let keywords = extract_keywords(query);

        if keywords.is_empty() || probes.is_empty() {
            return;
        }

        // Update keyword-probe associations
        for keyword in &keywords {
            let stats = self.keyword_probes.entry(keyword.clone()).or_default();
            stats.success_count += 1;
            for probe in probes {
                *stats.effective_probes.entry(probe.clone()).or_insert(0) += 1;
            }
        }

        // Store successful pattern
        self.successful_patterns.push(SuccessfulPattern {
            keywords,
            probes: probes.to_vec(),
            quality,
            category,
            timestamp: now_secs(),
        });

        // Keep only last 200 successful patterns
        if self.successful_patterns.len() > 200 {
            self.successful_patterns.remove(0);
        }

        // v0.0.331: Update quality history
        self.update_quality_history();
    }

    /// v0.0.331: Update quality history (called after recording success)
    pub(super) fn update_quality_history(&mut self) {
        let now = now_secs();
        let day = 24 * 60 * 60;
        let needs_new = self
            .quality_history
            .last()
            .map(|l| now - l.timestamp >= day)
            .unwrap_or(true);
        if !needs_new {
            return;
        }

        let today_start = now - (now % day);
        let today: Vec<_> = self
            .successful_patterns
            .iter()
            .filter(|p| p.timestamp >= today_start)
            .collect();
        if today.is_empty() {
            return;
        }

        let avg = today.iter().map(|p| p.quality as f32).sum::<f32>() / today.len() as f32;
        self.quality_history.push(super::types::QualityDataPoint {
            timestamp: today_start,
            avg_quality: avg,
            query_count: today.len() as u32,
        });
        if self.quality_history.len() > 30 {
            self.quality_history.remove(0);
        }
    }

    /// v0.0.401: Boost probes recommended by specialist (high confidence)
    /// Called when a specialist interaction successfully used these probes
    pub fn boost_specialist_probes(
        &mut self,
        category: QueryCategory,
        probes: &[String],
        boost: u32,
    ) {
        let cat_map = self.effectiveness.entry(category).or_default();
        for probe_id in probes {
            let eff = cat_map.entry(probe_id.clone()).or_default();
            eff.uses += boost;
            eff.helpful += boost;
            eff.compute_score();
        }
    }
}
