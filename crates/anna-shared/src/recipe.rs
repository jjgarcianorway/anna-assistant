//! Team-tagged recipes for learning from successful patterns.
//!
//! Recipes capture successful ticket resolutions for future reference.
//! Only persisted when: Ticket status = Verified, reliability score >= 80.
//!
//! Storage: ~/.anna/recipes/{recipe_id}.json
//!
//! v0.0.27: Extended with RecipeKind for config edits and change actions.

use crate::teams::Team;
use crate::ticket::RiskLevel;
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Kind of recipe action (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecipeKind {
    /// Read-only query (default, no system changes)
    Query,
    /// Append a line to config file if not present
    ConfigEditLineAppend,
    /// Ensure a specific line exists in config file
    ConfigEnsureLine,
    /// Clarification template (v0.0.31) - learned pattern of what to ask
    ClarificationTemplate,
    /// Install a package (future)
    #[serde(other)]
    Unknown,
}

impl Default for RecipeKind {
    fn default() -> Self {
        Self::Query
    }
}

/// Target for a recipe action (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeTarget {
    pub app_id: String,
    pub config_path_template: String,
}

impl RecipeTarget {
    pub fn new(app_id: impl Into<String>, config_path: impl Into<String>) -> Self {
        Self { app_id: app_id.into(), config_path_template: config_path.into() }
    }
    /// Expand config path template with environment variables
    pub fn expand_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(self.config_path_template.replace("$HOME", &home).replace("~", &home))
    }
}

/// Action specification for config edit recipes (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RecipeAction {
    /// Ensure a line exists in the config file
    EnsureLine { line: String },
    /// Append a line to the end of the config file
    AppendLine { line: String },
    /// No action (read-only query)
    None,
}

impl Default for RecipeAction {
    fn default() -> Self {
        Self::None
    }
}

/// Rollback information for reversible changes (v0.0.27)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub backup_path: PathBuf,
    pub description: String,
    #[serde(default)]
    pub tested: bool,
}

impl RollbackInfo {
    pub fn new(backup_path: PathBuf, description: impl Into<String>) -> Self {
        Self { backup_path, description: description.into(), tested: false }
    }
}

/// Signature that uniquely identifies a query pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeSignature {
    pub domain: String,
    pub intent: String,
    pub route_class: String,
    pub query_pattern: String,
}

impl RecipeSignature {
    pub fn new(domain: impl Into<String>, intent: impl Into<String>, route_class: impl Into<String>, query: &str) -> Self {
        Self { domain: domain.into(), intent: intent.into(), route_class: route_class.into(), query_pattern: query.to_lowercase().trim().to_string() }
    }
    pub fn hash_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// A learned recipe from a successful ticket resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub signature: RecipeSignature,
    pub team: Team,
    pub risk_level: RiskLevel,
    pub required_evidence_kinds: Vec<EvidenceKind>,
    pub probe_sequence: Vec<String>,
    #[serde(default)]
    pub answer_template: String,
    pub created_at: u64,
    #[serde(default)]
    pub success_count: u32,
    pub reliability_score: u8,
    #[serde(default)]
    pub kind: RecipeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<RecipeTarget>,
    #[serde(default)]
    pub action: RecipeAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback: Option<RollbackInfo>,
    // v0.0.31 clarification template fields
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clarification_slots: Vec<RecipeSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_question_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub populates_facts: Vec<String>,
    // v0.0.41: Deterministic retrieval keys for RAG-lite
    /// Intent tags for matching (e.g., ["enable", "syntax", "highlight"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intent_tags: Vec<String>,
    /// Target identifiers for boosted matching (e.g., ["vim", "vimrc"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    /// Preconditions required (e.g., ["vim_installed"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    /// v0.45.5: Clarification prerequisites - facts that must be known before execution
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clarify_prereqs: Vec<ClarifyPrereq>,
}

/// Slot definition for clarification template recipes (v0.0.31)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeSlot {
    /// Slot name (e.g., "editor_name", "config_path")
    pub name: String,
    /// Question ID to use
    pub question_id: String,
    /// Whether this slot is required
    #[serde(default = "default_true")]
    pub required: bool,
    /// Verification type (e.g., "binary", "unit", "mount")
    #[serde(default)]
    pub verify_type: String,
}

/// Prerequisite for recipe execution requiring clarification (v0.45.5)
/// When a recipe has a ClarifyPrereq, the system must ensure the fact
/// is known and verified before executing the recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClarifyPrereq {
    /// The fact key that must be known (e.g., "preferred_editor")
    pub fact_key: String,
    /// Question ID to use if fact is unknown
    pub question_id: String,
    /// Must offer only installed/verified options
    #[serde(default = "default_true")]
    pub evidence_only: bool,
    /// Verification command template (e.g., "command -v {}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_template: Option<String>,
}

impl ClarifyPrereq {
    pub fn new(fact_key: impl Into<String>, question_id: impl Into<String>) -> Self {
        Self {
            fact_key: fact_key.into(),
            question_id: question_id.into(),
            evidence_only: true,
            verify_template: None,
        }
    }

    pub fn with_verify(mut self, template: impl Into<String>) -> Self {
        self.verify_template = Some(template.into());
        self
    }

    /// Create prereq for editor selection
    pub fn editor() -> Self {
        Self::new("preferred_editor", "editor_select")
            .with_verify("command -v {}")
    }
}

fn default_true() -> bool { true }

impl RecipeSlot {
    pub fn new(name: &str, question_id: &str) -> Self {
        Self { name: name.to_string(), question_id: question_id.to_string(), required: true, verify_type: String::new() }
    }

    pub fn with_verify(mut self, verify_type: &str) -> Self {
        self.verify_type = verify_type.to_string();
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

impl Recipe {
    /// Create a new read-only query recipe from verified ticket
    pub fn new(
        signature: RecipeSignature,
        team: Team,
        risk_level: RiskLevel,
        required_evidence_kinds: Vec<EvidenceKind>,
        probe_sequence: Vec<String>,
        answer_template: String,
        reliability_score: u8,
    ) -> Self {
        let id = compute_recipe_id(&signature, team);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id,
            signature,
            team,
            risk_level,
            required_evidence_kinds,
            probe_sequence,
            answer_template,
            created_at,
            success_count: 1,
            reliability_score,
            kind: RecipeKind::Query,
            target: None,
            action: RecipeAction::None,
            rollback: None,
            clarification_slots: Vec::new(),
            default_question_id: None,
            populates_facts: Vec::new(),
            intent_tags: Vec::new(),
            targets: Vec::new(),
            preconditions: Vec::new(),
            clarify_prereqs: Vec::new(),
        }
    }

    /// Create a config edit recipe (v0.0.27)
    pub fn config_edit(
        signature: RecipeSignature,
        team: Team,
        target: RecipeTarget,
        action: RecipeAction,
        reliability_score: u8,
    ) -> Self {
        let id = compute_recipe_id(&signature, team);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let kind = match &action {
            RecipeAction::EnsureLine { .. } => RecipeKind::ConfigEnsureLine,
            RecipeAction::AppendLine { .. } => RecipeKind::ConfigEditLineAppend,
            RecipeAction::None => RecipeKind::Query,
        };

        Self {
            id,
            signature,
            team,
            risk_level: RiskLevel::LowRiskChange,
            required_evidence_kinds: vec![],
            probe_sequence: vec![],
            answer_template: String::new(),
            created_at,
            success_count: 1,
            reliability_score,
            kind,
            target: Some(target),
            action,
            rollback: None,
            clarification_slots: Vec::new(),
            default_question_id: None,
            populates_facts: Vec::new(),
            intent_tags: Vec::new(),
            targets: Vec::new(),
            preconditions: Vec::new(),
            clarify_prereqs: Vec::new(),
        }
    }

    /// Create a clarification template recipe (v0.0.31)
    /// Stores a learned pattern for what clarifications to ask for an intent
    pub fn clarification_template(
        signature: RecipeSignature,
        team: Team,
        slots: Vec<RecipeSlot>,
        default_question: Option<String>,
        populates: Vec<String>,
        reliability_score: u8,
    ) -> Self {
        let id = compute_recipe_id(&signature, team);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id,
            signature,
            team,
            risk_level: RiskLevel::ReadOnly,
            required_evidence_kinds: vec![],
            probe_sequence: vec![],
            answer_template: String::new(),
            created_at,
            success_count: 1,
            reliability_score,
            kind: RecipeKind::ClarificationTemplate,
            target: None,
            action: RecipeAction::None,
            rollback: None,
            clarification_slots: slots,
            default_question_id: default_question,
            populates_facts: populates,
            intent_tags: Vec::new(),
            targets: Vec::new(),
            preconditions: Vec::new(),
            clarify_prereqs: Vec::new(),
        }
    }

    /// Set rollback info
    pub fn with_rollback(mut self, rollback: RollbackInfo) -> Self {
        self.rollback = Some(rollback);
        self
    }

    /// Set intent tags for RAG-lite retrieval (v0.0.41)
    pub fn with_intent_tags(mut self, tags: Vec<String>) -> Self {
        self.intent_tags = tags;
        self
    }

    /// Set targets for boosted matching (v0.0.41)
    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.targets = targets;
        self
    }

    /// Set preconditions (v0.0.41)
    pub fn with_preconditions(mut self, preconditions: Vec<String>) -> Self {
        self.preconditions = preconditions;
        self
    }

    /// Set clarification prerequisites (v0.45.5)
    pub fn with_clarify_prereqs(mut self, prereqs: Vec<ClarifyPrereq>) -> Self {
        self.clarify_prereqs = prereqs;
        self
    }

    /// Check if recipe requires clarification before execution (v0.45.5)
    pub fn needs_clarification(&self) -> bool {
        !self.clarify_prereqs.is_empty()
    }

    /// Get clarification prerequisites
    pub fn get_clarify_prereqs(&self) -> &[ClarifyPrereq] {
        &self.clarify_prereqs
    }

    /// Increment success count
    pub fn record_success(&mut self) {
        self.success_count = self.success_count.saturating_add(1);
    }

    /// Check if recipe is mature (used successfully multiple times)
    pub fn is_mature(&self) -> bool {
        self.success_count >= 3
    }

    /// Check if this is a config edit recipe
    pub fn is_config_edit(&self) -> bool {
        matches!(
            self.kind,
            RecipeKind::ConfigEnsureLine | RecipeKind::ConfigEditLineAppend
        )
    }

    /// Check if this is a clarification template recipe (v0.0.31)
    pub fn is_clarification_template(&self) -> bool {
        matches!(self.kind, RecipeKind::ClarificationTemplate)
    }

    /// Get clarification slots if this is a template
    pub fn get_clarification_slots(&self) -> &[RecipeSlot] {
        &self.clarification_slots
    }

    /// Get filesystem path for this recipe
    pub fn file_path(&self) -> PathBuf {
        recipe_dir().join(format!("{}.json", self.id))
    }

    /// Save recipe to disk
    pub fn save(&self) -> std::io::Result<()> {
        let dir = recipe_dir();
        std::fs::create_dir_all(&dir)?;
        let path = self.file_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load recipe from disk
    pub fn load(recipe_id: &str) -> std::io::Result<Self> {
        let path = recipe_filename(recipe_id);
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Compute deterministic recipe ID from signature and team
pub fn compute_recipe_id(signature: &RecipeSignature, team: Team) -> String {
    let mut hasher = DefaultHasher::new();
    signature.hash(&mut hasher);
    team.to_string().hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

/// Get the recipes directory path
pub fn recipe_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("recipes")
}

/// Get path for a specific recipe file
pub fn recipe_filename(recipe_id: &str) -> PathBuf {
    recipe_dir().join(format!("{}.json", recipe_id))
}

/// Check if a recipe should be persisted (v0.45.x stabilization gate).
/// Only persist when: Verified status AND reliability >= 80.
///
/// This is the ONLY gate for recipe persistence - all callers MUST use this function.
/// Rationale: Never learn from unverified outcomes; only from proven successes.
pub fn should_persist_recipe(verified: bool, score: u8) -> bool {
    verified && score >= 80
}

/// Threshold for recipe persistence
pub const RECIPE_PERSIST_THRESHOLD: u8 = 80;

/// Clear all recipes (for reset) (v0.0.28)
pub fn clear_all_recipes() -> std::io::Result<()> {
    let dir = recipe_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

/// Count recipes in store
pub fn recipe_count() -> usize {
    let dir = recipe_dir();
    if !dir.exists() {
        return 0;
    }
    std::fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0)
}

// === RAG-lite: Keyword-based recipe retrieval (v0.0.35) ===

/// A recipe match with relevance score
#[derive(Debug, Clone)]
pub struct RecipeMatch {
    pub recipe: Recipe,
    pub score: u32,
    pub matched_keywords: Vec<String>,
}

/// Search recipes by keywords (v0.0.35 RAG-lite)
/// Returns top N matches sorted by score descending, then by id ascending for determinism.
pub fn search_recipes_by_keywords(keywords: &[&str], limit: usize) -> Vec<RecipeMatch> {
    let dir = recipe_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut matches: Vec<RecipeMatch> = Vec::new();

    // Load all recipes and score them
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(json) = std::fs::read_to_string(entry.path()) {
                if let Ok(recipe) = serde_json::from_str::<Recipe>(&json) {
                    if let Some(match_result) = score_recipe(&recipe, keywords) {
                        matches.push(match_result);
                    }
                }
            }
        }
    }

    // Sort by score desc, then by id asc for determinism
    matches.sort_by(|a, b| {
        b.score.cmp(&a.score).then_with(|| a.recipe.id.cmp(&b.recipe.id))
    });

    matches.truncate(limit);
    matches
}

/// Score a recipe against keywords
fn score_recipe(recipe: &Recipe, keywords: &[&str]) -> Option<RecipeMatch> {
    let mut score: u32 = 0;
    let mut matched = Vec::new();

    // Build searchable text from recipe
    let searchable = format!(
        "{} {} {} {}",
        recipe.signature.query_pattern,
        recipe.signature.domain,
        recipe.signature.route_class,
        recipe.answer_template
    ).to_lowercase();

    // Score each keyword
    for &kw in keywords {
        let kw_lower = kw.to_lowercase();
        if searchable.contains(&kw_lower) {
            score += 10;
            matched.push(kw.to_string());

            // Bonus for exact route_class match
            if recipe.signature.route_class.to_lowercase() == kw_lower {
                score += 20;
            }
            // Bonus for domain match
            if recipe.signature.domain.to_lowercase() == kw_lower {
                score += 15;
            }
        }
    }

    // Bonus for high reliability
    score += (recipe.reliability_score / 10) as u32;

    // Bonus for maturity
    if recipe.is_mature() {
        score += 5;
    }

    if score > 0 && !matched.is_empty() {
        Some(RecipeMatch { recipe: recipe.clone(), score, matched_keywords: matched })
    } else {
        None
    }
}

/// Find recipes for config edit intents (v0.0.35)
/// Used before escalating to junior reviewer
pub fn find_config_edit_recipes(app_id: &str) -> Vec<Recipe> {
    let dir = recipe_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut recipes: Vec<Recipe> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(json) = std::fs::read_to_string(entry.path()) {
                if let Ok(recipe) = serde_json::from_str::<Recipe>(&json) {
                    if recipe.is_config_edit() {
                        if let Some(target) = &recipe.target {
                            if target.app_id == app_id {
                                recipes.push(recipe);
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by success_count desc for consistency
    recipes.sort_by(|a, b| b.success_count.cmp(&a.success_count));
    recipes
}

// Tests moved to tests/recipe_tests.rs to keep this file under 400 lines
