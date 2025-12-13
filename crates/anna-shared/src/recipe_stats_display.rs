//! Recipe Statistics Display (v0.0.490).
//!
//! Provides statistics on learned and created recipes.
//! Shows categorization and usage patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recipe category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecipeCategory {
    /// Package management
    Package,
    /// Service management
    Service,
    /// Configuration files
    Config,
    /// System monitoring
    System,
    /// Network operations
    Network,
    /// Storage management
    Storage,
    /// Docker/containers
    Container,
    /// Git operations
    Git,
    /// Editor configuration
    Editor,
    /// Security-related
    Security,
    /// Custom/other
    Custom,
}

impl RecipeCategory {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Package => "Package",
            Self::Service => "Service",
            Self::Config => "Config",
            Self::System => "System",
            Self::Network => "Network",
            Self::Storage => "Storage",
            Self::Container => "Container",
            Self::Git => "Git",
            Self::Editor => "Editor",
            Self::Security => "Security",
            Self::Custom => "Custom",
        }
    }

    /// All categories
    pub fn all() -> Vec<Self> {
        vec![
            Self::Package,
            Self::Service,
            Self::Config,
            Self::System,
            Self::Network,
            Self::Storage,
            Self::Container,
            Self::Git,
            Self::Editor,
            Self::Security,
            Self::Custom,
        ]
    }
}

/// Recipe origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecipeOriginType {
    /// Built-in seed recipe
    Seed,
    /// Learned from specialist
    Learned,
    /// Learned from user interaction
    UserTaught,
    /// Imported from file
    Imported,
}

impl RecipeOriginType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Seed => "Built-in",
            Self::Learned => "Learned",
            Self::UserTaught => "User Taught",
            Self::Imported => "Imported",
        }
    }
}

/// Statistics for a single recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeStats {
    /// Recipe ID
    pub id: String,
    /// Recipe name
    pub name: String,
    /// Category
    pub category: RecipeCategory,
    /// Origin
    pub origin: RecipeOriginType,
    /// Times used
    pub use_count: u64,
    /// Success count
    pub success_count: u64,
    /// Average confidence when matched
    pub avg_confidence: f64,
    /// Last used timestamp
    pub last_used: Option<u64>,
    /// Created timestamp
    pub created_at: u64,
}

impl RecipeStats {
    /// Create new recipe stats
    pub fn new(id: &str, name: &str, category: RecipeCategory, origin: RecipeOriginType, created_at: u64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            origin,
            use_count: 0,
            success_count: 0,
            avg_confidence: 0.0,
            last_used: None,
            created_at,
        }
    }

    /// Record usage
    pub fn record_use(&mut self, success: bool, confidence: f64, timestamp: u64) {
        self.use_count += 1;
        if success {
            self.success_count += 1;
        }
        self.last_used = Some(timestamp);

        // Update rolling average
        let prev_total = self.avg_confidence * (self.use_count - 1) as f64;
        self.avg_confidence = (prev_total + confidence) / self.use_count as f64;
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.use_count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.use_count as f64 * 100.0
        }
    }
}

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

impl RecipeStatsTracker {
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

/// Format recipe stats for display
pub fn format_recipe_stats(tracker: &RecipeStatsTracker) -> String {
    let mut output = String::new();

    output.push_str("Recipe Statistics\n");
    output.push_str("══════════════════════════════════════\n\n");

    if tracker.total_recipes() == 0 {
        output.push_str("No recipes registered yet.\n");
        return output;
    }

    output.push_str(&format!(
        "Total Recipes: {} ({} seed, {} learned)\n",
        tracker.total_recipes(),
        tracker.seed_count(),
        tracker.learned_count()
    ));
    output.push_str(&format!(
        "Total Uses:    {} ({:.1}% success)\n\n",
        tracker.total_uses,
        tracker.success_rate()
    ));

    output.push_str("By Category:\n");
    for category in RecipeCategory::all() {
        let count = tracker.by_category.get(&category).copied().unwrap_or(0);
        if count > 0 {
            output.push_str(&format!("  {}: {}\n", category.display_name(), count));
        }
    }

    let most_used = tracker.most_used(3);
    if !most_used.is_empty() {
        output.push_str("\nMost Used:\n");
        for recipe in most_used {
            output.push_str(&format!(
                "  {} - {} uses ({:.0}% success)\n",
                recipe.name,
                recipe.use_count,
                recipe.success_rate()
            ));
        }
    }

    output
}

/// Format compact recipe stats
pub fn format_recipe_stats_compact(tracker: &RecipeStatsTracker) -> String {
    if tracker.total_recipes() == 0 {
        return "No recipes yet".to_string();
    }

    format!(
        "{} recipes ({} learned), {} uses, {:.0}% success",
        tracker.total_recipes(),
        tracker.learned_count(),
        tracker.total_uses,
        tracker.success_rate()
    )
}

/// Generate fun fact about recipes
pub fn recipe_stats_fun_fact(tracker: &RecipeStatsTracker) -> Option<String> {
    if tracker.total_recipes() < 3 {
        return None;
    }

    let facts = vec![
        format!(
            "Anna has learned {} recipes beyond the basics - knowledge grows!",
            tracker.learned_count()
        ),
        format!(
            "Recipes have been used {} times with {:.0}% success!",
            tracker.total_uses,
            tracker.success_rate()
        ),
        tracker
            .top_category()
            .map(|(cat, count)| {
                format!(
                    "{} is the most popular category with {} recipes!",
                    cat.display_name(),
                    count
                )
            })
            .unwrap_or_else(|| "Diverse recipe collection!".to_string()),
        tracker
            .most_used(1)
            .first()
            .map(|r| format!("\"{}\" is the most popular recipe with {} uses!", r.name, r.use_count))
            .unwrap_or_else(|| "All recipes contribute to success!".to_string()),
    ];

    let index = (tracker.total_uses as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about recipe stats
pub fn is_recipe_stats_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "recipe stats",
        "recipes learned",
        "how many recipes",
        "recipe count",
        "most used recipe",
        "popular recipes",
        "recipe categories",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_display() {
        assert_eq!(RecipeCategory::Package.display_name(), "Package");
        assert_eq!(RecipeCategory::Container.display_name(), "Container");
    }

    #[test]
    fn test_origin_display() {
        assert_eq!(RecipeOriginType::Seed.display_name(), "Built-in");
        assert_eq!(RecipeOriginType::Learned.display_name(), "Learned");
    }

    #[test]
    fn test_recipe_stats_new() {
        let stats = RecipeStats::new(
            "pkg-install",
            "Package Install",
            RecipeCategory::Package,
            RecipeOriginType::Seed,
            1000,
        );
        assert_eq!(stats.id, "pkg-install");
        assert_eq!(stats.use_count, 0);
    }

    #[test]
    fn test_record_use() {
        let mut stats = RecipeStats::new(
            "test",
            "Test",
            RecipeCategory::Custom,
            RecipeOriginType::Seed,
            1000,
        );

        stats.record_use(true, 0.9, 2000);
        stats.record_use(false, 0.7, 3000);

        assert_eq!(stats.use_count, 2);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }

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

    #[test]
    fn test_format_compact() {
        let mut tracker = RecipeStatsTracker::new();

        tracker.register(RecipeStats::new("test", "Test", RecipeCategory::Package, RecipeOriginType::Seed, 1000));
        tracker.record_use("test", true, 0.9, 2000);

        let output = format_recipe_stats_compact(&tracker);
        assert!(output.contains("1 recipes"));
    }

    #[test]
    fn test_fun_fact() {
        let mut tracker = RecipeStatsTracker::new();

        for i in 0..5 {
            tracker.register(RecipeStats::new(
                &format!("r{}", i),
                &format!("Recipe {}", i),
                RecipeCategory::Package,
                RecipeOriginType::Learned,
                1000 + i as u64,
            ));
        }

        let fact = recipe_stats_fun_fact(&tracker);
        assert!(fact.is_some());
    }

    #[test]
    fn test_is_recipe_stats_query() {
        assert!(is_recipe_stats_query("show recipe stats"));
        assert!(is_recipe_stats_query("how many recipes learned"));
        assert!(is_recipe_stats_query("most used recipe"));

        assert!(!is_recipe_stats_query("install vim"));
        assert!(!is_recipe_stats_query("status"));
    }
}
