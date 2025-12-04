//! Recipe Engine v0.0.75
//!
//! Enhanced recipe matching and lifecycle management:
//! - Intent-pattern matching on canonical translator fields
//! - Precondition validation before recipe use
//! - Success/failure tracking with automatic demotion
//! - Recipe coverage statistics for status display
//!
//! Storage: /var/lib/anna/recipes/

use crate::case_engine::IntentType;
use crate::recipes::{Recipe, RecipeManager, RecipeRiskLevel, RecipeStatus, RECIPES_DIR};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Recipe engine state file
pub const RECIPE_ENGINE_STATE_FILE: &str = "/var/lib/anna/internal/recipe_engine_state.json";

/// Minimum match score to consider a recipe
pub const MIN_MATCH_SCORE: f64 = 0.4;

/// Consecutive failures before demotion
pub const DEMOTION_FAILURE_THRESHOLD: u64 = 3;

/// Reliability thresholds by risk level
pub const MIN_RELIABILITY_READ_ONLY: u8 = 90;
pub const MIN_RELIABILITY_DOCTOR: u8 = 80;
pub const MIN_RELIABILITY_MUTATION: u8 = 95;

/// Minimum evidence count by type
pub const MIN_EVIDENCE_READ_ONLY: usize = 1;
pub const MIN_EVIDENCE_DOCTOR: usize = 2;
pub const MIN_EVIDENCE_MUTATION: usize = 3;

/// Recipe matching result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMatch {
    /// Matched recipe
    pub recipe_id: String,
    /// Recipe name
    pub name: String,
    /// Match score (0.0 to 1.0)
    pub score: f64,
    /// Whether preconditions are satisfied
    pub preconditions_met: bool,
    /// Reason if preconditions not met
    pub precondition_failure: Option<String>,
    /// Recommended: use this recipe
    pub recommended: bool,
}

/// Recipe creation gate result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeGate {
    /// Whether recipe can be created
    pub can_create: bool,
    /// Status to assign
    pub status: RecipeStatus,
    /// Reason for decision
    pub reason: String,
}

/// Recipe engine statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeEngineStats {
    /// Total recipe matches attempted
    pub match_attempts: u64,
    /// Successful recipe uses
    pub recipe_uses: u64,
    /// Recipe use failures
    pub recipe_failures: u64,
    /// Recipes created
    pub recipes_created: u64,
    /// Recipes demoted
    pub recipes_demoted: u64,
    /// Coverage: % of requests that matched a recipe
    pub coverage_percent: f64,
    /// By-domain stats
    pub domain_stats: HashMap<String, DomainRecipeStats>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Per-domain recipe statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainRecipeStats {
    pub active_recipes: usize,
    pub total_uses: u64,
    pub success_rate: f64,
}

/// Recipe engine state (persisted)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeEngineState {
    /// Running statistics
    pub stats: RecipeEngineStats,
    /// Recent recipe uses for tracking
    pub recent_uses: Vec<RecipeUseRecord>,
    /// Rolling window for coverage calculation
    pub rolling_requests: u64,
    pub rolling_matches: u64,
    /// Schema version
    pub schema_version: u32,
}

/// Record of a recipe use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeUseRecord {
    pub recipe_id: String,
    pub case_id: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub reliability_score: u8,
}

impl RecipeEngineState {
    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(RECIPE_ENGINE_STATE_FILE) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self {
                schema_version: 1,
                ..Default::default()
            }
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(RECIPE_ENGINE_STATE_FILE).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(RECIPE_ENGINE_STATE_FILE, json)
    }

    /// Update coverage percentage
    pub fn update_coverage(&mut self) {
        if self.rolling_requests > 0 {
            self.stats.coverage_percent =
                (self.rolling_matches as f64 / self.rolling_requests as f64) * 100.0;
        }
    }
}

/// Recipe Engine for matching and lifecycle management
pub struct RecipeEngine;

impl RecipeEngine {
    /// Find matching recipes for a request using canonical translator fields
    pub fn find_matches(
        intent: IntentType,
        targets: &[String],
        confidence: u8,
        tools_planned: &[String],
        doctor_id: Option<&str>,
    ) -> Vec<RecipeMatch> {
        let recipes = RecipeManager::get_all();
        let mut matches = Vec::new();

        for recipe in recipes {
            // Only match active recipes
            if recipe.status != RecipeStatus::Active {
                continue;
            }

            let score = Self::calculate_match_score(&recipe, intent, targets, tools_planned);

            if score >= MIN_MATCH_SCORE {
                let (preconditions_met, precondition_failure) =
                    Self::check_preconditions(&recipe, doctor_id);

                matches.push(RecipeMatch {
                    recipe_id: recipe.id.clone(),
                    name: recipe.name.clone(),
                    score,
                    preconditions_met,
                    precondition_failure,
                    recommended: preconditions_met && score >= 0.6,
                });
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches
    }

    /// Calculate match score using canonical fields
    fn calculate_match_score(
        recipe: &Recipe,
        intent: IntentType,
        targets: &[String],
        tools_planned: &[String],
    ) -> f64 {
        let mut score = 0.0;

        // Intent type match (30%)
        let intent_str = intent.to_string().to_lowercase().replace('_', "");
        let recipe_intent = recipe
            .intent_pattern
            .intent_type
            .to_lowercase()
            .replace('_', "");
        if intent_str == recipe_intent {
            score += 0.30;
        }

        // Target match (25%)
        if !recipe.intent_pattern.targets.is_empty() && !targets.is_empty() {
            let matched_targets = recipe
                .intent_pattern
                .targets
                .iter()
                .filter(|t| targets.iter().any(|rt| rt.eq_ignore_ascii_case(t)))
                .count();
            let target_ratio = matched_targets as f64 / recipe.intent_pattern.targets.len() as f64;
            score += 0.25 * target_ratio;
        }

        // Tool plan overlap (25%)
        if !recipe.tool_plan.steps.is_empty() && !tools_planned.is_empty() {
            let recipe_tools: Vec<_> = recipe
                .tool_plan
                .steps
                .iter()
                .map(|s| &s.tool_name)
                .collect();
            let matched_tools = recipe_tools
                .iter()
                .filter(|t| tools_planned.iter().any(|pt| pt == **t))
                .count();
            let tool_ratio = matched_tools as f64 / recipe_tools.len() as f64;
            score += 0.25 * tool_ratio;
        }

        // Confidence bonus from recipe (20%)
        score += 0.20 * recipe.confidence;

        score.min(1.0)
    }

    /// Check if recipe preconditions are satisfied
    fn check_preconditions(recipe: &Recipe, doctor_id: Option<&str>) -> (bool, Option<String>) {
        // Check each precondition
        for precondition in &recipe.preconditions {
            match &precondition.check_type {
                crate::recipes::PreconditionCheck::PackageInstalled { name } => {
                    if !Self::is_package_installed(name) {
                        return (false, Some(format!("Package '{}' not installed", name)));
                    }
                }
                crate::recipes::PreconditionCheck::ServiceRunning { name } => {
                    if !Self::is_service_running(name) {
                        return (false, Some(format!("Service '{}' not running", name)));
                    }
                }
                crate::recipes::PreconditionCheck::FileExists { path } => {
                    if !Path::new(path).exists() {
                        return (false, Some(format!("File '{}' not found", path)));
                    }
                }
                crate::recipes::PreconditionCheck::CommandSucceeds { command } => {
                    if !Self::command_succeeds(command) {
                        return (false, Some(format!("Command '{}' failed", command)));
                    }
                }
            }
        }

        // Check if doctor matches (if recipe was from a specific doctor)
        if let Some(origin_case) = &recipe.origin_case_id {
            if origin_case.contains("doctor") {
                if let Some(required_doctor) = recipe.tags.iter().find(|t| t.starts_with("doctor:"))
                {
                    let required = required_doctor.strip_prefix("doctor:").unwrap_or("");
                    if let Some(current) = doctor_id {
                        if !current.contains(required) {
                            return (false, Some(format!("Requires doctor '{}'", required)));
                        }
                    }
                }
            }
        }

        (true, None)
    }

    /// Check if package is installed (quick check)
    fn is_package_installed(name: &str) -> bool {
        std::process::Command::new("pacman")
            .args(["-Q", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if service is running (quick check)
    fn is_service_running(name: &str) -> bool {
        std::process::Command::new("systemctl")
            .args(["is-active", "--quiet", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if command succeeds (quick check)
    fn command_succeeds(command: &str) -> bool {
        std::process::Command::new("sh")
            .args(["-c", command])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Determine if a recipe should be created from a completed case
    pub fn check_creation_gate(
        risk_level: RecipeRiskLevel,
        reliability_score: u8,
        evidence_count: usize,
        is_doctor_case: bool,
    ) -> RecipeGate {
        // Get thresholds based on risk level
        let (min_reliability, min_evidence) = match risk_level {
            RecipeRiskLevel::ReadOnly => {
                if is_doctor_case {
                    (MIN_RELIABILITY_DOCTOR, MIN_EVIDENCE_DOCTOR)
                } else {
                    (MIN_RELIABILITY_READ_ONLY, MIN_EVIDENCE_READ_ONLY)
                }
            }
            RecipeRiskLevel::LowRisk | RecipeRiskLevel::MediumRisk | RecipeRiskLevel::HighRisk => {
                (MIN_RELIABILITY_MUTATION, MIN_EVIDENCE_MUTATION)
            }
        };

        // Check evidence count
        if evidence_count < min_evidence {
            return RecipeGate {
                can_create: false,
                status: RecipeStatus::Draft,
                reason: format!(
                    "Need {} evidence items, only have {}",
                    min_evidence, evidence_count
                ),
            };
        }

        // Check reliability
        if reliability_score < min_reliability {
            return RecipeGate {
                can_create: true,
                status: RecipeStatus::Draft,
                reason: format!(
                    "Reliability {}% < {}% threshold, creating as draft",
                    reliability_score, min_reliability
                ),
            };
        }

        // All gates passed
        RecipeGate {
            can_create: true,
            status: RecipeStatus::Active,
            reason: format!(
                "Reliability {}% >= {}% with {} evidence",
                reliability_score, min_reliability, evidence_count
            ),
        }
    }

    /// Record a recipe use and update stats
    pub fn record_use(
        recipe_id: &str,
        case_id: &str,
        success: bool,
        reliability_score: u8,
    ) -> std::io::Result<()> {
        // Update recipe success/failure count
        if let Some(mut recipe) = Recipe::load(recipe_id) {
            if success {
                recipe.record_success();
            } else {
                recipe.record_failure();

                // Check for demotion
                if recipe.failure_count >= DEMOTION_FAILURE_THRESHOLD
                    && recipe.status == RecipeStatus::Active
                {
                    recipe.status = RecipeStatus::Draft;
                    recipe.notes.push_str(&format!(
                        "\n[{}] Demoted due to {} consecutive failures",
                        Utc::now().format("%Y-%m-%d"),
                        recipe.failure_count
                    ));
                }
            }
            recipe.save()?;
        }

        // Update engine state
        let mut state = RecipeEngineState::load();
        state.recent_uses.push(RecipeUseRecord {
            recipe_id: recipe_id.to_string(),
            case_id: case_id.to_string(),
            timestamp: Utc::now(),
            success,
            reliability_score,
        });

        // Keep only last 100 uses
        if state.recent_uses.len() > 100 {
            state.recent_uses.drain(0..(state.recent_uses.len() - 100));
        }

        // Update stats
        state.rolling_requests += 1;
        state.rolling_matches += 1;
        if success {
            state.stats.recipe_uses += 1;
        } else {
            state.stats.recipe_failures += 1;
        }
        state.stats.updated_at = Utc::now();
        state.update_coverage();

        state.save()
    }

    /// Record that a request was processed (for coverage tracking)
    pub fn record_request(matched_recipe: bool) -> std::io::Result<()> {
        let mut state = RecipeEngineState::load();
        state.stats.match_attempts += 1;
        state.rolling_requests += 1;
        if matched_recipe {
            state.rolling_matches += 1;
        }
        state.update_coverage();
        state.stats.updated_at = Utc::now();
        state.save()
    }

    /// Get current engine statistics
    pub fn get_stats() -> RecipeEngineStats {
        let state = RecipeEngineState::load();
        let recipe_stats = RecipeManager::get_stats();

        let mut stats = state.stats;
        stats.recipes_created = recipe_stats.total_recipes as u64;

        // Build domain stats
        let recipes = RecipeManager::get_all();
        let mut domain_map: HashMap<String, DomainRecipeStats> = HashMap::new();

        for recipe in recipes {
            let domain = recipe
                .tags
                .iter()
                .find(|t| {
                    matches!(
                        t.as_str(),
                        "network"
                            | "audio"
                            | "storage"
                            | "boot"
                            | "graphics"
                            | "software"
                            | "other"
                    )
                })
                .cloned()
                .unwrap_or_else(|| "other".to_string());

            let entry = domain_map.entry(domain).or_default();
            if recipe.status == RecipeStatus::Active {
                entry.active_recipes += 1;
            }
            entry.total_uses += recipe.success_count;
            let total = recipe.success_count + recipe.failure_count;
            if total > 0 {
                entry.success_rate = recipe.success_count as f64 / total as f64;
            }
        }

        stats.domain_stats = domain_map;
        stats
    }

    /// Get recipe coverage percentage
    pub fn get_coverage_percent() -> f64 {
        let state = RecipeEngineState::load();
        state.stats.coverage_percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_gate_read_only() {
        let gate = RecipeEngine::check_creation_gate(RecipeRiskLevel::ReadOnly, 92, 2, false);
        assert!(gate.can_create);
        assert_eq!(gate.status, RecipeStatus::Active);
    }

    #[test]
    fn test_creation_gate_low_reliability() {
        let gate = RecipeEngine::check_creation_gate(RecipeRiskLevel::ReadOnly, 85, 2, false);
        assert!(gate.can_create);
        assert_eq!(gate.status, RecipeStatus::Draft);
    }

    #[test]
    fn test_creation_gate_insufficient_evidence() {
        let gate = RecipeEngine::check_creation_gate(RecipeRiskLevel::MediumRisk, 96, 2, false);
        assert!(!gate.can_create);
    }

    #[test]
    fn test_creation_gate_mutation() {
        let gate = RecipeEngine::check_creation_gate(RecipeRiskLevel::MediumRisk, 96, 3, false);
        assert!(gate.can_create);
        assert_eq!(gate.status, RecipeStatus::Active);
    }

    #[test]
    fn test_creation_gate_doctor() {
        let gate = RecipeEngine::check_creation_gate(RecipeRiskLevel::ReadOnly, 82, 2, true);
        assert!(gate.can_create);
        assert_eq!(gate.status, RecipeStatus::Active);
    }
}
