//! Relevance Engine - v0.25.0
//!
//! Core engine for calculating and maintaining relevance scores.
//! No hardcoded rankings - everything is discovered and scored dynamically.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::protocol_v25::{
    RankedEntity, RankedEntityType, RelevanceFactors, RelevanceScore, UsageEvent, UsageEventType,
};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Default recency decay half-life in seconds (1 hour)
pub const DEFAULT_RECENCY_HALFLIFE_SECS: i64 = 3600;

/// Default minimum usage count for meaningful frequency score
pub const DEFAULT_MIN_USAGE_FOR_FREQUENCY: u64 = 3;

/// Default score refresh interval in seconds (5 minutes)
pub const DEFAULT_SCORE_REFRESH_INTERVAL: i64 = 300;

/// Configuration for the relevance engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceConfig {
    /// Half-life for recency decay (seconds)
    pub recency_halflife_secs: i64,
    /// Minimum usage count for frequency scoring
    pub min_usage_for_frequency: u64,
    /// Score refresh interval (seconds)
    pub score_refresh_interval: i64,
    /// Weight for recency factor
    pub weight_recency: f32,
    /// Weight for frequency factor
    pub weight_frequency: f32,
    /// Weight for session context factor
    pub weight_session: f32,
    /// Weight for user preference factor
    pub weight_preference: f32,
    /// Weight for discovery bonus
    pub weight_discovery: f32,
}

impl Default for RelevanceConfig {
    fn default() -> Self {
        Self {
            recency_halflife_secs: DEFAULT_RECENCY_HALFLIFE_SECS,
            min_usage_for_frequency: DEFAULT_MIN_USAGE_FOR_FREQUENCY,
            score_refresh_interval: DEFAULT_SCORE_REFRESH_INTERVAL,
            weight_recency: 0.25,
            weight_frequency: 0.25,
            weight_session: 0.20,
            weight_preference: 0.20,
            weight_discovery: 0.10,
        }
    }
}

// ============================================================================
// USAGE STATISTICS
// ============================================================================

/// Usage statistics for an entity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total usage count (all time)
    pub total_count: u64,
    /// Usage count in current period (24h rolling)
    pub period_count: u64,
    /// Last usage timestamp
    pub last_used: Option<i64>,
    /// First usage timestamp
    pub first_used: Option<i64>,
    /// Usage by event type
    pub by_event_type: HashMap<String, u64>,
    /// Usage by session (recent sessions only)
    pub by_session: HashMap<String, u64>,
}

impl UsageStats {
    /// Record a usage event
    pub fn record(&mut self, event: &UsageEvent) {
        self.total_count += 1;
        self.period_count += 1;
        self.last_used = Some(event.timestamp);

        if self.first_used.is_none() {
            self.first_used = Some(event.timestamp);
        }

        // Track by event type
        let event_key = format!("{:?}", event.event_type);
        *self.by_event_type.entry(event_key).or_insert(0) += 1;

        // Track by session
        if let Some(ref session_id) = event.session_id {
            *self.by_session.entry(session_id.clone()).or_insert(0) += 1;
        }
    }

    /// Decay period count (call periodically)
    pub fn decay_period(&mut self, decay_factor: f32) {
        self.period_count = (self.period_count as f32 * decay_factor) as u64;
    }

    /// Clean old session data (keep only recent N sessions)
    pub fn clean_old_sessions(&mut self, keep_count: usize) {
        if self.by_session.len() > keep_count {
            // Keep most recent sessions (by count as proxy)
            let mut sessions: Vec<_> = self.by_session.iter().collect();
            sessions.sort_by(|a, b| b.1.cmp(a.1));
            sessions.truncate(keep_count);
            self.by_session = sessions.into_iter().map(|(k, v)| (k.clone(), *v)).collect();
        }
    }
}

// ============================================================================
// RELEVANCE CALCULATOR
// ============================================================================

/// Calculator for relevance scores
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct RelevanceCalculator {
    /// Configuration
    pub config: RelevanceConfig,
    /// Current session ID
    pub current_session: Option<String>,
    /// User preferences (entity_id -> preference_score)
    pub user_preferences: HashMap<String, f32>,
}


impl RelevanceCalculator {
    /// Create with custom config
    pub fn with_config(config: RelevanceConfig) -> Self {
        Self {
            config,
            current_session: None,
            user_preferences: HashMap::new(),
        }
    }

    /// Set current session
    pub fn set_session(&mut self, session_id: String) {
        self.current_session = Some(session_id);
    }

    /// Set user preference for an entity
    pub fn set_preference(&mut self, entity_id: &str, preference: f32) {
        self.user_preferences
            .insert(entity_id.to_string(), preference.clamp(0.0, 1.0));
    }

    /// Calculate relevance score for an entity
    pub fn calculate(&self, entity_id: &str, stats: &UsageStats, is_discovered: bool) -> RelevanceScore {
        let factors = self.calculate_factors(entity_id, stats, is_discovered);
        let score = self.weighted_score(&factors);
        RelevanceScore::new(score, factors)
    }

    /// Calculate individual factors
    fn calculate_factors(&self, entity_id: &str, stats: &UsageStats, is_discovered: bool) -> RelevanceFactors {
        RelevanceFactors {
            recency: self.recency_factor(stats),
            frequency: self.frequency_factor(stats),
            session_context: self.session_factor(stats),
            user_preference: self.preference_factor(entity_id),
            discovery_bonus: if is_discovered { 1.0 } else { 0.5 },
        }
    }

    /// Calculate recency factor using exponential decay
    fn recency_factor(&self, stats: &UsageStats) -> f32 {
        let Some(last_used) = stats.last_used else {
            return 0.0;
        };

        let now = chrono::Utc::now().timestamp();
        let age_secs = (now - last_used).max(0);

        // Exponential decay: score = 0.5^(age / halflife)
        let halflife = self.config.recency_halflife_secs as f64;
        let decay = 0.5_f64.powf(age_secs as f64 / halflife);

        decay as f32
    }

    /// Calculate frequency factor
    fn frequency_factor(&self, stats: &UsageStats) -> f32 {
        let min = self.config.min_usage_for_frequency;
        if stats.period_count < min {
            return (stats.period_count as f32) / (min as f32);
        }

        // Logarithmic scaling above minimum
        let count = stats.period_count as f32;
        let base = min as f32;
        let factor = (count / base).ln() / 3.0 + 1.0;

        factor.min(1.0)
    }

    /// Calculate session context factor
    fn session_factor(&self, stats: &UsageStats) -> f32 {
        let Some(ref current) = self.current_session else {
            return 0.0;
        };

        // Check if entity was used in current session
        if stats.by_session.contains_key(current) {
            let session_count = stats.by_session.get(current).copied().unwrap_or(0);
            // More uses in session = higher context relevance
            let factor = (session_count as f32 / 5.0).min(1.0);
            return factor;
        }

        0.0
    }

    /// Get user preference factor
    fn preference_factor(&self, entity_id: &str) -> f32 {
        self.user_preferences.get(entity_id).copied().unwrap_or(0.5)
    }

    /// Calculate weighted score from factors
    fn weighted_score(&self, factors: &RelevanceFactors) -> f32 {
        let c = &self.config;
        let score = (factors.recency * c.weight_recency)
            + (factors.frequency * c.weight_frequency)
            + (factors.session_context * c.weight_session)
            + (factors.user_preference * c.weight_preference)
            + (factors.discovery_bonus * c.weight_discovery);

        score.clamp(0.0, 1.0)
    }
}

// ============================================================================
// RANKING ENGINE
// ============================================================================

/// Engine for ranking entities by relevance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RankingEngine {
    /// Usage stats per entity
    pub stats: HashMap<String, UsageStats>,
    /// Cached scores (refreshed periodically)
    pub cached_scores: HashMap<String, RelevanceScore>,
    /// Entity metadata
    pub entities: HashMap<String, EntityMetadata>,
}

/// Metadata about an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    /// Entity ID
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Entity type
    pub entity_type: RankedEntityType,
    /// Was discovered via probes (not hardcoded)
    pub is_discovered: bool,
    /// When this entity was first seen
    pub first_seen: i64,
}

impl RankingEngine {
    /// Create new ranking engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an entity
    pub fn register_entity(&mut self, id: &str, display_name: &str, entity_type: RankedEntityType, is_discovered: bool) {
        let now = chrono::Utc::now().timestamp();
        self.entities.insert(
            id.to_string(),
            EntityMetadata {
                id: id.to_string(),
                display_name: display_name.to_string(),
                entity_type,
                is_discovered,
                first_seen: now,
            },
        );
        self.stats.entry(id.to_string()).or_default();
    }

    /// Record a usage event
    pub fn record_usage(&mut self, entity_id: &str, event: &UsageEvent) {
        self.stats
            .entry(entity_id.to_string())
            .or_default()
            .record(event);

        // Invalidate cached score
        self.cached_scores.remove(entity_id);
    }

    /// Get ranked list of entities (highest relevance first)
    pub fn get_ranked(&self, calculator: &RelevanceCalculator, entity_type: Option<RankedEntityType>) -> Vec<RankedEntity> {
        let mut ranked: Vec<_> = self
            .entities
            .iter()
            .filter(|(_, meta)| entity_type.is_none() || Some(&meta.entity_type) == entity_type.as_ref())
            .map(|(id, meta)| {
                let stats = self.stats.get(id).cloned().unwrap_or_default();
                let score = calculator.calculate(id, &stats, meta.is_discovered);
                RankedEntity {
                    id: id.clone(),
                    entity_type: meta.entity_type.clone(),
                    display_name: meta.display_name.clone(),
                    relevance: score,
                    usage_count: stats.total_count,
                    last_used: stats.last_used,
                    discovered_via: super::protocol_v25::DiscoverySource::Probe {
                        probe_id: "discovery".to_string(),
                    },
                }
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.relevance.score.partial_cmp(&a.relevance.score).unwrap_or(std::cmp::Ordering::Equal)
        });

        ranked
    }

    /// Get top N entities by relevance
    pub fn get_top(&self, calculator: &RelevanceCalculator, n: usize, entity_type: Option<RankedEntityType>) -> Vec<RankedEntity> {
        self.get_ranked(calculator, entity_type).into_iter().take(n).collect()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_stats_record() {
        let mut stats = UsageStats::default();
        let event = UsageEvent::new(UsageEventType::AppLaunch, "firefox".to_string())
            .with_session("session-001".to_string());

        stats.record(&event);

        assert_eq!(stats.total_count, 1);
        assert!(stats.last_used.is_some());
        assert!(stats.by_session.contains_key("session-001"));
    }

    #[test]
    fn test_relevance_calculator() {
        let mut calc = RelevanceCalculator::default();
        calc.set_session("session-001".to_string());
        calc.set_preference("firefox", 0.9);

        let mut stats = UsageStats::default();
        let event = UsageEvent::new(UsageEventType::AppLaunch, "firefox".to_string())
            .with_session("session-001".to_string());
        stats.record(&event);

        let score = calc.calculate("firefox", &stats, true);
        assert!(score.score > 0.0);
        assert!(score.factors.discovery_bonus == 1.0);
    }

    #[test]
    fn test_recency_decay() {
        let calc = RelevanceCalculator::default();

        // Recent usage should have high recency
        let mut recent_stats = UsageStats::default();
        recent_stats.last_used = Some(chrono::Utc::now().timestamp());
        let recent_factor = calc.recency_factor(&recent_stats);
        assert!(recent_factor > 0.9);

        // Old usage should have low recency
        let mut old_stats = UsageStats::default();
        old_stats.last_used = Some(chrono::Utc::now().timestamp() - 7200); // 2 hours ago
        let old_factor = calc.recency_factor(&old_stats);
        assert!(old_factor < 0.5);
    }

    #[test]
    fn test_ranking_engine() {
        let mut engine = RankingEngine::new();
        engine.register_entity("firefox", "Firefox", RankedEntityType::Application, true);
        engine.register_entity("chrome", "Chrome", RankedEntityType::Application, true);

        // Record more usage for firefox
        for _ in 0..5 {
            let event = UsageEvent::new(UsageEventType::AppLaunch, "firefox".to_string());
            engine.record_usage("firefox", &event);
        }

        let calc = RelevanceCalculator::default();
        let ranked = engine.get_ranked(&calc, Some(RankedEntityType::Application));

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].id, "firefox"); // More usage = higher rank
    }

    #[test]
    fn test_frequency_factor() {
        let calc = RelevanceCalculator::default();

        let mut low_stats = UsageStats::default();
        low_stats.period_count = 1;
        let low = calc.frequency_factor(&low_stats);

        let mut high_stats = UsageStats::default();
        high_stats.period_count = 10;
        let high = calc.frequency_factor(&high_stats);

        assert!(high > low);
    }
}
