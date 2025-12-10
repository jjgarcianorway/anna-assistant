//! Probe effectiveness learning system (v0.0.327).
//!
//! Tracks which probes work well for which query types, learning from:
//! 1. User feedback (helpful/not helpful)
//! 2. LLM self-assessment (answer quality rating)
//! 3. Probe failure rates
//! 4. Query keyword patterns (v0.0.325)
//! 5. Learning decay for old patterns (v0.0.327)
//!
//! This allows the translator to prefer better-performing probes over time.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Probe effectiveness record for a specific query category
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProbeEffectiveness {
    /// Number of times this probe was used for this category
    pub uses: u32,
    /// Number of times the answer was marked helpful
    pub helpful: u32,
    /// Number of times the answer was marked not helpful
    pub not_helpful: u32,
    /// Number of times the probe command failed (non-zero exit)
    pub failures: u32,
    /// Computed effectiveness score (0.0 - 1.0)
    pub score: f32,
}

impl ProbeEffectiveness {
    /// Calculate effectiveness score based on usage stats
    pub fn compute_score(&mut self) {
        if self.uses == 0 {
            self.score = 0.5; // Neutral for unused probes
            return;
        }

        // Base score from helpful/not helpful ratio
        let total_feedback = self.helpful + self.not_helpful;
        let feedback_score = if total_feedback > 0 {
            self.helpful as f32 / total_feedback as f32
        } else {
            0.5 // Neutral if no feedback
        };

        // Penalty for failures
        let failure_rate = self.failures as f32 / self.uses as f32;
        let failure_penalty = 1.0 - (failure_rate * 0.5); // Max 50% penalty

        // Confidence boost for more uses (bayesian-ish)
        let confidence = (self.uses as f32 / 10.0).min(1.0);

        // Blend neutral prior with observed score based on confidence
        self.score = (0.5 * (1.0 - confidence) + feedback_score * confidence) * failure_penalty;
    }
}

/// Query category for grouping probe effectiveness
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryCategory {
    /// System health (CPU, memory, processes)
    SystemHealth,
    /// Disk and storage
    Storage,
    /// Network and connectivity
    Network,
    /// Hardware info (GPU, USB, PCI)
    Hardware,
    /// Security and permissions
    Security,
    /// Packages and software
    Packages,
    /// Services and systemd
    Services,
    /// Graphics and display
    Graphics,
    /// General/other
    General,
}

impl QueryCategory {
    /// Infer category from domain string
    pub fn from_domain(domain: &str) -> Self {
        match domain.to_lowercase().as_str() {
            "system" => Self::SystemHealth,
            "storage" => Self::Storage,
            "network" => Self::Network,
            "security" => Self::Security,
            "packages" => Self::Packages,
            _ => Self::General,
        }
    }

    /// Infer category from query keywords
    pub fn from_query(query: &str) -> Self {
        let q = query.to_lowercase();

        // Graphics/display queries
        if q.contains("gpu") || q.contains("graphics") || q.contains("display")
            || q.contains("vaapi") || q.contains("vdpau") || q.contains("vulkan")
            || q.contains("hardware acceleration") || q.contains("video acceleration")
            || q.contains("render") {
            return Self::Graphics;
        }

        // Hardware queries
        if q.contains("usb") || q.contains("pci") || q.contains("bluetooth")
            || q.contains("printer") || q.contains("audio") || q.contains("sound") {
            return Self::Hardware;
        }

        // Network queries
        if q.contains("network") || q.contains("wifi") || q.contains("ethernet")
            || q.contains("ip address") || q.contains("dns") || q.contains("ping") {
            return Self::Network;
        }

        // Storage queries
        if q.contains("disk") || q.contains("storage") || q.contains("space")
            || q.contains("mount") || q.contains("partition") {
            return Self::Storage;
        }

        // Security queries
        if q.contains("firewall") || q.contains("permission") || q.contains("security")
            || q.contains("user") || q.contains("group") {
            return Self::Security;
        }

        // Package queries
        if q.contains("package") || q.contains("install") || q.contains("update")
            || q.contains("pacman") || q.contains("apt") || q.contains("dnf") {
            return Self::Packages;
        }

        // Service queries
        if q.contains("service") || q.contains("systemd") || q.contains("daemon")
            || q.contains("running") && q.contains("process") {
            return Self::Services;
        }

        // System health queries
        if q.contains("cpu") || q.contains("memory") || q.contains("ram")
            || q.contains("process") || q.contains("load") || q.contains("uptime") {
            return Self::SystemHealth;
        }

        Self::General
    }
}

/// Probe learning store - persists probe effectiveness data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProbeLearningStore {
    /// Effectiveness scores by (category, probe_id)
    pub effectiveness: HashMap<QueryCategory, HashMap<String, ProbeEffectiveness>>,
    /// Query patterns that led to poor answers (for negative learning)
    pub negative_patterns: Vec<NegativePattern>,
    /// v0.0.325: Keyword to probe mapping (learned associations)
    #[serde(default)]
    pub keyword_probes: HashMap<String, KeywordProbeStats>,
    /// v0.0.325: Successful query patterns (for positive learning)
    #[serde(default)]
    pub successful_patterns: Vec<SuccessfulPattern>,
    /// v0.0.327: Last decay timestamp (Unix seconds)
    #[serde(default)]
    pub last_decay_time: u64,
    /// Version for migration
    pub version: u32,
}

/// v0.0.325: Stats for keyword-probe associations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeywordProbeStats {
    /// Probes that worked well for this keyword
    pub effective_probes: HashMap<String, u32>,
    /// Total times this keyword appeared in successful queries
    pub success_count: u32,
}

/// v0.0.325: A successful query pattern for positive learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessfulPattern {
    /// Keywords extracted from the query
    pub keywords: Vec<String>,
    /// Probes that were used successfully
    pub probes: Vec<String>,
    /// Quality score (1-5 or reliability-based)
    pub quality: u8,
    /// Category
    pub category: QueryCategory,
    /// Timestamp
    pub timestamp: u64,
}

/// A pattern that led to a poor answer (for learning what NOT to do)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegativePattern {
    /// The query that got a bad answer
    pub query: String,
    /// The category it was assigned
    pub category: QueryCategory,
    /// Probes that were used
    pub probes_used: Vec<String>,
    /// Why the answer was bad (from user/LLM feedback)
    pub failure_reason: String,
    /// Timestamp
    pub timestamp: u64,
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
    fn store_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".anna").join("probe_learning.json")
    }

    /// v0.0.329: Reset all learning data
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
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
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

        // Sort by score descending
        recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        recommendations
    }

    /// Check if a query+probe combo has been problematic before
    pub fn is_known_bad_combo(&self, query: &str, probes: &[String]) -> Option<&str> {
        let q_lower = query.to_lowercase();
        for pattern in &self.negative_patterns {
            // Simple keyword overlap check
            let pattern_words: Vec<&str> = pattern.query.split_whitespace().collect();
            let query_words: Vec<&str> = q_lower.split_whitespace().collect();
            let overlap = pattern_words.iter()
                .filter(|w| query_words.contains(w))
                .count();

            // If high keyword overlap and same probes, flag it
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
        let total_uses: u32 = self.effectiveness.values()
            .flat_map(|m| m.values())
            .map(|e| e.uses)
            .sum();
        let negative_patterns = self.negative_patterns.len();

        format!(
            "{} categories, {} probes tracked, {} uses, {} negative patterns",
            total_categories, total_probes, total_uses, negative_patterns
        )
    }

    /// v0.0.325: Record a successful query pattern
    pub fn record_success(&mut self, query: &str, probes: &[String], quality: u8, category: QueryCategory) {
        // Extract keywords from query
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
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        });

        // Keep only last 200 successful patterns
        if self.successful_patterns.len() > 200 {
            self.successful_patterns.remove(0);
        }
    }

    /// v0.0.325: Get probe suggestions based on query keywords
    pub fn suggest_probes_for_query(&self, query: &str) -> Vec<(String, u32)> {
        let keywords = extract_keywords(query);

        if keywords.is_empty() {
            return vec![];
        }

        // Aggregate probe scores from matching keywords
        let mut probe_scores: HashMap<String, u32> = HashMap::new();

        for keyword in &keywords {
            if let Some(stats) = self.keyword_probes.get(keyword) {
                for (probe, count) in &stats.effective_probes {
                    *probe_scores.entry(probe.clone()).or_insert(0) += count;
                }
            }
        }

        // Sort by score
        let mut suggestions: Vec<_> = probe_scores.into_iter().collect();
        suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        suggestions.truncate(5); // Top 5

        suggestions
    }

    /// v0.0.325: Get learning stats for display
    pub fn learning_stats(&self) -> LearningStats {
        LearningStats {
            total_queries: self.successful_patterns.len() + self.negative_patterns.len(),
            successful_patterns: self.successful_patterns.len(),
            negative_patterns: self.negative_patterns.len(),
            keywords_learned: self.keyword_probes.len(),
            categories_with_data: self.effectiveness.len(),
            avg_quality: self.successful_patterns.iter()
                .map(|p| p.quality as f32)
                .sum::<f32>() / self.successful_patterns.len().max(1) as f32,
        }
    }

    /// v0.0.327: Apply decay to old learning data
    /// Should be called periodically (e.g., on load or weekly)
    /// This ensures recent experiences have more weight than old ones
    pub fn apply_decay(&mut self) -> DecayResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Only decay if it's been more than a week since last decay
        const DECAY_INTERVAL_SECS: u64 = 7 * 24 * 60 * 60; // 1 week
        if now - self.last_decay_time < DECAY_INTERVAL_SECS {
            return DecayResult::skipped();
        }

        // Remove old successful patterns (older than 30 days)
        const PATTERN_MAX_AGE_SECS: u64 = 30 * 24 * 60 * 60; // 30 days
        let old_pattern_count = self.successful_patterns.len();
        self.successful_patterns.retain(|p| now - p.timestamp < PATTERN_MAX_AGE_SECS);
        let mut patterns_removed = old_pattern_count - self.successful_patterns.len();

        // Remove old negative patterns (older than 14 days - we learn from mistakes faster)
        const NEGATIVE_MAX_AGE_SECS: u64 = 14 * 24 * 60 * 60; // 14 days
        let old_negative_count = self.negative_patterns.len();
        self.negative_patterns.retain(|p| now - p.timestamp < NEGATIVE_MAX_AGE_SECS);
        patterns_removed += old_negative_count - self.negative_patterns.len();

        // Decay keyword counts (reduce by 20%, remove if too low)
        let old_keyword_count = self.keyword_probes.len();
        for stats in self.keyword_probes.values_mut() {
            stats.success_count = (stats.success_count * 80) / 100;
            for count in stats.effective_probes.values_mut() {
                *count = (*count * 80) / 100;
            }
            // Remove probes with count < 1
            stats.effective_probes.retain(|_, c| *c >= 1);
        }
        // Remove keywords with no data
        self.keyword_probes.retain(|_, stats| stats.success_count >= 1 && !stats.effective_probes.is_empty());
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
            // Remove probes with no data
            category_map.retain(|_, eff| eff.uses >= 1);
        }
        // Remove empty categories
        self.effectiveness.retain(|_, m| !m.is_empty());

        self.last_decay_time = now;

        DecayResult {
            applied: true,
            patterns_removed,
            keywords_decayed,
            probes_decayed,
        }
    }

    /// v0.0.327: Apply decay if needed on load
    pub fn load_with_decay() -> Self {
        let mut store = Self::load();
        let result = store.apply_decay();
        if result.applied {
            let _ = store.save(); // Save decayed state
        }
        store
    }
}

/// v0.0.327: Result of applying decay
#[derive(Debug, Clone)]
pub struct DecayResult {
    pub applied: bool,
    pub patterns_removed: usize,
    pub keywords_decayed: usize,
    pub probes_decayed: usize,
}

impl DecayResult {
    fn skipped() -> Self {
        Self {
            applied: false,
            patterns_removed: 0,
            keywords_decayed: 0,
            probes_decayed: 0,
        }
    }
}

/// v0.0.325: Learning statistics for display
#[derive(Debug, Clone)]
pub struct LearningStats {
    pub total_queries: usize,
    pub successful_patterns: usize,
    pub negative_patterns: usize,
    pub keywords_learned: usize,
    pub categories_with_data: usize,
    pub avg_quality: f32,
}

/// v0.0.325: Extract meaningful keywords from a query
fn extract_keywords(query: &str) -> Vec<String> {
    // Stop words to filter out
    const STOP_WORDS: &[&str] = &[
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "shall", "can", "need", "dare",
        "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
        "into", "through", "during", "before", "after", "above", "below",
        "between", "under", "again", "further", "then", "once", "here",
        "there", "when", "where", "why", "how", "all", "each", "few",
        "more", "most", "other", "some", "such", "no", "nor", "not",
        "only", "own", "same", "so", "than", "too", "very", "just",
        "i", "me", "my", "you", "your", "we", "our", "it", "its",
        "what", "which", "who", "whom", "this", "that", "these", "those",
        "am", "and", "but", "if", "or", "because", "until", "while",
        "about", "show", "tell", "give", "get", "check", "see", "look",
    ];

    query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !STOP_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_query() {
        assert_eq!(QueryCategory::from_query("what gpu do I have"), QueryCategory::Graphics);
        assert_eq!(QueryCategory::from_query("check disk space"), QueryCategory::Storage);
        assert_eq!(QueryCategory::from_query("list usb devices"), QueryCategory::Hardware);
        assert_eq!(QueryCategory::from_query("how much ram"), QueryCategory::SystemHealth);
        assert_eq!(QueryCategory::from_query("random question"), QueryCategory::General);
    }

    #[test]
    fn test_effectiveness_score() {
        let mut eff = ProbeEffectiveness::default();
        eff.uses = 10;
        eff.helpful = 8;
        eff.not_helpful = 2;
        eff.failures = 1;
        eff.compute_score();

        // Should be high but not perfect due to some failures
        assert!(eff.score > 0.7);
        assert!(eff.score < 0.95);
    }

    #[test]
    fn test_record_feedback() {
        let mut store = ProbeLearningStore::default();

        // Record some positive feedback
        store.record_usage(QueryCategory::Graphics, "gpu_info", false);
        store.record_feedback(
            QueryCategory::Graphics,
            &["gpu_info".to_string()],
            true,
            None,
            None,
        );

        let recs = store.get_recommended_probes(&QueryCategory::Graphics);
        assert!(!recs.is_empty());
        assert_eq!(recs[0].0, "gpu_info");
    }

    #[test]
    fn test_decay_reduces_counts() {
        let mut store = ProbeLearningStore::default();

        // Set last decay time to long ago so decay will apply
        store.last_decay_time = 0;

        // Add some probe data
        store.record_usage(QueryCategory::Graphics, "gpu_info", false);
        store.record_usage(QueryCategory::Graphics, "gpu_info", false);
        store.record_usage(QueryCategory::Graphics, "gpu_info", false);
        store.record_usage(QueryCategory::Graphics, "gpu_info", false);
        store.record_usage(QueryCategory::Graphics, "gpu_info", false); // 5 uses

        // Apply decay
        let result = store.apply_decay();
        assert!(result.applied);

        // Check that counts were reduced (5 * 0.8 = 4)
        let eff = store.effectiveness
            .get(&QueryCategory::Graphics)
            .and_then(|m| m.get("gpu_info"));
        assert!(eff.is_some());
        assert_eq!(eff.unwrap().uses, 4);
    }

    #[test]
    fn test_decay_skipped_if_recent() {
        let mut store = ProbeLearningStore::default();

        // Set last decay time to recent (now)
        store.last_decay_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let result = store.apply_decay();
        assert!(!result.applied); // Should be skipped
    }
}
