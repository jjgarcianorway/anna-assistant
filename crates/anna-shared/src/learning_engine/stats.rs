//! Learning statistics for learning engine (v0.0.427).
//!
//! Tracks recipe usage, hit rates, and learning progress.

use super::RecipeLibrary;
use serde::{Deserialize, Serialize};

/// Learning statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningStats {
    /// Total recipes in library
    pub recipes_total: usize,
    /// Recipes created in last 7 days
    pub recipes_recent: usize,
    /// Seed recipes count
    pub seed_recipes: usize,
    /// Learned recipes count
    pub learned_recipes: usize,
    /// Total tickets processed
    pub tickets_processed: usize,
    /// Tickets resolved by recipes (no LLM)
    pub recipe_hits: usize,
    /// Tickets that required LLM
    pub llm_fallbacks: usize,
    /// Recipe hit rate (percentage)
    pub recipe_hit_rate: f32,
    /// Evidence cache entries
    pub evidence_entries: usize,
    /// Average recipe success rate
    pub avg_recipe_success_rate: f32,
    /// Top recipes by usage
    pub top_recipes: Vec<RecipeUsageSummary>,
}

/// Summary of a recipe's usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeUsageSummary {
    /// Recipe ID
    pub id: String,
    /// Domain
    pub domain: String,
    /// Total uses
    pub uses: u32,
    /// Successes
    pub successes: u32,
    /// Success rate
    pub success_rate: f32,
}

impl LearningStats {
    /// Compute stats from a recipe library
    pub fn compute(library: &RecipeLibrary, tickets_processed: usize, recipe_hits: usize) -> Self {
        let recipes = library.all();
        let seeds = library.seeds();
        let learned = library.learned();
        let recent = library.recent(7);

        // Calculate average success rate
        let total_uses: u32 = recipes.iter().map(|r| r.stats.uses).sum();
        let total_successes: u32 = recipes.iter().map(|r| r.stats.successes).sum();
        let avg_success_rate = if total_uses > 0 {
            total_successes as f32 / total_uses as f32 * 100.0
        } else {
            0.0
        };

        // Get top recipes by usage
        let mut top_recipes: Vec<RecipeUsageSummary> = recipes
            .iter()
            .filter(|r| r.stats.uses > 0)
            .map(|r| RecipeUsageSummary {
                id: r.id.clone(),
                domain: r.domain.clone(),
                uses: r.stats.uses,
                successes: r.stats.successes,
                success_rate: r.stats.success_rate() * 100.0,
            })
            .collect();
        top_recipes.sort_by(|a, b| b.uses.cmp(&a.uses));
        top_recipes.truncate(10);

        // Calculate hit rate
        let llm_fallbacks = tickets_processed.saturating_sub(recipe_hits);
        let hit_rate = if tickets_processed > 0 {
            recipe_hits as f32 / tickets_processed as f32 * 100.0
        } else {
            0.0
        };

        Self {
            recipes_total: recipes.len(),
            recipes_recent: recent.len(),
            seed_recipes: seeds.len(),
            learned_recipes: learned.len(),
            tickets_processed,
            recipe_hits,
            llm_fallbacks,
            recipe_hit_rate: hit_rate,
            evidence_entries: 0, // Set separately
            avg_recipe_success_rate: avg_success_rate,
            top_recipes,
        }
    }

    /// Set evidence entries count
    pub fn with_evidence_count(mut self, count: usize) -> Self {
        self.evidence_entries = count;
        self
    }
}

impl std::fmt::Display for LearningStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[learning]")?;
        writeln!(f, "  recipes_total         {}", self.recipes_total)?;
        writeln!(
            f,
            "  recipes_recent        {} (last 7 days)",
            self.recipes_recent
        )?;
        writeln!(f, "  seed_recipes          {}", self.seed_recipes)?;
        writeln!(f, "  learned_recipes       {}", self.learned_recipes)?;
        writeln!(
            f,
            "  recipe_hit_rate       {:.0}%   (tickets answered by recipes)",
            self.recipe_hit_rate
        )?;
        writeln!(
            f,
            "  evidence_cache        {} entries",
            self.evidence_entries
        )?;
        writeln!(f)?;

        if !self.top_recipes.is_empty() {
            writeln!(f, "[recipes]")?;
            for recipe in &self.top_recipes {
                writeln!(
                    f,
                    "  {:<30}  uses: {:<4}  success: {:<4}  domain: {}",
                    recipe.id, recipe.uses, recipe.successes, recipe.domain
                )?;
            }
        }

        Ok(())
    }
}

/// Learning progress tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningProgress {
    /// Tickets since last recipe learned
    pub tickets_since_last_learned: usize,
    /// Last learned recipe ID
    pub last_learned_recipe: Option<String>,
    /// Last learned timestamp
    pub last_learned_at: Option<String>,
    /// Eligible tickets skipped (and why)
    pub skipped_tickets: Vec<SkippedTicket>,
    /// Total recipes learned ever
    pub total_learned: usize,
}

/// A ticket that was eligible but skipped for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedTicket {
    /// Ticket ID
    pub ticket_id: String,
    /// Skip reason
    pub reason: String,
    /// Timestamp
    pub at: String,
}

impl LearningProgress {
    /// Record a learned recipe
    pub fn record_learned(&mut self, recipe_id: &str) {
        self.tickets_since_last_learned = 0;
        self.last_learned_recipe = Some(recipe_id.to_string());
        self.last_learned_at = Some(now_iso8601());
        self.total_learned += 1;
    }

    /// Record a processed ticket
    pub fn record_ticket(&mut self) {
        self.tickets_since_last_learned += 1;
    }

    /// Record a skipped ticket
    pub fn record_skipped(&mut self, ticket_id: &str, reason: &str) {
        // Keep last 20 skipped tickets
        if self.skipped_tickets.len() >= 20 {
            self.skipped_tickets.remove(0);
        }

        self.skipped_tickets.push(SkippedTicket {
            ticket_id: ticket_id.to_string(),
            reason: reason.to_string(),
            at: now_iso8601(),
        });
    }
}

/// Counter for recipe vs LLM resolution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolutionCounter {
    /// Tickets resolved by recipe
    pub by_recipe: usize,
    /// Tickets resolved by LLM
    pub by_llm: usize,
    /// Tickets failed
    pub failed: usize,
}

impl ResolutionCounter {
    /// Record a recipe resolution
    pub fn record_recipe(&mut self) {
        self.by_recipe += 1;
    }

    /// Record an LLM resolution
    pub fn record_llm(&mut self) {
        self.by_llm += 1;
    }

    /// Record a failure
    pub fn record_failure(&mut self) {
        self.failed += 1;
    }

    /// Get total
    pub fn total(&self) -> usize {
        self.by_recipe + self.by_llm + self.failed
    }

    /// Get recipe hit rate
    pub fn recipe_hit_rate(&self) -> f32 {
        let total = self.by_recipe + self.by_llm;
        if total == 0 {
            0.0
        } else {
            self.by_recipe as f32 / total as f32 * 100.0
        }
    }
}

/// Get current timestamp as ISO 8601 string
fn now_iso8601() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning_engine::{LearnedRecipe, RecipeOrigin, RecipePattern};

    fn make_recipe(id: &str, uses: u32, successes: u32, is_seed: bool) -> LearnedRecipe {
        let mut recipe = LearnedRecipe::new(id, "test").with_pattern(RecipePattern::new("test"));
        recipe.stats.uses = uses;
        recipe.stats.successes = successes;
        recipe.origin.is_seed = is_seed;
        recipe
    }

    #[test]
    fn test_learning_stats_compute() {
        let mut library = RecipeLibrary::new();
        library.add(make_recipe("seed-1", 10, 9, true)).unwrap();
        library.add(make_recipe("learned-1", 5, 4, false)).unwrap();
        library.add(make_recipe("learned-2", 3, 3, false)).unwrap();

        let stats = LearningStats::compute(&library, 100, 38);

        assert_eq!(stats.recipes_total, 3);
        assert_eq!(stats.seed_recipes, 1);
        assert_eq!(stats.learned_recipes, 2);
        assert_eq!(stats.recipe_hit_rate, 38.0);
    }

    #[test]
    fn test_resolution_counter() {
        let mut counter = ResolutionCounter::default();
        counter.record_recipe();
        counter.record_recipe();
        counter.record_llm();

        assert_eq!(counter.by_recipe, 2);
        assert_eq!(counter.by_llm, 1);
        assert!((counter.recipe_hit_rate() - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_learning_progress() {
        let mut progress = LearningProgress::default();

        progress.record_ticket();
        progress.record_ticket();
        assert_eq!(progress.tickets_since_last_learned, 2);

        progress.record_learned("new-recipe");
        assert_eq!(progress.tickets_since_last_learned, 0);
        assert_eq!(progress.total_learned, 1);
    }

    #[test]
    fn test_stats_display() {
        let stats = LearningStats {
            recipes_total: 15,
            recipes_recent: 3,
            seed_recipes: 5,
            learned_recipes: 10,
            recipe_hit_rate: 42.0,
            evidence_entries: 150,
            top_recipes: vec![RecipeUsageSummary {
                id: "check-ram".to_string(),
                domain: "memory".to_string(),
                uses: 20,
                successes: 18,
                success_rate: 90.0,
            }],
            ..Default::default()
        };

        let display = format!("{}", stats);
        assert!(display.contains("recipes_total         15"));
        assert!(display.contains("recipe_hit_rate       42%"));
        assert!(display.contains("check-ram"));
    }
}
