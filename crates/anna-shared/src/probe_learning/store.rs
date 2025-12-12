//! Probe learning store (v0.0.401).
//! Persistent storage and core operations for probe learning.
//! v0.0.401: Added specialist recommendation boosting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::types::{
    KeywordProbeStats, LearningHealth, LearningStats, NegativePattern, ProbeEffectiveness,
    QualityDataPoint, QualityTrend, QueryCategory, SuccessfulPattern, TrendDirection,
};
use super::utils::extract_keywords;

/// Probe learning store - persists probe effectiveness data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProbeLearningStore {
    /// Effectiveness scores by (category, probe_id)
    pub effectiveness: HashMap<QueryCategory, HashMap<String, ProbeEffectiveness>>,
    /// Query patterns that led to poor answers (for negative learning)
    pub negative_patterns: Vec<NegativePattern>,
    /// Keyword to probe mapping (learned associations)
    #[serde(default)]
    pub keyword_probes: HashMap<String, KeywordProbeStats>,
    /// Successful query patterns (for positive learning)
    #[serde(default)]
    pub successful_patterns: Vec<SuccessfulPattern>,
    /// Last decay timestamp (Unix seconds)
    #[serde(default)]
    pub last_decay_time: u64,
    /// v0.0.331: Quality trend history (weekly averages)
    #[serde(default)]
    pub quality_history: Vec<QualityDataPoint>,
    /// Version for migration
    pub version: u32,
}

impl ProbeLearningStore {
    /// Load from disk or create new
    pub fn load() -> Self {
        let path = Self::store_path();
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Store path
    pub fn store_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".anna")
            .join("probe_learning.json")
    }

    /// Reset all learning data
    pub fn reset() -> Result<(), String> {
        let path = Self::store_path();
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

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

    /// Get summary stats for display
    pub fn summary(&self) -> String {
        let total_categories = self.effectiveness.len();
        let total_probes: usize = self.effectiveness.values().map(|m| m.len()).sum();
        let total_uses: u32 = self
            .effectiveness
            .values()
            .flat_map(|m| m.values())
            .map(|e| e.uses)
            .sum();
        let negative_patterns = self.negative_patterns.len();

        format!(
            "{} categories, {} probes tracked, {} uses, {} negative patterns",
            total_categories, total_probes, total_uses, negative_patterns
        )
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

    /// Get learning stats for display
    pub fn learning_stats(&self) -> LearningStats {
        LearningStats {
            total_queries: self.successful_patterns.len() + self.negative_patterns.len(),
            successful_patterns: self.successful_patterns.len(),
            negative_patterns: self.negative_patterns.len(),
            keywords_learned: self.keyword_probes.len(),
            categories_with_data: self.effectiveness.len(),
            avg_quality: self
                .successful_patterns
                .iter()
                .map(|p| p.quality as f32)
                .sum::<f32>()
                / self.successful_patterns.len().max(1) as f32,
        }
    }

    /// v0.0.331: Get quality trend (comparing recent vs previous period)
    pub fn quality_trend(&self) -> Option<QualityTrend> {
        let now = now_secs();
        let week = 7 * 24 * 60 * 60;
        let recent: Vec<_> = self
            .successful_patterns
            .iter()
            .filter(|p| now - p.timestamp < week)
            .collect();
        let previous: Vec<_> = self
            .successful_patterns
            .iter()
            .filter(|p| {
                let age = now - p.timestamp;
                age >= week && age < 2 * week
            })
            .collect();

        if recent.is_empty() && previous.is_empty() {
            return None;
        }

        let current_avg = if recent.is_empty() {
            0.0
        } else {
            recent.iter().map(|p| p.quality as f32).sum::<f32>() / recent.len() as f32
        };
        let previous_avg = if previous.is_empty() {
            current_avg
        } else {
            previous.iter().map(|p| p.quality as f32).sum::<f32>() / previous.len() as f32
        };

        let change = current_avg - previous_avg;
        let trend = if change > 0.3 {
            TrendDirection::Improving
        } else if change < -0.3 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        Some(QualityTrend {
            current_avg,
            previous_avg,
            trend,
            change,
        })
    }

    /// v0.0.331: Update quality history (called after recording success)
    fn update_quality_history(&mut self) {
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
        self.quality_history.push(QualityDataPoint {
            timestamp: today_start,
            avg_quality: avg,
            query_count: today.len() as u32,
        });
        if self.quality_history.len() > 30 {
            self.quality_history.remove(0);
        }
    }

    /// Apply decay if needed on load
    pub fn load_with_decay() -> Self {
        let mut store = Self::load();
        let result = store.apply_decay();
        if result.applied {
            let _ = store.save();
        }
        store
    }

    /// v0.0.332: Get confidence factor (0.0-1.0) based on learning health
    pub fn confidence_factor(&self) -> f32 {
        let stats = self.learning_stats();
        let volume = (stats.total_queries as f32 / 50.0).min(1.0);
        let quality = stats.avg_quality / 5.0;
        let diversity = (stats.keywords_learned as f32 / 30.0).min(1.0);
        let trend = match self.quality_trend() {
            Some(t) => match t.trend {
                TrendDirection::Improving => 1.1,
                TrendDirection::Stable => 1.0,
                TrendDirection::Declining => 0.8,
            },
            None => 0.9,
        };
        ((volume * 0.4 + quality * 0.3 + diversity * 0.3) * trend).clamp(0.0, 1.0)
    }

    /// v0.0.332: Should we trust learned recommendations?
    /// Returns true if confidence is above threshold
    pub fn should_use_learning(&self) -> bool {
        self.confidence_factor() >= 0.3
    }

    /// v0.0.332: Get learning health status
    pub fn health_status(&self) -> LearningHealth {
        let confidence = self.confidence_factor();
        let trend = self.quality_trend();
        if confidence >= 0.7 {
            LearningHealth::Excellent
        } else if confidence >= 0.5 {
            if let Some(t) = &trend {
                if t.trend == TrendDirection::Declining {
                    return LearningHealth::NeedsAttention;
                }
            }
            LearningHealth::Good
        } else if confidence >= 0.3 {
            LearningHealth::Developing
        } else {
            LearningHealth::Insufficient
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

/// Get current Unix timestamp in seconds
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
