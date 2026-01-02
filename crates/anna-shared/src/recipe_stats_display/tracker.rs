//! Recipe statistics tracking and aggregation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::category::RecipeCategory;
use super::origin::RecipeOriginType;
use super::stats::RecipeStats;

/// Recipe statistics tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStatsTracker {
    /// All recipe stats
    pub recipes: HashMap<String, RecipeStats>,
    /// Count by category
    pub by_category: HashMap<RecipeCategory, u64>,
    /// Count by origin
    pub by_origin: HashMap<RecipeOriginType, u64>,
    /// Total uses across all recipes
    pub total_uses: u64,
    /// Total successful uses
    pub total_successes: u64,
}

impl RecipeStatsTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a recipe
    pub fn register(&mut self, stats: RecipeStats) {
        *self.by_category.entry(stats.category).or_default() += 1;
        *self.by_origin.entry(stats.origin).or_default() += 1;
        self.recipes.insert(stats.id.clone(), stats);
    }

    /// Record recipe usage
    pub fn record_use(&mut self, recipe_id: &str, success: bool, confidence: f64, timestamp: u64) {
        self.total_uses += 1;
        if success {
            self.total_successes += 1;
        }

        if let Some(stats) = self.recipes.get_mut(recipe_id) {
            stats.record_use(success, confidence, timestamp);
        }
    }

    /// Get total recipe count
    pub fn total_recipes(&self) -> usize {
        self.recipes.len()
    }

    /// Get learned recipe count (non-seed)
    pub fn learned_count(&self) -> u64 {
        self.by_origin.get(&RecipeOriginType::Learned).copied().unwrap_or(0)
            + self.by_origin.get(&RecipeOriginType::UserTaught).copied().unwrap_or(0)
    }

    /// Get seed recipe count
    pub fn seed_count(&self) -> u64 {
        self.by_origin.get(&RecipeOriginType::Seed).copied().unwrap_or(0)
    }

    /// Get overall success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_uses == 0 {
            0.0
        } else {
            self.total_successes as f64 / self.total_uses as f64 * 100.0
        }
    }

    /// Get most used recipes
    pub fn most_used(&self, limit: usize) -> Vec<&RecipeStats> {
        let mut sorted: Vec<_> = self.recipes.values().collect();
        sorted.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        sorted.into_iter().take(limit).collect()
    }

    /// Get recipes by category
    pub fn recipes_by_category(&self, category: RecipeCategory) -> Vec<&RecipeStats> {
        self.recipes
            .values()
            .filter(|r| r.category == category)
            .collect()
    }

    /// Get most reliable recipes (highest success rate, min 5 uses)
    pub fn most_reliable(&self, limit: usize) -> Vec<&RecipeStats> {
        let mut sorted: Vec<_> = self.recipes
            .values()
            .filter(|r| r.use_count >= 5)
            .collect();
        sorted.sort_by(|a, b| {
            b.success_rate()
                .partial_cmp(&a.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(limit).collect()
    }

    /// Get recently learned recipes
    pub fn recently_learned(&self, limit: usize) -> Vec<&RecipeStats> {
        let mut sorted: Vec<_> = self.recipes
            .values()
            .filter(|r| r.origin != RecipeOriginType::Seed)
            .collect();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sorted.into_iter().take(limit).collect()
    }

    /// Get top category
    pub fn top_category(&self) -> Option<(RecipeCategory, u64)> {
        self.by_category
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&cat, &count)| (cat, count))
    }

    /// Generate summary
    pub fn summary(&self) -> RecipeStatsSummary {
        RecipeStatsSummary {
            total: self.total_recipes(),
            seed: self.seed_count(),
            learned: self.learned_count(),
            total_uses: self.total_uses,
            success_rate: self.success_rate(),
            top_category: self.top_category().map(|(c, _)| c.display_name().to_string()),
        }
    }
}

/// Recipe stats summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStatsSummary {
    /// Total recipes
    pub total: usize,
    /// Seed recipes
    pub seed: u64,
    /// Learned recipes
    pub learned: u64,
    /// Total uses
    pub total_uses: u64,
    /// Success rate
    pub success_rate: f64,
    /// Top category
    pub top_category: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_register() {
        let mut tracker = RecipeStatsTracker::new();

        let stats = RecipeStats::new(
            "test",
            "Test",
            RecipeCategory::Package,
            RecipeOriginType::Learned,
            1000,
        );
        tracker.register(stats);

        assert_eq!(tracker.total_recipes(), 1);
        assert_eq!(tracker.learned_count(), 1);
    }

    #[test]
    fn test_tracker_record_use() {
        let mut tracker = RecipeStatsTracker::new();

        let stats = RecipeStats::new(
            "test",
            "Test",
            RecipeCategory::Package,
            RecipeOriginType::Seed,
            1000,
        );
        tracker.register(stats);
        tracker.record_use("test", true, 0.95, 2000);

        assert_eq!(tracker.total_uses, 1);
        assert_eq!(tracker.total_successes, 1);
    }

    #[test]
    fn test_most_used() {
        let mut tracker = RecipeStatsTracker::new();

        let r1 = RecipeStats::new("r1", "Recipe 1", RecipeCategory::Package, RecipeOriginType::Seed, 1000);
        let r2 = RecipeStats::new("r2", "Recipe 2", RecipeCategory::Service, RecipeOriginType::Seed, 1000);
        tracker.register(r1);
        tracker.register(r2);

        // Use r1 more
        for _ in 0..5 {
            tracker.record_use("r1", true, 0.9, 2000);
        }
        tracker.record_use("r2", true, 0.9, 2000);

        let most = tracker.most_used(1);
        assert_eq!(most[0].id, "r1");
    }

    #[test]
    fn test_by_category() {
        let mut tracker = RecipeStatsTracker::new();

        tracker.register(RecipeStats::new("p1", "Pkg 1", RecipeCategory::Package, RecipeOriginType::Seed, 1000));
        tracker.register(RecipeStats::new("p2", "Pkg 2", RecipeCategory::Package, RecipeOriginType::Seed, 1000));
        tracker.register(RecipeStats::new("s1", "Svc 1", RecipeCategory::Service, RecipeOriginType::Seed, 1000));

        let pkg = tracker.recipes_by_category(RecipeCategory::Package);
        assert_eq!(pkg.len(), 2);
    }

    #[test]
    fn test_top_category() {
        let mut tracker = RecipeStatsTracker::new();

        tracker.register(RecipeStats::new("p1", "Pkg", RecipeCategory::Package, RecipeOriginType::Seed, 1000));
        tracker.register(RecipeStats::new("p2", "Pkg", RecipeCategory::Package, RecipeOriginType::Seed, 1000));
        tracker.register(RecipeStats::new("s1", "Svc", RecipeCategory::Service, RecipeOriginType::Seed, 1000));

        let (cat, count) = tracker.top_category().unwrap();
        assert_eq!(cat, RecipeCategory::Package);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_summary() {
        let mut tracker = RecipeStatsTracker::new();

        tracker.register(RecipeStats::new("seed", "Seed", RecipeCategory::Package, RecipeOriginType::Seed, 1000));
        tracker.register(RecipeStats::new("learned", "Learned", RecipeCategory::Service, RecipeOriginType::Learned, 1000));

        let summary = tracker.summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.seed, 1);
        assert_eq!(summary.learned, 1);
    }
}
