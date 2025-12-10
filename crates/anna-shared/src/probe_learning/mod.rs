//! Probe effectiveness learning system (v0.0.332).
//!
//! Tracks which probes work well for which query types, learning from:
//! 1. User feedback (helpful/not helpful)
//! 2. LLM self-assessment (answer quality rating)
//! 3. Probe failure rates
//! 4. Query keyword patterns
//! 5. Learning decay for old patterns
//! 6. Quality trend tracking (v0.0.331)
//!
//! This allows the translator to prefer better-performing probes over time.
//!
//! ## Module Structure
//! - `types`: Core data structures (ProbeEffectiveness, QueryCategory, etc.)
//! - `store`: Persistent storage and main operations
//! - `decay`: Learning decay for old data
//! - `utils`: Helper functions (keyword extraction)

mod decay;
mod store;
mod types;
mod utils;

// Re-export public API
pub use store::ProbeLearningStore;
pub use types::{
    DecayResult, KeywordProbeStats, LearningHealth, LearningStats, NegativePattern,
    ProbeEffectiveness, QualityDataPoint, QualityTrend, QueryCategory, SuccessfulPattern,
    TrendDirection,
};
pub use utils::extract_keywords;

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

        assert!(eff.score > 0.7);
        assert!(eff.score < 0.95);
    }

    #[test]
    fn test_record_feedback() {
        let mut store = ProbeLearningStore::default();

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
        store.last_decay_time = 0;

        for _ in 0..5 {
            store.record_usage(QueryCategory::Graphics, "gpu_info", false);
        }

        let result = store.apply_decay();
        assert!(result.applied);

        let eff = store.effectiveness
            .get(&QueryCategory::Graphics)
            .and_then(|m| m.get("gpu_info"));
        assert!(eff.is_some());
        assert_eq!(eff.unwrap().uses, 4); // 5 * 0.8 = 4
    }

    #[test]
    fn test_decay_skipped_if_recent() {
        let mut store = ProbeLearningStore::default();
        store.last_decay_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let result = store.apply_decay();
        assert!(!result.applied);
    }

    #[test]
    fn test_quality_trend_none_when_empty() {
        let store = ProbeLearningStore::default();
        assert!(store.quality_trend().is_none());
    }
}
