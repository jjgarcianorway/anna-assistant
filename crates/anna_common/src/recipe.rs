//! Recipe System v3.0 - Learning from Success
//!
//! Recipes capture HOW to answer certain types of questions.
//! When Junior/Senior successfully answer a question, the engine extracts
//! a recipe describing what worked. Future similar questions can then be
//! answered by Brain using the recipe - no LLM needed.
//!
//! ## Design Goals
//!
//! 1. Learn from success - extract patterns from high-reliability answers
//! 2. Brain-first answers - use recipes to skip LLM calls
//! 3. Simple matching - type + key tokens + time window
//! 4. Graceful degradation - if recipe fails, fall back to LLM
//!
//! ## Recipe Structure
//!
//! ```text
//! Recipe {
//!     intent: "logs_annad_last_N_hours",
//!     question_type: SelfLogs,
//!     parameters: { "hours": "3" },
//!     probes: ["logs.annad"],
//!     answer_template: "In the last {hours} hours, annad logged {line_count} entries...",
//!     success_score: 0.95,
//!     usage_count: 12,
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::router_llm::QuestionType;

// ============================================================================
// Configuration
// ============================================================================

/// Recipe store location
pub const RECIPE_STORE_PATH: &str = "/var/lib/anna/recipes/store.json";

/// Minimum reliability score to extract a recipe
pub const MIN_RECIPE_RELIABILITY: f64 = 0.85;

/// Maximum recipes per question type
pub const MAX_RECIPES_PER_TYPE: usize = 20;

/// Recipe match confidence threshold
pub const RECIPE_MATCH_THRESHOLD: f64 = 0.7;

// ============================================================================
// Recipe
// ============================================================================

/// A recipe captures how to answer a specific type of question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe ID
    pub id: String,

    /// Normalized intent (e.g., "logs_annad_time_window")
    pub intent: String,

    /// Question type this recipe applies to
    pub question_type: QuestionType,

    /// Extracted parameters from the question
    /// e.g., { "hours": "3", "service": "annad" }
    #[serde(default)]
    pub parameters: HashMap<String, String>,

    /// Key tokens that must be present in similar questions
    #[serde(default)]
    pub key_tokens: Vec<String>,

    /// Probes to run to gather evidence
    pub probes: Vec<String>,

    /// Answer template with {placeholders}
    pub answer_template: String,

    /// Last reliability score when this recipe was used
    pub last_success_score: f64,

    /// Number of times this recipe has been used
    pub usage_count: u32,

    /// Last used timestamp (unix seconds)
    pub last_used: u64,

    /// Creation timestamp
    pub created_at: u64,
}

impl Recipe {
    /// Create a new recipe from a successful answer
    pub fn new(
        intent: &str,
        question_type: QuestionType,
        probes: Vec<String>,
        answer_template: &str,
        success_score: f64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: format!("{}_{}", intent, now),
            intent: intent.to_string(),
            question_type,
            parameters: HashMap::new(),
            key_tokens: vec![],
            probes,
            answer_template: answer_template.to_string(),
            last_success_score: success_score,
            usage_count: 1,
            last_used: now,
            created_at: now,
        }
    }

    /// Add a parameter to the recipe
    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }

    /// Add key tokens for matching
    pub fn with_tokens(mut self, tokens: &[&str]) -> Self {
        self.key_tokens = tokens.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Check if this recipe matches a question
    pub fn matches(&self, question: &str, question_type: &QuestionType) -> f64 {
        // Must be same question type
        if &self.question_type != question_type {
            return 0.0;
        }

        let q_lower = question.to_lowercase();

        // Check key tokens
        if !self.key_tokens.is_empty() {
            let token_matches = self
                .key_tokens
                .iter()
                .filter(|t| q_lower.contains(&t.to_lowercase()))
                .count();

            let token_ratio = token_matches as f64 / self.key_tokens.len() as f64;
            if token_ratio < 0.5 {
                return 0.0;
            }

            // Weighted score: type match + token match + recency bonus
            let base_score = 0.5 + (token_ratio * 0.3);
            let recency_bonus = self.recency_score() * 0.2;
            return (base_score + recency_bonus).min(1.0);
        }

        // No tokens - basic type match
        0.5 + self.recency_score() * 0.2
    }

    /// Calculate recency bonus (0.0-1.0, decays over 24 hours)
    fn recency_score(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age_secs = now.saturating_sub(self.last_used);
        let age_hours = age_secs as f64 / 3600.0;

        // Exponential decay: 1.0 at 0 hours, ~0.37 at 24 hours, ~0.14 at 48 hours
        (-age_hours / 24.0).exp()
    }

    /// Apply the recipe to generate an answer from evidence
    pub fn apply(&self, evidence: &HashMap<String, String>) -> String {
        let mut answer = self.answer_template.clone();

        // Replace {placeholders} with evidence values
        for (key, value) in evidence {
            answer = answer.replace(&format!("{{{}}}", key), value);
        }

        // Also replace with recipe parameters
        for (key, value) in &self.parameters {
            answer = answer.replace(&format!("{{{}}}", key), value);
        }

        answer
    }

    /// Record successful use of this recipe
    pub fn record_use(&mut self, score: f64) {
        self.usage_count += 1;
        self.last_success_score = score;
        self.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

// ============================================================================
// Recipe Store
// ============================================================================

/// Store for learned recipes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeStore {
    /// Recipes indexed by question type
    #[serde(default)]
    pub recipes: HashMap<String, Vec<Recipe>>,

    /// Total recipes stored
    #[serde(default)]
    pub total_recipes: usize,

    /// Total recipe applications (Brain answers via recipe)
    #[serde(default)]
    pub total_applications: u64,
}

impl RecipeStore {
    /// Load from disk or create new
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(RECIPE_STORE_PATH) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(RECIPE_STORE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(RECIPE_STORE_PATH, json)
    }

    /// Add a new recipe
    pub fn add(&mut self, recipe: Recipe) {
        let type_key = format!("{:?}", recipe.question_type);

        let recipes = self.recipes.entry(type_key).or_default();

        // Remove duplicate intents
        recipes.retain(|r| r.intent != recipe.intent);

        // Add new recipe
        recipes.push(recipe);

        // Trim to max size (remove oldest/least used)
        if recipes.len() > MAX_RECIPES_PER_TYPE {
            recipes.sort_by(|a, b| {
                // Sort by usage_count * recency_score (descending)
                let a_score = a.usage_count as f64 * a.recency_score();
                let b_score = b.usage_count as f64 * b.recency_score();
                b_score.partial_cmp(&a_score).unwrap()
            });
            recipes.truncate(MAX_RECIPES_PER_TYPE);
        }

        self.total_recipes = self.recipes.values().map(|v| v.len()).sum();
        let _ = self.save();
    }

    /// Find the best matching recipe for a question
    pub fn find_match(
        &self,
        question: &str,
        question_type: &QuestionType,
    ) -> Option<&Recipe> {
        let type_key = format!("{:?}", question_type);
        let recipes = self.recipes.get(&type_key)?;

        let mut best_match: Option<(&Recipe, f64)> = None;

        for recipe in recipes {
            let score = recipe.matches(question, question_type);
            if score >= RECIPE_MATCH_THRESHOLD {
                match &best_match {
                    Some((_, best_score)) if score > *best_score => {
                        best_match = Some((recipe, score));
                    }
                    None => {
                        best_match = Some((recipe, score));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(r, _)| r)
    }

    /// Record a recipe application
    pub fn record_application(&mut self, recipe_id: &str) {
        self.total_applications += 1;

        // Find and update the recipe
        for recipes in self.recipes.values_mut() {
            if let Some(recipe) = recipes.iter_mut().find(|r| r.id == recipe_id) {
                recipe.record_use(recipe.last_success_score);
                break;
            }
        }

        let _ = self.save();
    }

    /// Get statistics for status display
    pub fn stats(&self) -> RecipeStats {
        RecipeStats {
            total_recipes: self.total_recipes,
            total_applications: self.total_applications,
            recipes_by_type: self
                .recipes
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
        }
    }
}

/// Recipe statistics for display
#[derive(Debug, Clone)]
pub struct RecipeStats {
    pub total_recipes: usize,
    pub total_applications: u64,
    pub recipes_by_type: HashMap<String, usize>,
}

impl RecipeStats {
    /// Format for status display
    pub fn format_status(&self) -> String {
        let mut output = String::new();
        // v4.5.5: ASCII only
        output.push_str("RECIPE LEARNING\n");
        output.push_str("------------------------------------------\n");
        output.push_str(&format!(
            "Total recipes: {}\n",
            self.total_recipes
        ));
        output.push_str(&format!(
            "Recipe applications: {}\n",
            self.total_applications
        ));

        if !self.recipes_by_type.is_empty() {
            output.push_str("\nRecipes by type:\n");
            for (type_name, count) in &self.recipes_by_type {
                output.push_str(&format!("  {}: {}\n", type_name, count));
            }
        }

        output
    }
}

// ============================================================================
// Recipe Extraction
// ============================================================================

/// Extract a recipe from a successful answer
pub fn extract_recipe(
    question: &str,
    question_type: QuestionType,
    probes_used: &[String],
    answer: &str,
    reliability: f64,
) -> Option<Recipe> {
    // Only extract from high-reliability answers
    if reliability < MIN_RECIPE_RELIABILITY {
        return None;
    }

    // Generate intent from question type and key words
    let intent = generate_intent(question, &question_type);

    // Extract key tokens from question
    let tokens = extract_key_tokens(question);

    // Generate answer template
    let template = generate_template(answer);

    let recipe = Recipe::new(
        &intent,
        question_type,
        probes_used.to_vec(),
        &template,
        reliability,
    )
    .with_tokens(&tokens.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    Some(recipe)
}

/// Generate an intent string from question and type
fn generate_intent(question: &str, question_type: &QuestionType) -> String {
    let type_str = format!("{:?}", question_type).to_lowercase();
    let q_lower = question.to_lowercase();

    // Extract time-related tokens
    if q_lower.contains("hour") || q_lower.contains("minute") || q_lower.contains("day") {
        return format!("{}_time_window", type_str);
    }

    // Extract count-related tokens
    if q_lower.contains("how many") || q_lower.contains("count") || q_lower.contains("number") {
        return format!("{}_count", type_str);
    }

    // Default intent
    format!("{}_basic", type_str)
}

/// Extract key tokens from question
fn extract_key_tokens(question: &str) -> Vec<String> {
    let stopwords = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "can", "i", "you", "he", "she", "it",
        "we", "they", "what", "which", "who", "when", "where", "why", "how",
        "my", "your", "his", "her", "its", "our", "their", "this", "that",
        "these", "those", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "about", "me", "and", "or", "but",
    ];

    question
        .to_lowercase()
        .split_whitespace()
        .filter(|w| {
            w.len() > 2
                && !stopwords.contains(w)
                && w.chars().all(|c| c.is_alphanumeric())
        })
        .take(5)
        .map(|s| s.to_string())
        .collect()
}

/// Generate a template from an answer
fn generate_template(answer: &str) -> String {
    // For now, just use the answer as template
    // Future: Extract numbers/values and replace with {placeholders}
    answer.to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_creation() {
        let recipe = Recipe::new(
            "cpu_info_basic",
            QuestionType::CpuInfo,
            vec!["cpu.info".to_string()],
            "Your CPU is {model}",
            0.95,
        );

        assert_eq!(recipe.intent, "cpu_info_basic");
        assert_eq!(recipe.question_type, QuestionType::CpuInfo);
        assert_eq!(recipe.usage_count, 1);
    }

    #[test]
    fn test_recipe_with_params() {
        let recipe = Recipe::new(
            "logs_time_window",
            QuestionType::SelfLogs,
            vec!["logs.annad".to_string()],
            "In the last {hours} hours...",
            0.9,
        )
        .with_param("hours", "3");

        assert_eq!(recipe.parameters.get("hours"), Some(&"3".to_string()));
    }

    #[test]
    fn test_recipe_matching() {
        let recipe = Recipe::new(
            "cpu_info_basic",
            QuestionType::CpuInfo,
            vec!["cpu.info".to_string()],
            "Your CPU is...",
            0.95,
        )
        .with_tokens(&["cpu", "model"]);

        // Should match same type with tokens
        let score = recipe.matches("What CPU model do I have?", &QuestionType::CpuInfo);
        assert!(score > RECIPE_MATCH_THRESHOLD);

        // Should not match different type
        let score = recipe.matches("What CPU model?", &QuestionType::RamInfo);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_recipe_apply() {
        let recipe = Recipe::new(
            "cpu_info_basic",
            QuestionType::CpuInfo,
            vec!["cpu.info".to_string()],
            "Your CPU is {model} with {cores} cores",
            0.95,
        );

        let mut evidence = HashMap::new();
        evidence.insert("model".to_string(), "AMD Ryzen 9".to_string());
        evidence.insert("cores".to_string(), "16".to_string());

        let answer = recipe.apply(&evidence);
        assert!(answer.contains("AMD Ryzen 9"));
        assert!(answer.contains("16 cores"));
    }

    #[test]
    fn test_recipe_store() {
        let mut store = RecipeStore::default();

        let recipe = Recipe::new(
            "cpu_info_basic",
            QuestionType::CpuInfo,
            vec!["cpu.info".to_string()],
            "Your CPU is...",
            0.95,
        );

        store.add(recipe);

        assert_eq!(store.total_recipes, 1);

        let found = store.find_match("What CPU do I have?", &QuestionType::CpuInfo);
        assert!(found.is_some());
    }

    #[test]
    fn test_extract_recipe() {
        let recipe = extract_recipe(
            "How much RAM do I have?",
            QuestionType::RamInfo,
            &["mem.info".to_string()],
            "You have 32 GiB of RAM",
            0.95,
        );

        assert!(recipe.is_some());
        let r = recipe.unwrap();
        assert_eq!(r.question_type, QuestionType::RamInfo);
    }

    #[test]
    fn test_extract_recipe_low_reliability() {
        let recipe = extract_recipe(
            "How much RAM?",
            QuestionType::RamInfo,
            &["mem.info".to_string()],
            "Maybe 16 GiB?",
            0.5, // Below threshold
        );

        assert!(recipe.is_none());
    }

    #[test]
    fn test_extract_key_tokens() {
        let tokens = extract_key_tokens("How much free disk space on root?");
        assert!(tokens.contains(&"disk".to_string()));
        assert!(tokens.contains(&"space".to_string()));
        assert!(tokens.contains(&"free".to_string()));
        assert!(!tokens.contains(&"how".to_string())); // Stopword
    }

    #[test]
    fn test_generate_intent() {
        let intent = generate_intent("Show logs from last 3 hours", &QuestionType::SelfLogs);
        assert!(intent.contains("time_window"));

        let intent = generate_intent("How many cores?", &QuestionType::CpuInfo);
        assert!(intent.contains("count"));

        let intent = generate_intent("What CPU?", &QuestionType::CpuInfo);
        assert!(intent.contains("basic"));
    }

    #[test]
    fn test_recipe_stats() {
        let mut store = RecipeStore::default();
        store.add(Recipe::new(
            "cpu_basic",
            QuestionType::CpuInfo,
            vec![],
            "test",
            0.95,
        ));
        store.add(Recipe::new(
            "ram_basic",
            QuestionType::RamInfo,
            vec![],
            "test",
            0.95,
        ));

        let stats = store.stats();
        assert_eq!(stats.total_recipes, 2);
        assert!(stats.recipes_by_type.len() >= 2);
    }
}
