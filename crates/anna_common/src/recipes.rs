//! Recipe System v0.0.13
//!
//! Recipes are learned patterns that Anna can reuse:
//! - Named intent patterns with conditions
//! - Tool plan templates (read-only and/or mutation)
//! - Safety classification and required confirmations
//! - Validation checks (preconditions)
//! - Rollback templates (for mutations)
//! - Provenance tracking (who created, when)
//! - Confidence scoring and success counters
//!
//! Storage: /var/lib/anna/recipes/
//! - Each recipe is a separate JSON file
//! - recipe_index.json for fast lookups

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Recipe storage directory
pub const RECIPES_DIR: &str = "/var/lib/anna/recipes";
/// Recipe index file
pub const RECIPE_INDEX_FILE: &str = "/var/lib/anna/recipes/recipe_index.json";
/// Recipe archive directory
pub const RECIPE_ARCHIVE_DIR: &str = "/var/lib/anna/recipes/archive";

/// Schema version for recipes
pub const RECIPE_SCHEMA_VERSION: u32 = 1;

/// Evidence ID prefix for recipe entries
pub const RECIPE_EVIDENCE_PREFIX: &str = "RCP";

/// Minimum reliability score to create a recipe
pub const MIN_RELIABILITY_FOR_RECIPE: u32 = 80;

/// Generate a unique recipe ID
pub fn generate_recipe_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{}{:05}", RECIPE_EVIDENCE_PREFIX, ts % 100000)
}

/// A recipe - learned pattern for handling requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe ID (also used as evidence ID)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this recipe does
    pub description: String,
    /// Intent pattern (what kinds of requests this matches)
    pub intent_pattern: IntentPattern,
    /// Tool plan template
    pub tool_plan: RecipeToolPlan,
    /// Safety classification
    pub safety: RecipeSafety,
    /// Preconditions that must be met
    pub preconditions: Vec<Precondition>,
    /// Rollback template (for mutations)
    pub rollback: Option<RollbackTemplate>,
    /// Who/what created this recipe
    pub created_by: RecipeCreator,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last updated
    pub updated_at: DateTime<Utc>,
    /// Internal confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Number of successful uses
    pub success_count: u64,
    /// Number of failed uses
    pub failure_count: u64,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Whether this is an experimental draft
    pub is_draft: bool,
    /// Source session evidence ID (if created from session)
    pub source_session: Option<String>,
    /// Schema version
    pub schema_version: u32,
}

/// Intent pattern for matching requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPattern {
    /// Primary intent type (question, system_query, action_request)
    pub intent_type: String,
    /// Keywords that should be present
    pub keywords: Vec<String>,
    /// Target systems (cpu, memory, docker, systemd, etc.)
    pub targets: Vec<String>,
    /// Negative keywords (if present, don't match)
    pub negative_keywords: Vec<String>,
    /// Example requests that match this pattern
    pub examples: Vec<String>,
}

/// Tool plan template in a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeToolPlan {
    /// Description of the plan
    pub description: String,
    /// Tool steps to execute
    pub steps: Vec<RecipeToolStep>,
    /// Whether this plan contains mutations
    pub has_mutations: bool,
}

/// A step in a recipe tool plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeToolStep {
    /// Tool name
    pub tool_name: String,
    /// Parameter templates (can contain placeholders)
    pub parameters: HashMap<String, String>,
    /// Whether this is a mutation
    pub is_mutation: bool,
    /// Description of what this step does
    pub description: String,
    /// Condition to execute this step (optional)
    pub condition: Option<String>,
}

/// Safety classification for a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSafety {
    /// Risk level
    pub risk_level: RecipeRiskLevel,
    /// Required confirmation (if any)
    pub confirmation_required: Option<String>,
    /// Warnings to show user
    pub warnings: Vec<String>,
}

/// Risk level for recipes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeRiskLevel {
    ReadOnly,
    LowRisk,
    MediumRisk,
    HighRisk,
}

impl RecipeRiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecipeRiskLevel::ReadOnly => "read_only",
            RecipeRiskLevel::LowRisk => "low_risk",
            RecipeRiskLevel::MediumRisk => "medium_risk",
            RecipeRiskLevel::HighRisk => "high_risk",
        }
    }
}

/// Precondition for a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precondition {
    /// Description of what must be true
    pub description: String,
    /// Check type
    pub check_type: PreconditionCheck,
}

/// Types of precondition checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreconditionCheck {
    /// Package must be installed
    PackageInstalled { name: String },
    /// Service must be running
    ServiceRunning { name: String },
    /// File must exist
    FileExists { path: String },
    /// Custom command must succeed
    CommandSucceeds { command: String },
}

/// Rollback template for mutation recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackTemplate {
    /// How to revert
    pub description: String,
    /// Steps to revert
    pub steps: Vec<String>,
    /// Whether automatic rollback is possible
    pub auto_rollback: bool,
}

/// Who created a recipe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeCreator {
    /// Created automatically by Anna
    Anna,
    /// Created/improved by Junior verification
    Junior,
    /// Manually created by user
    User,
}

impl Recipe {
    /// Create a new recipe
    pub fn new(name: &str, description: &str, created_by: RecipeCreator) -> Self {
        let id = generate_recipe_id();
        let now = Utc::now();

        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            intent_pattern: IntentPattern {
                intent_type: String::new(),
                keywords: Vec::new(),
                targets: Vec::new(),
                negative_keywords: Vec::new(),
                examples: Vec::new(),
            },
            tool_plan: RecipeToolPlan {
                description: String::new(),
                steps: Vec::new(),
                has_mutations: false,
            },
            safety: RecipeSafety {
                risk_level: RecipeRiskLevel::ReadOnly,
                confirmation_required: None,
                warnings: Vec::new(),
            },
            preconditions: Vec::new(),
            rollback: None,
            created_by,
            created_at: now,
            updated_at: now,
            confidence: 0.5,
            success_count: 0,
            failure_count: 0,
            tags: Vec::new(),
            is_draft: false,
            source_session: None,
            schema_version: RECIPE_SCHEMA_VERSION,
        }
    }

    /// Format as a one-line summary
    pub fn format_summary(&self) -> String {
        let draft = if self.is_draft { " [draft]" } else { "" };
        format!(
            "[{}] {} - {} ({}% confidence, {} uses){}",
            self.id,
            self.name,
            truncate_string(&self.description, 40),
            (self.confidence * 100.0) as u32,
            self.success_count,
            draft
        )
    }

    /// Format detailed view
    pub fn format_detail(&self) -> String {
        let mut lines = vec![
            format!("[{}] Recipe: {}", self.id, self.name),
            format!("  Description: {}", self.description),
            format!("  Created:     {} by {:?}", self.created_at.format("%Y-%m-%d"), self.created_by),
            format!("  Updated:     {}", self.updated_at.format("%Y-%m-%d")),
            format!("  Confidence:  {:.0}%", self.confidence * 100.0),
            format!("  Success:     {} uses, {} failures", self.success_count, self.failure_count),
            format!("  Risk:        {}", self.safety.risk_level.as_str()),
        ];

        if self.is_draft {
            lines.push("  Status:      DRAFT (experimental)".to_string());
        }

        if !self.intent_pattern.keywords.is_empty() {
            lines.push(format!("  Keywords:    {}", self.intent_pattern.keywords.join(", ")));
        }

        if !self.intent_pattern.targets.is_empty() {
            lines.push(format!("  Targets:     {}", self.intent_pattern.targets.join(", ")));
        }

        if !self.tool_plan.steps.is_empty() {
            lines.push(format!("  Steps:       {} tool(s)", self.tool_plan.steps.len()));
            for step in &self.tool_plan.steps {
                let mutation = if step.is_mutation { " [mutation]" } else { "" };
                lines.push(format!("    - {}{}", step.tool_name, mutation));
            }
        }

        if !self.tags.is_empty() {
            lines.push(format!("  Tags:        {}", self.tags.join(", ")));
        }

        lines.join("\n")
    }

    /// Record a successful use
    pub fn record_success(&mut self) {
        self.success_count += 1;
        // Increase confidence slightly (diminishing returns)
        self.confidence = (self.confidence + 0.02).min(0.99);
        self.updated_at = Utc::now();
    }

    /// Record a failed use
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        // Decrease confidence
        self.confidence = (self.confidence - 0.05).max(0.1);
        self.updated_at = Utc::now();
    }

    /// Calculate match score against a request
    pub fn match_score(&self, request: &str, intent: &str, targets: &[String]) -> f64 {
        let request_lower = request.to_lowercase();
        let mut score = 0.0;

        // Intent match
        if !self.intent_pattern.intent_type.is_empty() && self.intent_pattern.intent_type == intent {
            score += 0.3;
        }

        // Keyword matches
        let keyword_count = self.intent_pattern.keywords.len();
        if keyword_count > 0 {
            let matched = self.intent_pattern.keywords.iter()
                .filter(|k| request_lower.contains(&k.to_lowercase()))
                .count();
            score += 0.3 * (matched as f64 / keyword_count as f64);
        }

        // Target matches
        let target_count = self.intent_pattern.targets.len();
        if target_count > 0 {
            let matched = self.intent_pattern.targets.iter()
                .filter(|t| targets.iter().any(|rt| rt.to_lowercase() == t.to_lowercase()))
                .count();
            score += 0.2 * (matched as f64 / target_count as f64);
        }

        // Negative keyword penalty
        for neg in &self.intent_pattern.negative_keywords {
            if request_lower.contains(&neg.to_lowercase()) {
                score -= 0.5;
            }
        }

        // Confidence multiplier
        score *= self.confidence;

        // Draft penalty
        if self.is_draft {
            score *= 0.5;
        }

        score.max(0.0)
    }

    /// Save to file
    pub fn save(&self) -> std::io::Result<()> {
        RecipeManager::ensure_dirs()?;
        let path = format!("{}/{}.json", RECIPES_DIR, self.id);
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    /// Load from file by ID
    pub fn load(id: &str) -> Option<Self> {
        let path = format!("{}/{}.json", RECIPES_DIR, id);
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }
}

/// Recipe manager for storage and retrieval
#[derive(Debug, Default)]
pub struct RecipeManager;

impl RecipeManager {
    /// Ensure recipe directories exist
    pub fn ensure_dirs() -> std::io::Result<()> {
        fs::create_dir_all(RECIPES_DIR)?;
        fs::create_dir_all(RECIPE_ARCHIVE_DIR)?;
        Ok(())
    }

    /// Create a recipe from a successful session
    pub fn create_from_session(
        name: &str,
        description: &str,
        request_text: &str,
        intent: &str,
        targets: &[String],
        tools_used: &[(String, bool)], // (name, is_mutation)
        reliability_score: u32,
        session_evidence_id: &str,
    ) -> Option<Recipe> {
        // Only create if reliability is high enough
        let is_draft = reliability_score < MIN_RELIABILITY_FOR_RECIPE;

        let mut recipe = Recipe::new(name, description, RecipeCreator::Anna);
        recipe.is_draft = is_draft;
        recipe.source_session = Some(session_evidence_id.to_string());

        // Set initial confidence based on reliability
        recipe.confidence = (reliability_score as f64 / 100.0).clamp(0.3, 0.9);

        // Build intent pattern
        recipe.intent_pattern.intent_type = intent.to_string();
        recipe.intent_pattern.keywords = extract_keywords(request_text);
        recipe.intent_pattern.targets = targets.to_vec();
        recipe.intent_pattern.examples.push(request_text.to_string());

        // Build tool plan
        let has_mutations = tools_used.iter().any(|(_, is_mut)| *is_mut);
        recipe.tool_plan.has_mutations = has_mutations;
        recipe.tool_plan.description = format!("Plan extracted from: {}", request_text);

        for (tool_name, is_mutation) in tools_used {
            recipe.tool_plan.steps.push(RecipeToolStep {
                tool_name: tool_name.clone(),
                parameters: HashMap::new(),
                is_mutation: *is_mutation,
                description: String::new(),
                condition: None,
            });
        }

        // Set safety based on mutations
        recipe.safety.risk_level = if has_mutations {
            RecipeRiskLevel::MediumRisk
        } else {
            RecipeRiskLevel::ReadOnly
        };

        // Save and update index
        if recipe.save().is_ok() {
            let mut index = RecipeIndex::load();
            index.add_recipe(&recipe);
            let _ = index.save();
            Some(recipe)
        } else {
            None
        }
    }

    /// Get all recipes
    pub fn get_all() -> Vec<Recipe> {
        let index = RecipeIndex::load();
        index.recipe_ids.iter()
            .filter_map(|id| Recipe::load(id))
            .collect()
    }

    /// Get recipe by ID
    pub fn get(id: &str) -> Option<Recipe> {
        Recipe::load(id)
    }

    /// Find matching recipes for a request
    pub fn find_matches(request: &str, intent: &str, targets: &[String], limit: usize) -> Vec<(Recipe, f64)> {
        let mut matches: Vec<_> = Self::get_all()
            .into_iter()
            .map(|r| {
                let score = r.match_score(request, intent, targets);
                (r, score)
            })
            .filter(|(_, score)| *score > 0.1)
            .collect();

        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(limit);
        matches
    }

    /// Search recipes by keyword
    pub fn search(query: &str, limit: usize) -> Vec<Recipe> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<_> = extract_keywords(&query_lower);

        let mut scored: Vec<_> = Self::get_all()
            .into_iter()
            .map(|r| {
                let mut score = 0;

                // Name match
                if r.name.to_lowercase().contains(&query_lower) {
                    score += 10;
                }

                // Description match
                if r.description.to_lowercase().contains(&query_lower) {
                    score += 5;
                }

                // Keyword matches
                for word in &query_words {
                    if r.intent_pattern.keywords.iter().any(|k| k.to_lowercase() == *word) {
                        score += 3;
                    }
                }

                // Tag matches
                for word in &query_words {
                    if r.tags.iter().any(|t| t.to_lowercase() == *word) {
                        score += 2;
                    }
                }

                (r, score)
            })
            .filter(|(_, score)| *score > 0)
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().take(limit).map(|(r, _)| r).collect()
    }

    /// Archive a recipe (for deletion/forget)
    pub fn archive(id: &str, reason: &str) -> std::io::Result<bool> {
        Self::ensure_dirs()?;

        let recipe = match Recipe::load(id) {
            Some(r) => r,
            None => return Ok(false),
        };

        // Save to archive
        let archive_path = format!("{}/{}.json", RECIPE_ARCHIVE_DIR, id);
        let archive_data = ArchivedRecipe {
            recipe,
            archived_at: Utc::now(),
            reason: reason.to_string(),
        };
        let json = serde_json::to_string_pretty(&archive_data)?;
        fs::write(archive_path, json)?;

        // Remove original
        let original_path = format!("{}/{}.json", RECIPES_DIR, id);
        fs::remove_file(original_path)?;

        // Update index
        let mut index = RecipeIndex::load();
        index.recipe_ids.retain(|rid| rid != id);
        index.archived_count += 1;
        index.save()?;

        Ok(true)
    }

    /// Get top N recipes by usage
    pub fn get_top(limit: usize) -> Vec<Recipe> {
        let mut recipes = Self::get_all();
        recipes.sort_by(|a, b| b.success_count.cmp(&a.success_count));
        recipes.truncate(limit);
        recipes
    }

    /// Get recipe statistics
    pub fn get_stats() -> RecipeStats {
        let index = RecipeIndex::load();
        let recipes = Self::get_all();

        let draft_count = recipes.iter().filter(|r| r.is_draft).count();
        let total_uses: u64 = recipes.iter().map(|r| r.success_count).sum();

        // Find most used recipes
        let mut by_usage: Vec<_> = recipes.iter()
            .map(|r| (r.id.clone(), r.name.clone(), r.success_count))
            .collect();
        by_usage.sort_by(|a, b| b.2.cmp(&a.2));
        let top_recipes: Vec<_> = by_usage.into_iter().take(3).collect();

        RecipeStats {
            total_recipes: index.recipe_ids.len(),
            draft_count,
            archived_count: index.archived_count as usize,
            total_uses,
            last_created_at: index.last_created_at,
            top_recipes,
        }
    }
}

/// Archived recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedRecipe {
    pub recipe: Recipe,
    pub archived_at: DateTime<Utc>,
    pub reason: String,
}

/// Recipe index for fast lookups
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeIndex {
    /// Schema version
    pub schema_version: u32,
    /// All recipe IDs
    pub recipe_ids: Vec<String>,
    /// Archived count
    pub archived_count: u64,
    /// Last recipe created timestamp
    pub last_created_at: Option<DateTime<Utc>>,
}

impl RecipeIndex {
    /// Load from file or create default
    pub fn load() -> Self {
        let path = Path::new(RECIPE_INDEX_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(index) = serde_json::from_str(&content) {
                    return index;
                }
            }
        }
        Self {
            schema_version: RECIPE_SCHEMA_VERSION,
            ..Default::default()
        }
    }

    /// Save to file
    pub fn save(&self) -> std::io::Result<()> {
        RecipeManager::ensure_dirs()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(RECIPE_INDEX_FILE, json)
    }

    /// Add a recipe to the index
    pub fn add_recipe(&mut self, recipe: &Recipe) {
        if !self.recipe_ids.contains(&recipe.id) {
            self.recipe_ids.push(recipe.id.clone());
        }
        self.last_created_at = Some(recipe.created_at);
    }
}

/// Recipe statistics
#[derive(Debug, Clone)]
pub struct RecipeStats {
    pub total_recipes: usize,
    pub draft_count: usize,
    pub archived_count: usize,
    pub total_uses: u64,
    pub last_created_at: Option<DateTime<Utc>>,
    /// Top recipes by usage (id, name, uses)
    pub top_recipes: Vec<(String, String, u64)>,
}

impl RecipeStats {
    /// Format for status display
    pub fn format_summary(&self) -> String {
        let drafts = if self.draft_count > 0 {
            format!(" ({} drafts)", self.draft_count)
        } else {
            String::new()
        };

        format!(
            "{} recipes{}, {} total uses",
            self.total_recipes,
            drafts,
            self.total_uses
        )
    }

    /// Format top recipes
    pub fn format_top_recipes(&self) -> String {
        if self.top_recipes.is_empty() {
            return "(none yet)".to_string();
        }

        self.top_recipes.iter()
            .map(|(_, name, uses)| format!("{} ({} uses)", name, uses))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Extract keywords from text
fn extract_keywords(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 3)
        .map(|s| s.to_string())
        .collect()
}

/// Truncate string with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_recipe_id() {
        let id = generate_recipe_id();
        assert!(id.starts_with(RECIPE_EVIDENCE_PREFIX));
    }

    #[test]
    fn test_recipe_new() {
        let recipe = Recipe::new("Test Recipe", "A test description", RecipeCreator::Anna);
        assert!(recipe.id.starts_with(RECIPE_EVIDENCE_PREFIX));
        assert_eq!(recipe.name, "Test Recipe");
        assert_eq!(recipe.created_by, RecipeCreator::Anna);
        assert!(!recipe.is_draft);
    }

    #[test]
    fn test_recipe_format_summary() {
        let mut recipe = Recipe::new("Check CPU", "Check CPU information", RecipeCreator::Anna);
        recipe.confidence = 0.85;
        recipe.success_count = 5;
        let summary = recipe.format_summary();
        assert!(summary.contains("Check CPU"));
        assert!(summary.contains("85%"));
        assert!(summary.contains("5 uses"));
    }

    #[test]
    fn test_recipe_record_success() {
        let mut recipe = Recipe::new("Test", "Test", RecipeCreator::Anna);
        recipe.confidence = 0.5;
        recipe.record_success();
        assert_eq!(recipe.success_count, 1);
        assert!(recipe.confidence > 0.5);
    }

    #[test]
    fn test_recipe_record_failure() {
        let mut recipe = Recipe::new("Test", "Test", RecipeCreator::Anna);
        recipe.confidence = 0.5;
        recipe.record_failure();
        assert_eq!(recipe.failure_count, 1);
        assert!(recipe.confidence < 0.5);
    }

    #[test]
    fn test_recipe_match_score() {
        let mut recipe = Recipe::new("Check CPU", "Check CPU info", RecipeCreator::Anna);
        recipe.intent_pattern.intent_type = "question".to_string();
        recipe.intent_pattern.keywords = vec!["cpu".to_string(), "check".to_string()];
        recipe.intent_pattern.targets = vec!["cpu".to_string()];
        recipe.confidence = 0.8;

        let score = recipe.match_score(
            "what cpu do I have",
            "question",
            &["cpu".to_string()],
        );
        assert!(score > 0.3);
    }

    #[test]
    fn test_recipe_match_score_negative_keywords() {
        let mut recipe = Recipe::new("Check CPU", "Check CPU info", RecipeCreator::Anna);
        recipe.intent_pattern.keywords = vec!["cpu".to_string()];
        recipe.intent_pattern.negative_keywords = vec!["temperature".to_string()];
        recipe.confidence = 0.8;

        let score_without = recipe.match_score("what cpu do I have", "", &[]);
        let score_with = recipe.match_score("what is cpu temperature", "", &[]);
        assert!(score_without > score_with);
    }

    #[test]
    fn test_recipe_risk_level() {
        assert_eq!(RecipeRiskLevel::ReadOnly.as_str(), "read_only");
        assert_eq!(RecipeRiskLevel::MediumRisk.as_str(), "medium_risk");
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("What CPU do I have?");
        assert!(keywords.contains(&"what".to_string()));
        assert!(keywords.contains(&"cpu".to_string()));
    }

    #[test]
    fn test_recipe_index_default() {
        let index = RecipeIndex::default();
        assert!(index.recipe_ids.is_empty());
        assert_eq!(index.archived_count, 0);
    }

    #[test]
    fn test_recipe_stats_format() {
        let stats = RecipeStats {
            total_recipes: 10,
            draft_count: 2,
            archived_count: 1,
            total_uses: 50,
            last_created_at: None,
            top_recipes: vec![
                ("id1".to_string(), "Recipe 1".to_string(), 20),
            ],
        };
        let summary = stats.format_summary();
        assert!(summary.contains("10 recipes"));
        assert!(summary.contains("2 drafts"));
    }
}
