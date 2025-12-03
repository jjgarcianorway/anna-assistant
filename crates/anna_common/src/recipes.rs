//! Recipe System v0.0.37
//!
//! Recipes are learned patterns that Anna can reuse:
//! - Named intent patterns with conditions
//! - Tool plan templates (read-only and/or mutation)
//! - Safety classification with confirmation phrases
//! - Validation checks (preconditions)
//! - Evidence requirements (what must be collected)
//! - Rollback plans (for mutations)
//! - Post-checks (verification after execution)
//! - Provenance tracking (origin case, creator, when)
//! - Confidence scoring and success counters
//! - Notes for human understanding
//!
//! Storage: /var/lib/anna/recipes/
//! - Each recipe is a separate JSON file
//! - recipe_index.json for fast lookups
//! - archive/ for deleted recipes (tombstones)
//!
//! Recipe Lifecycle:
//! - Created when case succeeds with reliability >= 80%
//! - Draft created when reliability < 80% (not auto-suggested)
//! - Promoted from draft after successful validated run
//! - Archived (not deleted) via natural language commands

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

/// Recipe status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    /// Active and ready to use
    Active,
    /// Draft - not auto-suggested, needs promotion
    Draft,
    /// Archived - tombstone, not used
    Archived,
}

impl RecipeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecipeStatus::Active => "active",
            RecipeStatus::Draft => "draft",
            RecipeStatus::Archived => "archived",
        }
    }
}

impl Default for RecipeStatus {
    fn default() -> Self {
        RecipeStatus::Active
    }
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
    /// v0.0.37: Intent tags for categorization
    pub intent_tags: Vec<String>,
    /// Intent pattern (what kinds of requests this matches)
    pub intent_pattern: IntentPattern,
    /// Tool plan template
    pub tool_plan: RecipeToolPlan,
    /// Safety classification
    pub safety: RecipeSafety,
    /// Preconditions that must be met
    pub preconditions: Vec<Precondition>,
    /// v0.0.37: Evidence that must be collected before applying
    pub evidence_required: Vec<String>,
    /// Rollback template (for mutations)
    pub rollback: Option<RollbackTemplate>,
    /// v0.0.37: Post-execution checks
    pub post_checks: Vec<PostCheck>,
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
    /// v0.0.37: Recipe status (active/draft/archived)
    #[serde(default)]
    pub status: RecipeStatus,
    /// Whether this is an experimental draft (deprecated, use status)
    #[serde(default)]
    pub is_draft: bool,
    /// v0.0.37: Origin case ID (where this recipe came from)
    #[serde(default)]
    pub origin_case_id: Option<String>,
    /// Source session evidence ID (if created from session)
    pub source_session: Option<String>,
    /// v0.0.37: Human-readable notes
    #[serde(default)]
    pub notes: String,
    /// Schema version
    pub schema_version: u32,
}

/// v0.0.37: Post-execution check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCheck {
    /// Description of what to verify
    pub description: String,
    /// Check type
    pub check_type: PostCheckType,
}

/// v0.0.37: Types of post-execution checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostCheckType {
    /// Service must be running after execution
    ServiceRunning { name: String },
    /// File must exist after execution
    FileExists { path: String },
    /// Command must succeed after execution
    CommandSucceeds { command: String },
    /// Output must contain string
    OutputContains { command: String, expected: String },
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
    /// v0.0.37: Required confirmation phrase (if any)
    #[serde(alias = "confirmation_required")]
    pub confirmation_phrase: Option<String>,
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
            intent_tags: Vec::new(),
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
                confirmation_phrase: None,
                warnings: Vec::new(),
            },
            preconditions: Vec::new(),
            evidence_required: Vec::new(),
            rollback: None,
            post_checks: Vec::new(),
            created_by,
            created_at: now,
            updated_at: now,
            confidence: 0.5,
            success_count: 0,
            failure_count: 0,
            tags: Vec::new(),
            status: RecipeStatus::Active,
            is_draft: false,
            origin_case_id: None,
            source_session: None,
            notes: String::new(),
            schema_version: RECIPE_SCHEMA_VERSION,
        }
    }

    /// Format as a one-line summary
    pub fn format_summary(&self) -> String {
        let status_str = match self.status {
            RecipeStatus::Draft => " [draft]",
            RecipeStatus::Archived => " [archived]",
            RecipeStatus::Active => "",
        };
        format!(
            "[{}] {} - {} ({}% confidence, {} uses){}",
            self.id,
            self.name,
            truncate_string(&self.description, 40),
            (self.confidence * 100.0) as u32,
            self.success_count,
            status_str
        )
    }

    /// Format detailed view
    pub fn format_detail(&self) -> String {
        let mut lines = vec![
            format!("[{}] Recipe: {}", self.id, self.name),
            format!("  Description: {}", self.description),
            format!("  Status:      {}", self.status.as_str().to_uppercase()),
            format!("  Created:     {} by {:?}", self.created_at.format("%Y-%m-%d"), self.created_by),
            format!("  Updated:     {}", self.updated_at.format("%Y-%m-%d")),
            format!("  Confidence:  {:.0}%", self.confidence * 100.0),
            format!("  Success:     {} uses, {} failures", self.success_count, self.failure_count),
            format!("  Risk:        {}", self.safety.risk_level.as_str()),
        ];

        if let Some(ref case_id) = self.origin_case_id {
            lines.push(format!("  Origin case: {}", case_id));
        }

        if !self.intent_tags.is_empty() {
            lines.push(format!("  Intent tags: {}", self.intent_tags.join(", ")));
        }

        if !self.intent_pattern.keywords.is_empty() {
            lines.push(format!("  Keywords:    {}", self.intent_pattern.keywords.join(", ")));
        }

        if !self.intent_pattern.targets.is_empty() {
            lines.push(format!("  Targets:     {}", self.intent_pattern.targets.join(", ")));
        }

        if !self.preconditions.is_empty() {
            lines.push(format!("  Preconditions: {}", self.preconditions.len()));
            for pc in &self.preconditions {
                lines.push(format!("    - {}", pc.description));
            }
        }

        if !self.evidence_required.is_empty() {
            lines.push(format!("  Evidence required: {}", self.evidence_required.join(", ")));
        }

        if !self.tool_plan.steps.is_empty() {
            lines.push(format!("  Steps:       {} tool(s)", self.tool_plan.steps.len()));
            for step in &self.tool_plan.steps {
                let mutation = if step.is_mutation { " [mutation]" } else { "" };
                lines.push(format!("    - {}{}", step.tool_name, mutation));
            }
        }

        if !self.post_checks.is_empty() {
            lines.push(format!("  Post-checks: {}", self.post_checks.len()));
            for pc in &self.post_checks {
                lines.push(format!("    - {}", pc.description));
            }
        }

        if let Some(ref rollback) = self.rollback {
            lines.push(format!("  Rollback:    {} (auto: {})", rollback.description, rollback.auto_rollback));
        }

        if let Some(ref phrase) = self.safety.confirmation_phrase {
            lines.push(format!("  Confirm:     \"{}\"", phrase));
        }

        if !self.tags.is_empty() {
            lines.push(format!("  Tags:        {}", self.tags.join(", ")));
        }

        if !self.notes.is_empty() {
            lines.push(format!("  Notes:       {}", self.notes));
        }

        lines.join("\n")
    }

    /// v0.0.37: Promote a draft recipe to active
    pub fn promote(&mut self) -> bool {
        if self.status == RecipeStatus::Draft {
            self.status = RecipeStatus::Active;
            self.is_draft = false;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// v0.0.37: Check if this recipe is usable (can be executed)
    /// Active and Draft recipes are usable; Archived are not
    pub fn is_usable(&self) -> bool {
        self.status == RecipeStatus::Active || self.status == RecipeStatus::Draft
    }

    /// v0.0.37: Check if this recipe can be auto-suggested
    /// Only Active recipes with sufficient confidence can auto-suggest
    pub fn can_auto_suggest(&self) -> bool {
        self.status == RecipeStatus::Active && self.confidence >= 0.5
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
        // v0.0.37: Only active recipes can be auto-matched
        if self.status != RecipeStatus::Active {
            return 0.0;
        }

        let request_lower = request.to_lowercase();
        let mut score = 0.0;

        // Intent match
        if !self.intent_pattern.intent_type.is_empty() && self.intent_pattern.intent_type == intent {
            score += 0.3;
        }

        // v0.0.37: Intent tags match
        let tag_count = self.intent_tags.len();
        if tag_count > 0 {
            let matched = self.intent_tags.iter()
                .filter(|t| request_lower.contains(&t.to_lowercase()))
                .count();
            score += 0.2 * (matched as f64 / tag_count as f64);
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

        // Success count bonus (diminishing)
        if self.success_count > 0 {
            score += 0.05 * (self.success_count as f64).log10().min(0.2);
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

    /// v0.0.37: Create a recipe from a successful case
    pub fn create_from_case(
        name: &str,
        description: &str,
        request_text: &str,
        intent: &str,
        intent_tags: &[String],
        targets: &[String],
        tools_used: &[(String, bool)], // (name, is_mutation)
        evidence_collected: &[String],
        reliability_score: u32,
        case_id: &str,
        notes: &str,
    ) -> Option<Recipe> {
        // v0.0.37: Create draft if reliability < 80%, active otherwise
        let status = if reliability_score >= MIN_RELIABILITY_FOR_RECIPE as u32 {
            RecipeStatus::Active
        } else {
            RecipeStatus::Draft
        };

        let mut recipe = Recipe::new(name, description, RecipeCreator::Anna);
        recipe.status = status;
        recipe.is_draft = status == RecipeStatus::Draft;
        recipe.origin_case_id = Some(case_id.to_string());
        recipe.source_session = Some(case_id.to_string());
        recipe.notes = notes.to_string();

        // Set initial confidence based on reliability
        recipe.confidence = (reliability_score as f64 / 100.0).clamp(0.3, 0.9);

        // v0.0.37: Set intent tags
        recipe.intent_tags = intent_tags.to_vec();

        // Build intent pattern
        recipe.intent_pattern.intent_type = intent.to_string();
        recipe.intent_pattern.keywords = extract_keywords(request_text);
        recipe.intent_pattern.targets = targets.to_vec();
        recipe.intent_pattern.examples.push(request_text.to_string());

        // v0.0.37: Store evidence requirements
        recipe.evidence_required = evidence_collected.to_vec();

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

        // v0.0.37: Add confirmation phrase for medium/high risk
        if has_mutations {
            recipe.safety.confirmation_phrase = Some("y".to_string());
        }

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

    /// Create a recipe from a successful session (legacy)
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
        Self::create_from_case(
            name,
            description,
            request_text,
            intent,
            &[],
            targets,
            tools_used,
            &[],
            reliability_score,
            session_evidence_id,
            "",
        )
    }

    /// v0.0.37: Promote a draft recipe by ID
    pub fn promote_recipe(id: &str) -> std::io::Result<bool> {
        if let Some(mut recipe) = Recipe::load(id) {
            if recipe.promote() {
                recipe.save()?;
                return Ok(true);
            }
        }
        Ok(false)
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

    /// Get top N active recipes by usage
    pub fn get_top(limit: usize) -> Vec<Recipe> {
        let mut recipes: Vec<_> = Self::get_all()
            .into_iter()
            .filter(|r| r.status == RecipeStatus::Active)
            .collect();
        recipes.sort_by(|a, b| b.success_count.cmp(&a.success_count));
        recipes.truncate(limit);
        recipes
    }

    /// Get recipe statistics
    pub fn get_stats() -> RecipeStats {
        let index = RecipeIndex::load();
        let recipes = Self::get_all();

        // Count by status
        let active_count = recipes.iter().filter(|r| r.status == RecipeStatus::Active).count();
        let draft_count = recipes.iter().filter(|r| r.status == RecipeStatus::Draft).count();
        let total_uses: u64 = recipes.iter().map(|r| r.success_count).sum();

        // Find most used active recipes
        let mut by_usage: Vec<_> = recipes.iter()
            .filter(|r| r.status == RecipeStatus::Active)
            .map(|r| (r.id.clone(), r.name.clone(), r.success_count))
            .collect();
        by_usage.sort_by(|a, b| b.2.cmp(&a.2));
        let top_recipes: Vec<_> = by_usage.into_iter().take(3).collect();

        RecipeStats {
            total_recipes: index.recipe_ids.len(),
            active_count,
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
    pub active_count: usize,
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
        let mut parts = vec![format!("{} active", self.active_count)];

        if self.draft_count > 0 {
            parts.push(format!("{} drafts", self.draft_count));
        }
        if self.archived_count > 0 {
            parts.push(format!("{} archived", self.archived_count));
        }

        format!(
            "{} recipes ({}), {} total uses",
            self.total_recipes,
            parts.join(", "),
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
            active_count: 7,
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
        assert!(summary.contains("7 active"));
        assert!(summary.contains("2 drafts"));
    }

    // v0.0.37: New tests for recipe status and promotion

    #[test]
    fn test_recipe_status_default() {
        let recipe = Recipe::new("Test", "Test desc", RecipeCreator::Anna);
        assert_eq!(recipe.status, RecipeStatus::Active);
        assert!(recipe.is_usable());
        assert!(recipe.can_auto_suggest());
    }

    #[test]
    fn test_recipe_status_draft() {
        let mut recipe = Recipe::new("Draft Test", "Test desc", RecipeCreator::Anna);
        recipe.status = RecipeStatus::Draft;
        recipe.is_draft = true;
        assert!(recipe.is_usable());
        assert!(!recipe.can_auto_suggest()); // Drafts should not auto-suggest
    }

    #[test]
    fn test_recipe_status_archived() {
        let mut recipe = Recipe::new("Archived Test", "Test desc", RecipeCreator::Anna);
        recipe.status = RecipeStatus::Archived;
        assert!(!recipe.is_usable());
        assert!(!recipe.can_auto_suggest());
    }

    #[test]
    fn test_recipe_promote() {
        let mut recipe = Recipe::new("Draft Recipe", "Test", RecipeCreator::Anna);
        recipe.status = RecipeStatus::Draft;
        recipe.is_draft = true;

        assert!(recipe.promote());
        assert_eq!(recipe.status, RecipeStatus::Active);
        assert!(!recipe.is_draft);
        assert!(recipe.can_auto_suggest());
    }

    #[test]
    fn test_recipe_promote_already_active() {
        let mut recipe = Recipe::new("Active Recipe", "Test", RecipeCreator::Anna);
        assert_eq!(recipe.status, RecipeStatus::Active);
        assert!(!recipe.promote()); // Should return false since already active
    }

    #[test]
    fn test_recipe_promote_archived_fails() {
        let mut recipe = Recipe::new("Archived Recipe", "Test", RecipeCreator::Anna);
        recipe.status = RecipeStatus::Archived;
        assert!(!recipe.promote()); // Archived recipes cannot be promoted
    }

    #[test]
    fn test_recipe_draft_no_match_score() {
        let mut recipe = Recipe::new("Draft Recipe", "Test", RecipeCreator::Anna);
        recipe.status = RecipeStatus::Draft;
        recipe.intent_pattern.keywords = vec!["test".to_string()];
        recipe.confidence = 0.8;

        // Draft recipes should not match
        let score = recipe.match_score("test something", "", &[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_post_check_types() {
        let check1 = PostCheck {
            description: "Check service is running".to_string(),
            check_type: PostCheckType::ServiceRunning { name: "nginx".to_string() },
        };
        assert!(matches!(check1.check_type, PostCheckType::ServiceRunning { .. }));

        let check2 = PostCheck {
            description: "Check file exists".to_string(),
            check_type: PostCheckType::FileExists { path: "/etc/nginx/nginx.conf".to_string() },
        };
        assert!(matches!(check2.check_type, PostCheckType::FileExists { .. }));

        let check3 = PostCheck {
            description: "Check command succeeds".to_string(),
            check_type: PostCheckType::CommandSucceeds { command: "nginx -t".to_string() },
        };
        assert!(matches!(check3.check_type, PostCheckType::CommandSucceeds { .. }));

        let check4 = PostCheck {
            description: "Check output".to_string(),
            check_type: PostCheckType::OutputContains {
                command: "cat /etc/hostname".to_string(),
                expected: "localhost".to_string(),
            },
        };
        assert!(matches!(check4.check_type, PostCheckType::OutputContains { .. }));
    }

    #[test]
    fn test_recipe_intent_tags() {
        let mut recipe = Recipe::new("Check CPU", "Test", RecipeCreator::Anna);
        recipe.intent_tags = vec!["hardware".to_string(), "cpu".to_string(), "info".to_string()];
        recipe.intent_pattern.intent_type = "question".to_string();
        recipe.intent_pattern.keywords = vec!["cpu".to_string()];
        recipe.confidence = 0.8;

        // Should get a boost from intent_tags matching
        let score = recipe.match_score("what is my cpu", "question", &["cpu".to_string()]);
        assert!(score > 0.3); // Should have a decent score
    }

    #[test]
    fn test_recipe_status_from_reliability_high() {
        // Test that high reliability (>= 80%) creates active recipe
        let mut recipe = Recipe::new("Test Recipe", "Test", RecipeCreator::Anna);
        let reliability = 85u32;
        let status = if reliability >= MIN_RELIABILITY_FOR_RECIPE as u32 {
            RecipeStatus::Active
        } else {
            RecipeStatus::Draft
        };
        recipe.status = status;
        recipe.is_draft = status == RecipeStatus::Draft;
        recipe.confidence = (reliability as f64 / 100.0).clamp(0.3, 0.9);

        assert_eq!(recipe.status, RecipeStatus::Active);
        assert!(!recipe.is_draft);
        assert!(recipe.confidence > 0.3);
        assert!(recipe.can_auto_suggest());
    }

    #[test]
    fn test_recipe_status_from_reliability_low() {
        // Test that low reliability (< 80%) creates draft recipe
        let mut recipe = Recipe::new("Draft Recipe", "Test", RecipeCreator::Anna);
        let reliability = 75u32;
        let status = if reliability >= MIN_RELIABILITY_FOR_RECIPE as u32 {
            RecipeStatus::Active
        } else {
            RecipeStatus::Draft
        };
        recipe.status = status;
        recipe.is_draft = status == RecipeStatus::Draft;
        recipe.origin_case_id = Some("case-456".to_string());
        recipe.confidence = (reliability as f64 / 100.0).clamp(0.3, 0.9);

        assert_eq!(recipe.status, RecipeStatus::Draft);
        assert!(recipe.is_draft);
        assert!(!recipe.can_auto_suggest());
        assert!(recipe.is_usable()); // Drafts are still usable, just not auto-suggested
    }

    #[test]
    fn test_recipe_format_detail_with_status() {
        let mut recipe = Recipe::new("Test Recipe", "Test description", RecipeCreator::Anna);
        recipe.status = RecipeStatus::Draft;
        recipe.intent_tags = vec!["test".to_string(), "demo".to_string()];
        recipe.notes = "Some important notes".to_string();

        let detail = recipe.format_detail();
        assert!(detail.contains("Status:      DRAFT"), "Expected DRAFT in: {}", detail);
        assert!(detail.contains("test, demo"), "Expected intent_tags in: {}", detail);
        assert!(detail.contains("Some important notes"), "Expected notes in: {}", detail);
    }
}
