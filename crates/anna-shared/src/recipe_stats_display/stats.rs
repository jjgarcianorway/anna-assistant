//! Individual recipe statistics.

use serde::{Deserialize, Serialize};

use super::category::RecipeCategory;
use super::origin::RecipeOriginType;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
