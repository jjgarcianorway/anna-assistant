//! Recipe Stats (v0.0.416).
//!
//! Extends honest stats tracking to include recipe usage.
//!
//! Tracks:
//! - Tickets resolved by recipes (no LLM)
//! - Tickets resolved by specialists (LLM)
//! - Recipe coverage by intent
//! - Learning progress

use crate::canonical_intents::CanonicalIntent;
use crate::recipe_fast_path::ResponseSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Recipe-aware stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStats {
    /// Total tickets processed
    pub total_tickets: u32,
    /// Resolved by recipes (no LLM)
    pub resolved_by_recipe: u32,
    /// Resolved by specialists (LLM)
    pub resolved_by_specialist: u32,
    /// Failed tickets
    pub failed_tickets: u32,
    /// Stats by intent
    pub by_intent: HashMap<String, IntentStats>,
    /// Stats by recipe
    pub by_recipe: HashMap<String, SingleRecipeStats>,
    /// Learning events
    pub learning_events: u32,
    /// Last updated
    pub last_updated: u64,
}

/// Stats for a single intent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentStats {
    /// Total queries for this intent
    pub total: u32,
    /// Resolved by recipe
    pub recipe_resolved: u32,
    /// Resolved by specialist
    pub specialist_resolved: u32,
    /// Failed
    pub failed: u32,
    /// Has active recipe
    pub has_recipe: bool,
}

impl IntentStats {
    pub fn recipe_coverage(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.recipe_resolved as f32 / self.total as f32
        }
    }

    pub fn success_rate(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            (self.recipe_resolved + self.specialist_resolved) as f32 / self.total as f32
        }
    }
}

/// Stats for a single recipe
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SingleRecipeStats {
    pub recipe_id: String,
    pub uses: u32,
    pub successes: u32,
    pub failures: u32,
    pub avg_confidence: f32,
    pub last_used: u64,
}

impl SingleRecipeStats {
    pub fn success_rate(&self) -> f32 {
        if self.uses == 0 {
            1.0
        } else {
            self.successes as f32 / self.uses as f32
        }
    }
}

impl RecipeStats {
    /// Load from disk
    pub fn load() -> Self {
        let path = stats_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&mut self) -> Result<(), String> {
        self.last_updated = current_secs();
        let path = stats_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Record a ticket resolution
    pub fn record_resolution(
        &mut self,
        intent: CanonicalIntent,
        source: &ResponseSource,
        success: bool,
        confidence: f32,
    ) {
        self.total_tickets += 1;
        let intent_key = format!("{:?}", intent);

        // Update global stats
        match source {
            ResponseSource::Recipe { recipe_id } => {
                if success {
                    self.resolved_by_recipe += 1;
                } else {
                    self.failed_tickets += 1;
                }

                // Update recipe stats
                let recipe_stats =
                    self.by_recipe
                        .entry(recipe_id.clone())
                        .or_insert_with(|| SingleRecipeStats {
                            recipe_id: recipe_id.clone(),
                            ..Default::default()
                        });
                recipe_stats.uses += 1;
                if success {
                    recipe_stats.successes += 1;
                } else {
                    recipe_stats.failures += 1;
                }
                recipe_stats.avg_confidence =
                    (recipe_stats.avg_confidence * (recipe_stats.uses - 1) as f32 + confidence)
                        / recipe_stats.uses as f32;
                recipe_stats.last_used = current_secs();
            }
            ResponseSource::Specialist { .. } => {
                if success {
                    self.resolved_by_specialist += 1;
                } else {
                    self.failed_tickets += 1;
                }
            }
            ResponseSource::BuiltIn => {
                if success {
                    self.resolved_by_recipe += 1; // Count as recipe-like
                } else {
                    self.failed_tickets += 1;
                }
            }
        }

        // Update intent stats
        let intent_stats = self.by_intent.entry(intent_key).or_default();
        intent_stats.total += 1;
        match source {
            ResponseSource::Recipe { .. } | ResponseSource::BuiltIn => {
                if success {
                    intent_stats.recipe_resolved += 1;
                } else {
                    intent_stats.failed += 1;
                }
            }
            ResponseSource::Specialist { .. } => {
                if success {
                    intent_stats.specialist_resolved += 1;
                } else {
                    intent_stats.failed += 1;
                }
            }
        }
    }

    /// Record a learning event (new recipe created)
    pub fn record_learning_event(&mut self, intent: CanonicalIntent) {
        self.learning_events += 1;
        let intent_key = format!("{:?}", intent);
        if let Some(stats) = self.by_intent.get_mut(&intent_key) {
            stats.has_recipe = true;
        }
    }

    /// Get overall recipe coverage
    pub fn recipe_coverage(&self) -> f32 {
        if self.total_tickets == 0 {
            0.0
        } else {
            self.resolved_by_recipe as f32 / self.total_tickets as f32
        }
    }

    /// Get overall success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_tickets == 0 {
            1.0
        } else {
            (self.resolved_by_recipe + self.resolved_by_specialist) as f32
                / self.total_tickets as f32
        }
    }

    /// Get intents with best recipe coverage
    pub fn best_covered_intents(&self, limit: usize) -> Vec<(&str, f32)> {
        let mut items: Vec<_> = self
            .by_intent
            .iter()
            .filter(|(_, s)| s.has_recipe && s.total > 0)
            .map(|(k, s)| (k.as_str(), s.recipe_coverage()))
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(limit);
        items
    }

    /// Get intents that need recipes (high specialist usage)
    pub fn intents_needing_recipes(&self, limit: usize) -> Vec<(&str, u32)> {
        let mut items: Vec<_> = self
            .by_intent
            .iter()
            .filter(|(_, s)| !s.has_recipe && s.specialist_resolved > 0)
            .map(|(k, s)| (k.as_str(), s.specialist_resolved))
            .collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(limit);
        items
    }

    /// Generate summary string
    pub fn summary(&self) -> String {
        let coverage = self.recipe_coverage() * 100.0;
        let success = self.success_rate() * 100.0;

        format!(
            "Tickets: {} total, {:.0}% recipe coverage, {:.0}% success rate. {} recipes active, {} learned.",
            self.total_tickets,
            coverage,
            success,
            self.by_recipe.len(),
            self.learning_events
        )
    }
}

fn stats_path() -> PathBuf {
    let base = std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| "/var/lib/anna".to_string());
    PathBuf::from(base).join("recipe_stats.json")
}

fn current_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_stats() {
        let mut stats = IntentStats::default();
        stats.total = 10;
        stats.recipe_resolved = 6;
        stats.specialist_resolved = 3;
        stats.failed = 1;

        assert!((stats.recipe_coverage() - 0.6).abs() < 0.01);
        assert!((stats.success_rate() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_recipe_stats_recording() {
        let mut stats = RecipeStats::default();

        stats.record_resolution(
            CanonicalIntent::CheckDiskUsage,
            &ResponseSource::Recipe {
                recipe_id: "disk_v1".to_string(),
            },
            true,
            0.95,
        );

        assert_eq!(stats.total_tickets, 1);
        assert_eq!(stats.resolved_by_recipe, 1);
        assert_eq!(stats.resolved_by_specialist, 0);
    }

    #[test]
    fn test_coverage_calculation() {
        let mut stats = RecipeStats::default();

        // 3 by recipe, 2 by specialist
        for _ in 0..3 {
            stats.record_resolution(
                CanonicalIntent::CheckDiskUsage,
                &ResponseSource::Recipe {
                    recipe_id: "test".to_string(),
                },
                true,
                0.9,
            );
        }
        for _ in 0..2 {
            stats.record_resolution(
                CanonicalIntent::CheckFreeRam,
                &ResponseSource::Specialist {
                    specialist: "system".to_string(),
                },
                true,
                0.85,
            );
        }

        assert!((stats.recipe_coverage() - 0.6).abs() < 0.01);
        assert!((stats.success_rate() - 1.0).abs() < 0.01);
    }
}
