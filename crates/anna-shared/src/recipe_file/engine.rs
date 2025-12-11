//! Recipe execution engine (v0.0.406).
//!
//! Provides the core recipe functions:
//! - find_matching_recipe(domain, intent, params) -> Option<Recipe>
//! - execute_recipe(recipe, context) -> ExecutionResult
//! - render_answer(recipe, execution_result) -> String

use super::format::{ConfirmLevel, FileRecipe, RecipeStep};
use super::loader::registry;
use crate::rpc::SpecialistDomain;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Context for recipe execution
#[derive(Debug, Clone)]
pub struct RecipeContext {
    /// User's home directory
    pub home_dir: String,
    /// Current working directory
    pub cwd: String,
    /// User ID (for permission checks)
    pub user_id: Option<String>,
    /// Pre-collected probe outputs (from earlier stages)
    pub probe_outputs: HashMap<String, String>,
    /// Whether to actually execute commands (false = dry run)
    pub execute: bool,
    /// Confirmation callback (returns true if user confirms)
    pub confirm_callback: Option<fn(&str) -> bool>,
}

impl Default for RecipeContext {
    fn default() -> Self {
        Self {
            home_dir: std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "/tmp".to_string()),
            user_id: None,
            probe_outputs: HashMap::new(),
            execute: true,
            confirm_callback: None,
        }
    }
}

/// Result of executing a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step ID
    pub id: String,
    /// Command that was run
    pub command: String,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Whether step was skipped (condition not met or dry run)
    pub skipped: bool,
    /// Extracted variables from output
    pub extracted: HashMap<String, String>,
}

/// Result of executing an entire recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Recipe ID that was executed
    pub recipe_id: String,
    /// Individual step results
    pub steps: Vec<StepResult>,
    /// Accumulated variables for template rendering
    pub variables: HashMap<String, String>,
    /// Whether all steps succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Whether confirmation was needed
    pub confirmation_required: bool,
    /// Whether execution was a dry run
    pub dry_run: bool,
}

impl ExecutionResult {
    /// Create a successful empty result
    pub fn empty(recipe_id: String) -> Self {
        Self {
            recipe_id,
            steps: vec![],
            variables: HashMap::new(),
            success: true,
            error: None,
            total_duration_ms: 0,
            confirmation_required: false,
            dry_run: false,
        }
    }

    /// Create a failed result
    pub fn failed(recipe_id: String, error: impl Into<String>) -> Self {
        Self {
            recipe_id,
            steps: vec![],
            variables: HashMap::new(),
            success: false,
            error: Some(error.into()),
            total_duration_ms: 0,
            confirmation_required: false,
            dry_run: false,
        }
    }
}

/// Recipe match result
#[derive(Debug, Clone)]
pub struct RecipeMatchResult {
    /// The matched recipe
    pub recipe: FileRecipe,
    /// Match confidence (0-100)
    pub confidence: u8,
    /// Which criteria matched
    pub matched_criteria: Vec<String>,
}

/// Find a matching recipe for the given domain, intent, and params
pub fn find_matching_recipe(
    domain: SpecialistDomain,
    intent: &str,
    params: &HashMap<String, String>,
    query: &str,
) -> Option<RecipeMatchResult> {
    let mut reg = registry();
    let recipes = reg.load();

    let domain_str = domain.to_string().to_lowercase();
    let intent_lower = intent.to_lowercase();
    let query_lower = query.to_lowercase();

    let mut best_match: Option<RecipeMatchResult> = None;
    let mut best_score = 0u32;

    for recipe in recipes.values() {
        // Domain must match
        if recipe.id.domain.to_lowercase() != domain_str {
            continue;
        }

        // Intent must match
        if recipe.match_criteria.intent.to_lowercase() != intent_lower {
            continue;
        }

        let mut score = 50u32; // Base score for domain + intent match
        let mut matched_criteria = vec!["domain".to_string(), "intent".to_string()];

        // Check key match
        if let Some(ref key) = recipe.match_criteria.key {
            if query_lower.contains(&key.to_lowercase()) {
                score += 20;
                matched_criteria.push(format!("key:{}", key));
            }
        }

        // Check target match
        if let Some(ref target) = recipe.match_criteria.target {
            if let Some(param_target) = params.get("target") {
                if param_target.to_lowercase() == target.to_lowercase() {
                    score += 15;
                    matched_criteria.push(format!("target:{}", target));
                }
            }
        }

        // Check keyword matches (any)
        for kw in &recipe.match_criteria.keywords {
            if query_lower.contains(&kw.to_lowercase()) {
                score += 5;
                matched_criteria.push(format!("keyword:{}", kw));
            }
        }

        // Check required keywords (all must match)
        let all_required_match = recipe.match_criteria.required_keywords.iter()
            .all(|kw| query_lower.contains(&kw.to_lowercase()));
        if !recipe.match_criteria.required_keywords.is_empty() {
            if all_required_match {
                score += 25;
                matched_criteria.push("all_required_keywords".to_string());
            } else {
                // Skip this recipe if required keywords don't match
                continue;
            }
        }

        // Check param matches
        for (key, value) in &recipe.match_criteria.params {
            if let Some(param_value) = params.get(key) {
                if param_value.to_lowercase() == value.to_lowercase() {
                    score += 10;
                    matched_criteria.push(format!("param:{}={}", key, value));
                }
            }
        }

        // Convert score to confidence (0-100)
        let confidence = (score.min(100)) as u8;

        // Check minimum confidence threshold
        if confidence < recipe.match_criteria.min_confidence {
            continue;
        }

        if score > best_score {
            best_score = score;
            best_match = Some(RecipeMatchResult {
                recipe: recipe.clone(),
                confidence,
                matched_criteria,
            });
        }
    }

    if let Some(ref m) = best_match {
        info!(
            "Recipe match: {} (confidence={}%, criteria={:?})",
            m.recipe.full_id(),
            m.confidence,
            m.matched_criteria
        );
    }

    best_match
}

/// Execute a recipe with the given context
pub fn execute_recipe(
    recipe: &FileRecipe,
    context: &RecipeContext,
    probe_lookup: impl Fn(&str) -> Option<String>,
) -> ExecutionResult {
    let start = std::time::Instant::now();
    let mut result = ExecutionResult::empty(recipe.full_id());
    result.dry_run = !context.execute;
    result.confirmation_required = recipe.requires_confirmation();

    // Create backups if needed
    if context.execute && !recipe.plan.backup_paths.is_empty() {
        for path in &recipe.plan.backup_paths {
            let expanded = expand_path(path, &context.home_dir);
            if std::path::Path::new(&expanded).exists() {
                let backup = format!("{}.anna-backup", expanded);
                if let Err(e) = std::fs::copy(&expanded, &backup) {
                    warn!("Failed to backup {}: {}", expanded, e);
                } else {
                    debug!("Created backup: {}", backup);
                }
            }
        }
    }

    // Execute steps
    for step in &recipe.plan.steps {
        // Check condition
        if let Some(ref cond) = step.condition {
            if !evaluate_condition(cond, &result.variables) {
                result.steps.push(StepResult {
                    id: step.id.clone(),
                    command: String::new(),
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: 0,
                    skipped: true,
                    extracted: HashMap::new(),
                });
                continue;
            }
        }

        // Get command
        let command = match step.get_command(&probe_lookup) {
            Some(cmd) => expand_path(&cmd, &context.home_dir),
            None => {
                result.error = Some(format!("Step {} has no command", step.id));
                result.success = false;
                break;
            }
        };

        // Check if already have probe output
        if let Some(ref probe_id) = step.probe {
            if let Some(output) = context.probe_outputs.get(probe_id) {
                let step_result = StepResult {
                    id: step.id.clone(),
                    command: command.clone(),
                    stdout: output.clone(),
                    stderr: String::new(),
                    exit_code: 0,
                    duration_ms: 0,
                    skipped: false,
                    extracted: extract_variables(&step.extract, output),
                };
                result.variables.extend(step_result.extracted.clone());
                result.steps.push(step_result);
                continue;
            }
        }

        // Handle confirmation
        if step.needs_confirm != ConfirmLevel::None && context.execute {
            let desc = step.description.as_deref().unwrap_or(&command);
            let confirmed = context.confirm_callback
                .map(|cb| cb(desc))
                .unwrap_or(false);

            if !confirmed {
                result.steps.push(StepResult {
                    id: step.id.clone(),
                    command,
                    stdout: String::new(),
                    stderr: "User declined confirmation".to_string(),
                    exit_code: -1,
                    duration_ms: 0,
                    skipped: true,
                    extracted: HashMap::new(),
                });
                result.error = Some("User declined confirmation".to_string());
                result.success = false;
                break;
            }
        }

        // Execute command
        let step_result = if context.execute {
            execute_step(step, &command)
        } else {
            // Dry run
            StepResult {
                id: step.id.clone(),
                command,
                stdout: "[dry run]".to_string(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 0,
                skipped: false,
                extracted: HashMap::new(),
            }
        };

        // Check for errors
        if step_result.exit_code != 0 && recipe.plan.stop_on_error {
            result.error = Some(format!(
                "Step {} failed with exit code {}",
                step.id, step_result.exit_code
            ));
            result.success = false;
            result.steps.push(step_result);
            break;
        }

        // Accumulate variables
        result.variables.extend(step_result.extracted.clone());
        result.steps.push(step_result);
    }

    result.total_duration_ms = start.elapsed().as_millis() as u64;
    result
}

/// Execute a single step
fn execute_step(step: &RecipeStep, command: &str) -> StepResult {
    let start = std::time::Instant::now();

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    let (stdout, stderr, exit_code) = match output {
        Ok(out) => (
            String::from_utf8_lossy(&out.stdout).to_string(),
            String::from_utf8_lossy(&out.stderr).to_string(),
            out.status.code().unwrap_or(-1),
        ),
        Err(e) => (String::new(), e.to_string(), -1),
    };

    let extracted = extract_variables(&step.extract, &stdout);

    StepResult {
        id: step.id.clone(),
        command: command.to_string(),
        stdout,
        stderr,
        exit_code,
        duration_ms: start.elapsed().as_millis() as u64,
        skipped: false,
        extracted,
    }
}

/// Extract variables from output using regex patterns
fn extract_variables(patterns: &HashMap<String, String>, output: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    for (var_name, pattern) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(output) {
                if let Some(m) = caps.get(1) {
                    vars.insert(var_name.clone(), m.as_str().to_string());
                } else if let Some(m) = caps.get(0) {
                    vars.insert(var_name.clone(), m.as_str().to_string());
                }
            }
        }
    }

    vars
}

/// Expand ~ in paths
fn expand_path(path: &str, home: &str) -> String {
    if path.starts_with("~/") {
        format!("{}/{}", home, &path[2..])
    } else if path == "~" {
        home.to_string()
    } else {
        path.to_string()
    }
}

/// Evaluate a simple condition
fn evaluate_condition(condition: &str, variables: &HashMap<String, String>) -> bool {
    // Simple conditions like "prev_exit_code == 0" or "var_name"
    if condition.contains("==") {
        let parts: Vec<&str> = condition.split("==").collect();
        if parts.len() == 2 {
            let var = parts[0].trim();
            let val = parts[1].trim();
            return variables.get(var).map(|v| v == val).unwrap_or(false);
        }
    }

    // Just check if variable exists and is non-empty
    variables.get(condition.trim()).map(|v| !v.is_empty()).unwrap_or(false)
}

/// Render the answer template with execution results
pub fn render_answer(recipe: &FileRecipe, result: &ExecutionResult) -> String {
    let mut answer = recipe.answer.template.clone();

    // Apply defaults first
    for (key, default) in &recipe.answer.defaults {
        let placeholder = format!("{{{}}}", key);
        if answer.contains(&placeholder) && !result.variables.contains_key(key) {
            answer = answer.replace(&placeholder, default);
        }
    }

    // Apply extracted variables
    for (key, value) in &result.variables {
        let placeholder = format!("{{{}}}", key);
        answer = answer.replace(&placeholder, value);
    }

    // Add raw output if requested
    if recipe.answer.include_raw_output {
        let mut raw = String::new();
        for step in &result.steps {
            if !step.skipped && !step.stdout.is_empty() {
                raw.push_str(&format!("\n[{}]\n{}", step.id, step.stdout));
            }
        }
        if !raw.is_empty() {
            answer.push_str(&format!("\n\nRaw output:{}", raw));
        }
    }

    answer.trim().to_string()
}

/// Recipe engine combining all operations
#[derive(Debug, Default)]
pub struct RecipeEngine {
    /// Probe lookup function registry
    probe_commands: HashMap<String, String>,
}

impl RecipeEngine {
    /// Create a new engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a probe command
    pub fn register_probe(&mut self, id: &str, command: &str) {
        self.probe_commands.insert(id.to_string(), command.to_string());
    }

    /// Find and execute matching recipe
    pub fn run(
        &self,
        domain: SpecialistDomain,
        intent: &str,
        params: &HashMap<String, String>,
        query: &str,
        context: &RecipeContext,
    ) -> Option<(ExecutionResult, String)> {
        let match_result = find_matching_recipe(domain, intent, params, query)?;

        let probe_lookup = |id: &str| self.probe_commands.get(id).cloned();
        let exec_result = execute_recipe(&match_result.recipe, context, probe_lookup);

        if exec_result.success {
            let answer = render_answer(&match_result.recipe, &exec_result);
            Some((exec_result, answer))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        assert_eq!(expand_path("~/test", "/home/user"), "/home/user/test");
        assert_eq!(expand_path("~", "/home/user"), "/home/user");
        assert_eq!(expand_path("/absolute", "/home/user"), "/absolute");
    }

    #[test]
    fn test_extract_variables() {
        let mut patterns = HashMap::new();
        patterns.insert("count".to_string(), r"(\d+) failed".to_string());

        let output = "There are 5 failed services";
        let vars = extract_variables(&patterns, output);
        assert_eq!(vars.get("count"), Some(&"5".to_string()));
    }

    #[test]
    fn test_evaluate_condition() {
        let mut vars = HashMap::new();
        vars.insert("exit_code".to_string(), "0".to_string());
        vars.insert("found".to_string(), "yes".to_string());

        assert!(evaluate_condition("exit_code == 0", &vars));
        assert!(!evaluate_condition("exit_code == 1", &vars));
        assert!(evaluate_condition("found", &vars));
        assert!(!evaluate_condition("missing", &vars));
    }

    #[test]
    fn test_render_answer() {
        use crate::recipe_file::format::*;

        let recipe = FileRecipe {
            id: RecipeId {
                name: "test".to_string(),
                domain: "system".to_string(),
                version: "1".to_string(),
            },
            match_criteria: RecipeMatch {
                intent: "diagnose".to_string(),
                keywords: vec![],
                required_keywords: vec![],
                key: None,
                target: None,
                params: HashMap::new(),
                min_confidence: 60,
            },
            plan: RecipePlan {
                steps: vec![],
                stop_on_error: true,
                backup_paths: vec![],
            },
            answer: RecipeAnswer {
                template: "Found {count} items (default: {missing})".to_string(),
                defaults: [("missing".to_string(), "none".to_string())]
                    .into_iter()
                    .collect(),
                include_raw_output: false,
            },
            meta: Default::default(),
        };

        let mut result = ExecutionResult::empty("test".to_string());
        result.variables.insert("count".to_string(), "5".to_string());

        let answer = render_answer(&recipe, &result);
        assert_eq!(answer, "Found 5 items (default: none)");
    }
}
