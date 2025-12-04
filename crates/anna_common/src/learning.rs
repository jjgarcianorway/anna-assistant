//! Learning System v0.0.48
//!
//! Unified learning system for Anna:
//! - Knowledge Packs v1: Local learned recipes stored as JSON
//! - XP System: Deterministic progression with level titles
//! - Recipe Learning: Convert successful cases into reusable recipes
//!
//! Non-negotiables:
//! - Local only (no data leaves the machine)
//! - Bounded storage (hard caps on packs, recipes, size)
//! - Deterministic XP progression (no randomness)
//!
//! Storage: /var/lib/anna/knowledge_packs/installed/*.json

use chrono::{DateTime, Utc, Local, Datelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// Constants and Limits
// =============================================================================

/// Base directory for knowledge packs
pub const LEARNING_PACKS_DIR: &str = "/var/lib/anna/knowledge_packs/installed";

/// XP state file
pub const XP_STATE_FILE: &str = "/var/lib/anna/internal/xp_state.json";

/// Maximum number of packs allowed
pub const MAX_PACKS: usize = 50;

/// Maximum total recipes across all packs
pub const MAX_RECIPES_TOTAL: usize = 500;

/// Maximum size per recipe (24 KB)
pub const MAX_RECIPE_SIZE_BYTES: usize = 24 * 1024;

/// Evidence ID prefix for knowledge entries
pub const KNOWLEDGE_EVIDENCE_PREFIX: &str = "K";

/// Schema version for learning packs
pub const LEARNING_PACK_SCHEMA_VERSION: u32 = 1;

/// Minimum reliability for learning (90% for v0.0.48)
pub const MIN_RELIABILITY_FOR_LEARNING: u8 = 90;

/// Minimum evidence count for learning
pub const MIN_EVIDENCE_FOR_LEARNING: usize = 1;

// =============================================================================
// XP System
// =============================================================================

/// XP progression curve (non-linear)
/// Level N requires XP_CURVE[N] total XP
const XP_CURVE: &[u64] = &[
    0,      // Level 0
    10,     // Level 1
    25,     // Level 2
    50,     // Level 3
    100,    // Level 4
    175,    // Level 5
    275,    // Level 6
    400,    // Level 7
    550,    // Level 8
    750,    // Level 9
    1000,   // Level 10
    1300,   // Level 11
    1650,   // Level 12
    2050,   // Level 13
    2500,   // Level 14
    3000,   // Level 15
    3600,   // Level 16
    4300,   // Level 17
    5100,   // Level 18
    6000,   // Level 19
    7000,   // Level 20
    // Beyond 20: each level requires 1000 more than previous
];

/// Level titles (fun, nerdy, stable)
const LEVEL_TITLES: &[(&str, u8)] = &[
    ("Intern", 0),
    ("Apprentice", 3),
    ("Technician", 6),
    ("Analyst", 9),
    ("Engineer", 12),
    ("Senior Engineer", 15),
    ("Architect", 18),
    ("Wizard", 21),
    ("Sage", 25),
    ("Grandmaster", 30),
];

/// XP gains
pub const XP_GAIN_SUCCESS_85: u64 = 2;      // >= 85% reliability
pub const XP_GAIN_SUCCESS_90: u64 = 5;      // >= 90% reliability
pub const XP_GAIN_RECIPE_CREATED: u64 = 10; // Recipe created/improved

/// XP state stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpState {
    /// Total XP accumulated
    pub total_xp: u64,
    /// Number of successful answers
    pub successful_answers: u64,
    /// Number of recipes created
    pub recipes_created: u64,
    /// Number of recipes improved
    pub recipes_improved: u64,
    /// Last XP change timestamp
    pub last_xp_change: DateTime<Utc>,
    /// Schema version
    #[serde(default)]
    pub schema_version: u32,
}

impl Default for XpState {
    fn default() -> Self {
        Self {
            total_xp: 0,
            successful_answers: 0,
            recipes_created: 0,
            recipes_improved: 0,
            last_xp_change: Utc::now(),
            schema_version: 1,
        }
    }
}

impl XpState {
    /// Load XP state from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(XP_STATE_FILE) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save XP state to disk
    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = Path::new(XP_STATE_FILE).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(XP_STATE_FILE, json)
    }

    /// Calculate current level from total XP
    pub fn level(&self) -> u8 {
        for (i, &required) in XP_CURVE.iter().enumerate().rev() {
            if self.total_xp >= required {
                // Handle levels beyond the curve
                if i == XP_CURVE.len() - 1 {
                    let extra = self.total_xp - required;
                    let extra_levels = (extra / 1000) as u8;
                    return (i as u8) + extra_levels;
                }
                return i as u8;
            }
        }
        0
    }

    /// Get XP required for next level
    pub fn xp_for_next_level(&self) -> u64 {
        let current_level = self.level() as usize;
        if current_level < XP_CURVE.len() - 1 {
            XP_CURVE[current_level + 1]
        } else {
            // Beyond curve: need 1000 more than last threshold
            let base = XP_CURVE[XP_CURVE.len() - 1];
            let extra_levels = current_level - (XP_CURVE.len() - 1);
            base + ((extra_levels + 1) as u64 * 1000)
        }
    }

    /// Get current level title
    pub fn title(&self) -> &'static str {
        let level = self.level();
        for (title, min_level) in LEVEL_TITLES.iter().rev() {
            if level >= *min_level {
                return title;
            }
        }
        "Intern"
    }

    /// Add XP for successful answer
    pub fn add_success_xp(&mut self, reliability: u8) -> u64 {
        let gain = if reliability >= 90 {
            XP_GAIN_SUCCESS_90
        } else if reliability >= 85 {
            XP_GAIN_SUCCESS_85
        } else {
            0
        };

        if gain > 0 {
            self.total_xp += gain;
            self.successful_answers += 1;
            self.last_xp_change = Utc::now();
        }
        gain
    }

    /// Add XP for recipe creation
    pub fn add_recipe_created_xp(&mut self) -> u64 {
        self.total_xp += XP_GAIN_RECIPE_CREATED;
        self.recipes_created += 1;
        self.last_xp_change = Utc::now();
        XP_GAIN_RECIPE_CREATED
    }

    /// Add XP for recipe improvement
    pub fn add_recipe_improved_xp(&mut self) -> u64 {
        self.total_xp += XP_GAIN_RECIPE_CREATED;
        self.recipes_improved += 1;
        self.last_xp_change = Utc::now();
        XP_GAIN_RECIPE_CREATED
    }

    /// Get summary for display
    pub fn summary(&self) -> XpSummary {
        XpSummary {
            level: self.level(),
            title: self.title().to_string(),
            current_xp: self.total_xp,
            next_level_xp: self.xp_for_next_level(),
            successful_answers: self.successful_answers,
            recipes_created: self.recipes_created,
            recipes_improved: self.recipes_improved,
        }
    }
}

/// XP summary for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpSummary {
    pub level: u8,
    pub title: String,
    pub current_xp: u64,
    pub next_level_xp: u64,
    pub successful_answers: u64,
    pub recipes_created: u64,
    pub recipes_improved: u64,
}

// =============================================================================
// Knowledge Pack v1 Format
// =============================================================================

/// Source of the knowledge pack
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackSource {
    /// Learned from successful case execution
    LearnedFromCase,
    /// Imported from external source
    Imported,
    /// User-created manually
    UserCreated,
}

impl Default for PackSource {
    fn default() -> Self {
        PackSource::LearnedFromCase
    }
}

/// A knowledge pack containing learned recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePack {
    /// Stable pack ID
    pub pack_id: String,
    /// Human-readable name
    pub name: String,
    /// Version string
    pub version: String,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last modified
    pub updated_at: DateTime<Utc>,
    /// Source of this pack
    pub source: PackSource,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Recipes in this pack
    pub entries: Vec<LearnedRecipe>,
    /// Schema version
    #[serde(default)]
    pub schema_version: u32,
}

impl KnowledgePack {
    /// Create a new monthly learned pack
    pub fn new_monthly() -> Self {
        let now = Local::now();
        let pack_id = format!("learned-pack-{}{:02}", now.year(), now.month());
        let name = format!("Learned Pack ({}/{})", now.year(), now.month());

        Self {
            pack_id,
            name,
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: PackSource::LearnedFromCase,
            tags: vec!["learned".to_string(), "auto".to_string()],
            entries: Vec::new(),
            schema_version: LEARNING_PACK_SCHEMA_VERSION,
        }
    }

    /// Get the file path for this pack
    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(LEARNING_PACKS_DIR).join(format!("{}.json", self.pack_id))
    }

    /// Load a pack from disk
    pub fn load(pack_id: &str) -> io::Result<Self> {
        let path = PathBuf::from(LEARNING_PACKS_DIR).join(format!("{}.json", pack_id));
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Save pack to disk
    pub fn save(&self) -> io::Result<()> {
        fs::create_dir_all(LEARNING_PACKS_DIR)?;
        let json = serde_json::to_string_pretty(self)?;

        // Check size limit
        if json.len() > MAX_RECIPE_SIZE_BYTES * self.entries.len().max(1) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Pack too large: {} bytes", json.len()),
            ));
        }

        fs::write(self.file_path(), json)
    }

    /// Add or update a recipe
    pub fn add_or_update_recipe(&mut self, recipe: LearnedRecipe) -> bool {
        // Check for similar existing recipe
        let existing_idx = self.entries.iter().position(|r| {
            r.intent == recipe.intent
                && r.targets == recipe.targets
                && r.required_evidence_tools == recipe.required_evidence_tools
        });

        if let Some(idx) = existing_idx {
            // Update existing: increment wins, update examples
            self.entries[idx].wins += 1;
            self.entries[idx].last_used_at = Some(Utc::now());
            if self.entries[idx].examples.len() < 3
               && !self.entries[idx].examples.contains(&recipe.examples[0]) {
                self.entries[idx].examples.push(recipe.examples[0].clone());
            }
            self.updated_at = Utc::now();
            false // Updated, not created
        } else {
            // Check limits
            if self.entries.len() >= MAX_RECIPES_TOTAL / MAX_PACKS {
                // Evict least used recipe
                if let Some(idx) = self.entries.iter()
                    .enumerate()
                    .min_by_key(|(_, r)| r.wins)
                    .map(|(i, _)| i)
                {
                    self.entries.remove(idx);
                }
            }
            self.entries.push(recipe);
            self.updated_at = Utc::now();
            true // Created new
        }
    }

    /// Search recipes in this pack
    pub fn search(&self, query: &str, limit: usize) -> Vec<(&LearnedRecipe, f64)> {
        let query_lower = query.to_lowercase();
        let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<_> = self.entries.iter()
            .map(|r| {
                let score = r.match_score(&query_tokens);
                (r, score)
            })
            .filter(|(_, score)| *score > 0.0)
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }
}

/// Intent type for recipes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeIntent {
    SystemQuery,
    DoctorQuery,
    ActionRequest,
    Question,
}

impl Default for RecipeIntent {
    fn default() -> Self {
        RecipeIntent::SystemQuery
    }
}

/// A learned recipe entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedRecipe {
    /// Unique recipe ID within the pack
    pub recipe_id: String,
    /// Human-readable title
    pub title: String,
    /// Intent type
    pub intent: RecipeIntent,
    /// Target systems/domains
    pub targets: Vec<String>,
    /// Trigger keywords/patterns
    pub triggers: Vec<String>,
    /// Required evidence tools
    pub required_evidence_tools: Vec<String>,
    /// Safe checks (natural language steps)
    pub safe_checks: Vec<String>,
    /// Actions (optional, references mutation types)
    #[serde(default)]
    pub actions: Vec<RecipeAction>,
    /// Rollback instructions
    #[serde(default)]
    pub rollback: Option<String>,
    /// Confidence rules
    pub confidence_rules: Vec<String>,
    /// Notes
    #[serde(default)]
    pub notes: String,
    /// Examples (1-3)
    pub examples: Vec<String>,
    /// Number of successful uses
    #[serde(default)]
    pub wins: u64,
    /// Origin case ID
    #[serde(default)]
    pub origin_case_id: Option<String>,
    /// Last used timestamp
    #[serde(default)]
    pub last_used_at: Option<DateTime<Utc>>,
}

impl LearnedRecipe {
    /// Generate a unique recipe ID
    pub fn generate_id() -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        format!("LRN{:08}", ts % 100_000_000)
    }

    /// Calculate match score against query tokens
    pub fn match_score(&self, query_tokens: &[&str]) -> f64 {
        let mut score = 0.0;

        // Title match (high weight)
        let title_lower = self.title.to_lowercase();
        for token in query_tokens {
            if title_lower.contains(token) {
                score += 0.3;
            }
        }

        // Trigger match (high weight)
        for trigger in &self.triggers {
            let trigger_lower = trigger.to_lowercase();
            for token in query_tokens {
                if trigger_lower.contains(token) || token.contains(&trigger_lower) {
                    score += 0.25;
                }
            }
            // Exact trigger match bonus
            if query_tokens.iter().any(|t| trigger_lower == *t) {
                score += 0.15;
            }
        }

        // Target match
        for target in &self.targets {
            let target_lower = target.to_lowercase();
            for token in query_tokens {
                if target_lower.contains(token) {
                    score += 0.15;
                }
            }
        }

        // Recent usage boost
        if let Some(last_used) = self.last_used_at {
            let days_ago = (Utc::now() - last_used).num_days();
            if days_ago < 7 {
                score += 0.1;
            }
        }

        // Wins boost (log scale)
        if self.wins > 0 {
            score += (self.wins as f64).log10() * 0.05;
        }

        score.min(1.0)
    }
}

/// Recipe action (references existing mutation types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeAction {
    /// Action type
    pub action_type: String,
    /// Parameters
    pub parameters: HashMap<String, String>,
    /// Description
    pub description: String,
}

// =============================================================================
// Learning Manager
// =============================================================================

/// Manager for the learning system
pub struct LearningManager;

impl LearningManager {
    /// Ensure directories exist
    pub fn ensure_dirs() -> io::Result<()> {
        fs::create_dir_all(LEARNING_PACKS_DIR)?;
        if let Some(parent) = Path::new(XP_STATE_FILE).parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Get all installed packs
    pub fn list_packs() -> io::Result<Vec<KnowledgePack>> {
        let mut packs = Vec::new();

        if let Ok(entries) = fs::read_dir(LEARNING_PACKS_DIR) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(pack) = serde_json::from_str::<KnowledgePack>(&content) {
                            packs.push(pack);
                        }
                    }
                }
            }
        }

        // Enforce max packs limit
        if packs.len() > MAX_PACKS {
            packs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            packs.truncate(MAX_PACKS);
        }

        Ok(packs)
    }

    /// Get or create current month's learned pack
    pub fn get_or_create_monthly_pack() -> io::Result<KnowledgePack> {
        let now = Local::now();
        let pack_id = format!("learned-pack-{}{:02}", now.year(), now.month());

        match KnowledgePack::load(&pack_id) {
            Ok(pack) => Ok(pack),
            Err(_) => {
                let pack = KnowledgePack::new_monthly();
                pack.save()?;
                Ok(pack)
            }
        }
    }

    /// Count total recipes across all packs
    pub fn count_total_recipes() -> io::Result<usize> {
        let packs = Self::list_packs()?;
        Ok(packs.iter().map(|p| p.entries.len()).sum())
    }

    /// Check if we can add more recipes
    pub fn can_add_recipe() -> io::Result<bool> {
        let count = Self::count_total_recipes()?;
        Ok(count < MAX_RECIPES_TOTAL)
    }

    /// Search across all packs
    pub fn search_all(query: &str, limit: usize) -> io::Result<Vec<SearchHit>> {
        let packs = Self::list_packs()?;
        let mut all_results = Vec::new();

        for pack in &packs {
            for (recipe, score) in pack.search(query, limit) {
                all_results.push(SearchHit {
                    evidence_id: String::new(), // Will be assigned after sorting
                    pack_id: pack.pack_id.clone(),
                    pack_name: pack.name.clone(),
                    recipe: recipe.clone(),
                    score,
                });
            }
        }

        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(limit);

        // Assign evidence IDs after sorting
        for (i, hit) in all_results.iter_mut().enumerate() {
            hit.evidence_id = format!("{}{}", KNOWLEDGE_EVIDENCE_PREFIX, i + 1);
        }

        Ok(all_results)
    }

    /// Learn from a successful case
    pub fn learn_from_case(
        case_id: &str,
        request: &str,
        intent: RecipeIntent,
        targets: &[String],
        tools_used: &[String],
        reliability: u8,
        evidence_count: usize,
        was_executed: bool,
    ) -> io::Result<LearningResult> {
        Self::ensure_dirs()?;

        // Check eligibility
        if reliability < MIN_RELIABILITY_FOR_LEARNING {
            return Ok(LearningResult {
                learned: false,
                reason: format!("Reliability {} < {} required", reliability, MIN_RELIABILITY_FOR_LEARNING),
                recipe_id: None,
                xp_gained: 0,
            });
        }

        if evidence_count < MIN_EVIDENCE_FOR_LEARNING {
            return Ok(LearningResult {
                learned: false,
                reason: format!("Evidence count {} < {} required", evidence_count, MIN_EVIDENCE_FOR_LEARNING),
                recipe_id: None,
                xp_gained: 0,
            });
        }

        // Check limits
        if !Self::can_add_recipe()? {
            return Ok(LearningResult {
                learned: false,
                reason: format!("Max recipes ({}) reached", MAX_RECIPES_TOTAL),
                recipe_id: None,
                xp_gained: 0,
            });
        }

        // Build recipe
        let recipe_id = LearnedRecipe::generate_id();
        let title = build_recipe_title(request, &intent, targets);
        let triggers = extract_triggers(request);

        let recipe = LearnedRecipe {
            recipe_id: recipe_id.clone(),
            title,
            intent,
            targets: targets.to_vec(),
            triggers,
            required_evidence_tools: tools_used.to_vec(),
            safe_checks: vec![
                "Verify evidence collected".to_string(),
                "Check reliability score".to_string(),
            ],
            actions: Vec::new(),
            rollback: None,
            confidence_rules: vec![
                format!("Requires reliability >= {}", MIN_RELIABILITY_FOR_LEARNING),
            ],
            notes: format!("Learned from case {}", case_id),
            examples: vec![request.to_string()],
            wins: 1,
            origin_case_id: Some(case_id.to_string()),
            last_used_at: Some(Utc::now()),
        };

        // Add to monthly pack
        let mut pack = Self::get_or_create_monthly_pack()?;
        let is_new = pack.add_or_update_recipe(recipe);
        pack.save()?;

        // Update XP
        let mut xp_state = XpState::load();
        let xp_gained = if is_new {
            xp_state.add_recipe_created_xp()
        } else {
            xp_state.add_recipe_improved_xp()
        };

        // Also add success XP
        let success_xp = xp_state.add_success_xp(reliability);
        let total_xp = xp_gained + success_xp;

        xp_state.save()?;

        Ok(LearningResult {
            learned: true,
            reason: if is_new { "New recipe created".to_string() } else { "Existing recipe improved".to_string() },
            recipe_id: Some(recipe_id),
            xp_gained: total_xp,
        })
    }

    /// Get learning stats
    pub fn get_stats() -> io::Result<LearningStats> {
        let packs = Self::list_packs()?;
        let total_recipes: usize = packs.iter().map(|p| p.entries.len()).sum();
        let xp = XpState::load();
        let summary = xp.summary();

        Ok(LearningStats {
            pack_count: packs.len(),
            recipe_count: total_recipes,
            max_packs: MAX_PACKS,
            max_recipes: MAX_RECIPES_TOTAL,
            xp_summary: summary,
        })
    }

    /// Record a successful recipe use
    pub fn record_recipe_use(pack_id: &str, recipe_id: &str) -> io::Result<()> {
        if let Ok(mut pack) = KnowledgePack::load(pack_id) {
            if let Some(recipe) = pack.entries.iter_mut().find(|r| r.recipe_id == recipe_id) {
                recipe.wins += 1;
                recipe.last_used_at = Some(Utc::now());
                pack.save()?;
            }
        }
        Ok(())
    }
}

/// Search hit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub evidence_id: String,
    pub pack_id: String,
    pub pack_name: String,
    pub recipe: LearnedRecipe,
    pub score: f64,
}

impl SearchHit {
    /// Get a short summary for display
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} - {} (score: {:.2}, wins: {})",
            self.evidence_id,
            self.recipe.title,
            self.recipe.triggers.join(", "),
            self.score,
            self.recipe.wins
        )
    }

    /// Why this matched
    pub fn match_reason(&self) -> String {
        format!(
            "Matched triggers: {:?}, targets: {:?}",
            self.recipe.triggers,
            self.recipe.targets
        )
    }
}

/// Result of learning attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResult {
    pub learned: bool,
    pub reason: String,
    pub recipe_id: Option<String>,
    pub xp_gained: u64,
}

/// Learning system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    pub pack_count: usize,
    pub recipe_count: usize,
    pub max_packs: usize,
    pub max_recipes: usize,
    pub xp_summary: XpSummary,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Build a recipe title from request
fn build_recipe_title(request: &str, intent: &RecipeIntent, targets: &[String]) -> String {
    let intent_str = match intent {
        RecipeIntent::SystemQuery => "Query",
        RecipeIntent::DoctorQuery => "Diagnose",
        RecipeIntent::ActionRequest => "Action",
        RecipeIntent::Question => "Answer",
    };

    let target_str = if targets.is_empty() {
        "system".to_string()
    } else {
        targets[0].clone()
    };

    // Extract key verb/noun from request
    let words: Vec<&str> = request.split_whitespace().take(6).collect();
    let key_phrase = words.join(" ");

    format!("{}: {} ({})", intent_str, key_phrase, target_str)
}

/// Extract trigger keywords from request
fn extract_triggers(request: &str) -> Vec<String> {
    let stop_words = ["the", "a", "an", "is", "are", "was", "were", "what", "how",
                      "do", "does", "can", "could", "my", "i", "me", "to", "of"];

    request
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .take(5)
        .map(String::from)
        .collect()
}

/// Generate knowledge evidence ID
pub fn generate_knowledge_evidence_id(index: usize) -> String {
    format!("{}{}", KNOWLEDGE_EVIDENCE_PREFIX, index + 1)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_level_calculation() {
        let mut xp = XpState::default();
        assert_eq!(xp.level(), 0);
        assert_eq!(xp.title(), "Intern");

        xp.total_xp = 100;
        assert_eq!(xp.level(), 4);

        xp.total_xp = 1000;
        assert_eq!(xp.level(), 10);
        assert_eq!(xp.title(), "Analyst"); // Level 10 is still Analyst (Engineer starts at 12)
    }

    #[test]
    fn test_xp_gains() {
        let mut xp = XpState::default();

        // 85% reliability
        let gain = xp.add_success_xp(85);
        assert_eq!(gain, XP_GAIN_SUCCESS_85);
        assert_eq!(xp.total_xp, 2);

        // 90% reliability
        let gain = xp.add_success_xp(90);
        assert_eq!(gain, XP_GAIN_SUCCESS_90);
        assert_eq!(xp.total_xp, 7);

        // Recipe created
        let gain = xp.add_recipe_created_xp();
        assert_eq!(gain, XP_GAIN_RECIPE_CREATED);
        assert_eq!(xp.total_xp, 17);
    }

    #[test]
    fn test_recipe_match_score() {
        let recipe = LearnedRecipe {
            recipe_id: "test".to_string(),
            title: "Check disk space".to_string(),
            intent: RecipeIntent::SystemQuery,
            targets: vec!["disk".to_string()],
            triggers: vec!["disk".to_string(), "space".to_string(), "free".to_string()],
            required_evidence_tools: vec!["mount_usage".to_string()],
            safe_checks: vec![],
            actions: vec![],
            rollback: None,
            confidence_rules: vec![],
            notes: String::new(),
            examples: vec!["how much disk space".to_string()],
            wins: 5,
            origin_case_id: None,
            last_used_at: Some(Utc::now()),
        };

        let tokens = vec!["disk", "space"];
        let score = recipe.match_score(&tokens);
        assert!(score > 0.5); // Should have good match

        let tokens = vec!["memory", "ram"];
        let score = recipe.match_score(&tokens);
        assert!(score < 0.3); // Should have poor match
    }

    #[test]
    fn test_extract_triggers() {
        let triggers = extract_triggers("how much disk space is free on root");
        assert!(triggers.contains(&"disk".to_string()));
        assert!(triggers.contains(&"space".to_string()));
        assert!(triggers.contains(&"free".to_string()));
        assert!(!triggers.contains(&"the".to_string()));
        assert!(!triggers.contains(&"is".to_string()));
    }

    #[test]
    fn test_monthly_pack_id() {
        let pack = KnowledgePack::new_monthly();
        let now = Local::now();
        let expected = format!("learned-pack-{}{:02}", now.year(), now.month());
        assert_eq!(pack.pack_id, expected);
    }
}
