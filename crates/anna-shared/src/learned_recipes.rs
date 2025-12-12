//! Learned Recipes (v0.0.416).
//!
//! Self-learning recipe system that:
//! - Creates recipes from successful tickets
//! - Matches by intent (not by specific phrasing)
//! - Executes deterministically without LLM
//! - Tracks effectiveness and deprecates bad recipes
//!
//! Design goals:
//! - No hardcoded answers or questions
//! - Generic, parameterized recipes
//! - Learns from specialist success cases

use crate::canonical_intents::CanonicalIntent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A learned recipe - deterministic, replayable solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedRecipe {
    /// Stable ID
    pub id: String,
    /// Name
    pub name: String,
    /// Version (increments on update)
    pub version: u32,
    /// Canonical intent this recipe handles
    pub intent: CanonicalIntent,
    /// Domain (storage, services, etc.)
    pub domain: String,
    /// Required probes (must all succeed)
    pub required_probes: Vec<String>,
    /// Optional probes (nice to have)
    pub optional_probes: Vec<String>,
    /// Computation steps
    pub steps: Vec<RecipeComputeStep>,
    /// Answer template (ok case)
    pub answer_ok: AnswerTemplate,
    /// Answer template (critical/warning case)
    pub answer_critical: Option<AnswerTemplate>,
    /// Answer template (partial case)
    pub answer_partial: Option<AnswerTemplate>,
    /// Knowledge topics for enrichment
    pub knowledge_topics: Vec<String>,
    /// Tickets that contributed to this recipe
    pub source_tickets: Vec<String>,
    /// Stats
    pub stats: RecipeStats,
    /// Created timestamp
    pub created_at: u64,
    /// Last used timestamp
    pub last_used_at: u64,
    /// Deprecated flag
    pub deprecated: bool,
}

/// Computation step - deterministic logic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RecipeComputeStep {
    /// Extract a value from probe output
    Extract {
        probe: String,
        pattern: String,
        variable: String,
    },
    /// Compare a value against threshold
    Compare {
        variable: String,
        operator: CompareOp,
        threshold: f64,
        result_var: String,
    },
    /// Count items matching pattern
    Count {
        probe: String,
        pattern: String,
        variable: String,
    },
    /// Check if probe output is empty
    IsEmpty { probe: String, variable: String },
    /// Parse a numeric value
    ParseNumber {
        source_var: String,
        target_var: String,
    },
}

/// Comparison operator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl CompareOp {
    pub fn eval(&self, a: f64, b: f64) -> bool {
        match self {
            Self::Lt => a < b,
            Self::Le => a <= b,
            Self::Gt => a > b,
            Self::Ge => a >= b,
            Self::Eq => (a - b).abs() < f64::EPSILON,
            Self::Ne => (a - b).abs() >= f64::EPSILON,
        }
    }
}

/// Answer template with placeholders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerTemplate {
    /// Summary with {placeholders}
    pub summary: String,
    /// Detail lines with {placeholders}
    pub details: Vec<String>,
    /// Evidence probes to cite
    pub evidence: Vec<String>,
}

impl AnswerTemplate {
    /// Render template with values
    pub fn render(&self, values: &HashMap<String, String>) -> RenderedAnswer {
        RenderedAnswer {
            summary: substitute(&self.summary, values),
            details: self.details.iter().map(|d| substitute(d, values)).collect(),
            evidence: self.evidence.clone(),
        }
    }
}

/// Rendered answer
#[derive(Debug, Clone)]
pub struct RenderedAnswer {
    pub summary: String,
    pub details: Vec<String>,
    pub evidence: Vec<String>,
}

/// Substitute {placeholders} in template
fn substitute(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in values {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

/// Recipe statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStats {
    /// Total uses
    pub uses: u32,
    /// Successful uses
    pub successes: u32,
    /// Failed uses
    pub failures: u32,
    /// Average confidence
    pub avg_confidence: f32,
}

impl RecipeStats {
    pub fn success_rate(&self) -> f32 {
        if self.uses == 0 {
            1.0
        } else {
            self.successes as f32 / self.uses as f32
        }
    }

    pub fn record_success(&mut self, confidence: f32) {
        self.uses += 1;
        self.successes += 1;
        self.avg_confidence =
            (self.avg_confidence * (self.uses - 1) as f32 + confidence) / self.uses as f32;
    }

    pub fn record_failure(&mut self) {
        self.uses += 1;
        self.failures += 1;
    }
}

/// Recipe store - persistent storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStore {
    /// Recipes by ID
    pub recipes: HashMap<String, LearnedRecipe>,
    /// Index by intent
    pub by_intent: HashMap<String, Vec<String>>,
    /// Last save time
    pub last_saved: u64,
}

impl RecipeStore {
    /// Load from disk
    pub fn load() -> Self {
        let path = store_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&mut self) -> Result<(), String> {
        self.last_saved = current_secs();
        let path = store_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Add or update recipe
    pub fn upsert(&mut self, recipe: LearnedRecipe) {
        let intent_key = format!("{:?}", recipe.intent);

        // Update index
        let ids = self.by_intent.entry(intent_key).or_default();
        if !ids.contains(&recipe.id) {
            ids.push(recipe.id.clone());
        }

        // Store recipe
        self.recipes.insert(recipe.id.clone(), recipe);
    }

    /// Find recipe for intent
    pub fn find_for_intent(&self, intent: CanonicalIntent) -> Option<&LearnedRecipe> {
        let intent_key = format!("{:?}", intent);
        let ids = self.by_intent.get(&intent_key)?;

        // Find best active recipe
        ids.iter()
            .filter_map(|id| self.recipes.get(id))
            .filter(|r| !r.deprecated && r.stats.success_rate() >= 0.5)
            .max_by(|a, b| {
                a.stats
                    .success_rate()
                    .partial_cmp(&b.stats.success_rate())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Get mutable recipe
    pub fn get_mut(&mut self, id: &str) -> Option<&mut LearnedRecipe> {
        self.recipes.get_mut(id)
    }

    /// List all active recipes
    pub fn active_recipes(&self) -> Vec<&LearnedRecipe> {
        self.recipes.values().filter(|r| !r.deprecated).collect()
    }

    /// Get stats summary
    pub fn stats_summary(&self) -> RecipeStoreSummary {
        let active = self.recipes.values().filter(|r| !r.deprecated).count();
        let deprecated = self.recipes.values().filter(|r| r.deprecated).count();
        let total_uses: u32 = self.recipes.values().map(|r| r.stats.uses).sum();
        let total_successes: u32 = self.recipes.values().map(|r| r.stats.successes).sum();

        RecipeStoreSummary {
            total: self.recipes.len(),
            active,
            deprecated,
            total_uses,
            success_rate: if total_uses > 0 {
                total_successes as f32 / total_uses as f32
            } else {
                1.0
            },
        }
    }
}

/// Recipe store summary stats
#[derive(Debug, Clone)]
pub struct RecipeStoreSummary {
    pub total: usize,
    pub active: usize,
    pub deprecated: usize,
    pub total_uses: u32,
    pub success_rate: f32,
}

/// Recipe execution context
#[derive(Debug, Clone)]
pub struct RecipeContext {
    /// Probe outputs
    pub probe_outputs: HashMap<String, String>,
    /// Computed variables
    pub variables: HashMap<String, String>,
}

impl RecipeContext {
    pub fn new() -> Self {
        Self {
            probe_outputs: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn with_probes(probes: HashMap<String, String>) -> Self {
        Self {
            probe_outputs: probes,
            variables: HashMap::new(),
        }
    }
}

impl Default for RecipeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Recipe execution result
#[derive(Debug, Clone)]
pub enum RecipeResult {
    /// Recipe succeeded with answer
    Success {
        answer: RenderedAnswer,
        confidence: f32,
    },
    /// Recipe partially succeeded
    Partial {
        answer: RenderedAnswer,
        confidence: f32,
        missing: Vec<String>,
    },
    /// Recipe failed (fall back to specialist)
    Failed { reason: String },
}

/// Execute a recipe
pub fn execute_recipe(recipe: &LearnedRecipe, ctx: &mut RecipeContext) -> RecipeResult {
    // Check required probes
    for probe in &recipe.required_probes {
        if !ctx.probe_outputs.contains_key(probe) {
            return RecipeResult::Failed {
                reason: format!("Missing required probe: {}", probe),
            };
        }
    }

    // Execute computation steps
    for step in &recipe.steps {
        if let Err(e) = execute_step(step, ctx) {
            return RecipeResult::Failed { reason: e };
        }
    }

    // Determine which answer template to use
    let (template, confidence) = select_answer_template(recipe, ctx);

    // Render answer
    let answer = template.render(&ctx.variables);

    RecipeResult::Success { answer, confidence }
}

/// Execute a single computation step
fn execute_step(step: &RecipeComputeStep, ctx: &mut RecipeContext) -> Result<(), String> {
    match step {
        RecipeComputeStep::Extract {
            probe,
            pattern,
            variable,
        } => {
            let output = ctx
                .probe_outputs
                .get(probe)
                .ok_or_else(|| format!("Probe {} not found", probe))?;

            let re = regex::Regex::new(pattern).map_err(|e| format!("Invalid pattern: {}", e))?;

            if let Some(caps) = re.captures(output) {
                let value = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                ctx.variables.insert(variable.clone(), value.to_string());
            }
            Ok(())
        }

        RecipeComputeStep::Compare {
            variable,
            operator,
            threshold,
            result_var,
        } => {
            let value: f64 = ctx
                .variables
                .get(variable)
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0);

            let result = operator.eval(value, *threshold);
            ctx.variables.insert(result_var.clone(), result.to_string());
            Ok(())
        }

        RecipeComputeStep::Count {
            probe,
            pattern,
            variable,
        } => {
            let output = ctx
                .probe_outputs
                .get(probe)
                .ok_or_else(|| format!("Probe {} not found", probe))?;

            let re = regex::Regex::new(pattern).map_err(|e| format!("Invalid pattern: {}", e))?;

            let count = re.find_iter(output).count();
            ctx.variables.insert(variable.clone(), count.to_string());
            Ok(())
        }

        RecipeComputeStep::IsEmpty { probe, variable } => {
            let output = ctx
                .probe_outputs
                .get(probe)
                .map(|s| s.trim().is_empty())
                .unwrap_or(true);

            ctx.variables.insert(variable.clone(), output.to_string());
            Ok(())
        }

        RecipeComputeStep::ParseNumber {
            source_var,
            target_var,
        } => {
            let source = ctx
                .variables
                .get(source_var)
                .ok_or_else(|| format!("Variable {} not found", source_var))?;

            // Extract first numeric value
            let re = regex::Regex::new(r"[\d.]+").unwrap();
            if let Some(m) = re.find(source) {
                ctx.variables
                    .insert(target_var.clone(), m.as_str().to_string());
            }
            Ok(())
        }
    }
}

/// Select appropriate answer template based on computed variables
fn select_answer_template<'a>(
    recipe: &'a LearnedRecipe,
    ctx: &RecipeContext,
) -> (&'a AnswerTemplate, f32) {
    // Check for critical condition
    if let Some(critical) = &recipe.answer_critical {
        if let Some(is_critical) = ctx.variables.get("is_critical") {
            if is_critical == "true" {
                return (critical, 0.9);
            }
        }
    }

    // Default to ok template
    (&recipe.answer_ok, 0.95)
}

fn store_path() -> PathBuf {
    let base = std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| "/var/lib/anna".to_string());
    PathBuf::from(base).join("learned_recipes.json")
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
    fn test_compare_op() {
        assert!(CompareOp::Gt.eval(10.0, 5.0));
        assert!(!CompareOp::Lt.eval(10.0, 5.0));
        assert!(CompareOp::Ge.eval(10.0, 10.0));
    }

    #[test]
    fn test_template_render() {
        let template = AnswerTemplate {
            summary: "RAM: {used} / {total} GiB ({percent}% used)".to_string(),
            details: vec![],
            evidence: vec!["memory_info".to_string()],
        };

        let mut values = HashMap::new();
        values.insert("used".to_string(), "8".to_string());
        values.insert("total".to_string(), "16".to_string());
        values.insert("percent".to_string(), "50".to_string());

        let rendered = template.render(&values);
        assert_eq!(rendered.summary, "RAM: 8 / 16 GiB (50% used)");
    }

    #[test]
    fn test_recipe_stats() {
        let mut stats = RecipeStats::default();
        stats.record_success(0.9);
        stats.record_success(0.85);
        stats.record_failure();

        assert_eq!(stats.uses, 3);
        assert_eq!(stats.successes, 2);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_recipe_store() {
        let mut store = RecipeStore::default();

        let recipe = LearnedRecipe {
            id: "test-recipe".to_string(),
            name: "Test Recipe".to_string(),
            version: 1,
            intent: CanonicalIntent::CheckDiskUsage,
            domain: "storage".to_string(),
            required_probes: vec!["disk_usage".to_string()],
            optional_probes: vec![],
            steps: vec![],
            answer_ok: AnswerTemplate {
                summary: "Disk is OK".to_string(),
                details: vec![],
                evidence: vec![],
            },
            answer_critical: None,
            answer_partial: None,
            knowledge_topics: vec![],
            source_tickets: vec![],
            stats: RecipeStats::default(),
            created_at: 0,
            last_used_at: 0,
            deprecated: false,
        };

        store.upsert(recipe);
        assert!(store
            .find_for_intent(CanonicalIntent::CheckDiskUsage)
            .is_some());
    }
}
