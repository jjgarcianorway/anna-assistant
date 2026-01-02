//! Probe learning store (v0.0.401).
//! Persistent storage and core operations for probe learning.
//! v0.0.401: Added specialist recommendation boosting.
//!
//! Implementation split across multiple modules:
//! - `store_core`: File I/O and persistence
//! - `store_feedback`: Feedback recording and pattern tracking
//! - `store_recommendations`: Probe suggestions and recommendations
//! - `store_stats`: Statistics, health, and quality tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{
    KeywordProbeStats, NegativePattern, ProbeEffectiveness, QualityDataPoint, QueryCategory,
    SuccessfulPattern,
};

// Re-export the helper function for use in other modules
pub(super) use super::store_feedback::now_secs;

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
