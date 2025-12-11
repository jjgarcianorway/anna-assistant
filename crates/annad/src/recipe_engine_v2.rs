//! Recipe Engine v2 Integration (v0.0.412).
//!
//! Integrates the new learned recipe system with the existing fast path.
//! Checks learned recipes BEFORE hardcoded ones to maximize self-learning.

use anna_shared::recipe_engine::{Recipe as LearnedRecipe, RecipeKind as LearnedKind};
use anna_shared::recipe_executor::{ExecutionContext, ExecutionResult, RecipeExecutor};
use anna_shared::recipe_store_v2::{MatchType, RecipeMatch, RecipeStoreV2};
use anna_shared::recipe_templates;
use anna_shared::rpc::{
    EvidenceBlock, QueryIntent, ReliabilitySignals, ServiceDeskResult, SpecialistDomain,
    TranslatorTicket,
};
use anna_shared::trace::{ExecutionTrace, ProbeStats};
use anna_shared::transcript::Transcript;
use std::collections::HashMap;
use std::sync::OnceLock;
use tracing::{debug, info, warn};

/// Global recipe store (lazy loaded)
static RECIPE_STORE: OnceLock<std::sync::RwLock<RecipeStoreV2>> = OnceLock::new();

/// Get or initialize the recipe store
pub fn get_store() -> &'static std::sync::RwLock<RecipeStoreV2> {
    RECIPE_STORE.get_or_init(|| {
        let mut store = RecipeStoreV2::load();
        // Initialize with generic templates if empty
        if store.is_empty() {
            recipe_templates::initialize_store(&mut store);
            let _ = store.save();
            info!("Initialized recipe store with {} generic templates", store.len());
        }
        // v0.0.412: Run GC on startup
        store.gc();
        let _ = store.save();
        std::sync::RwLock::new(store)
    })
}

/// Trigger garbage collection manually
pub fn run_gc() {
    let store = get_store();
    if let Ok(mut s) = store.write() {
        s.gc();
        let _ = s.save();
        info!("Recipe store GC completed");
    }
}

/// Minimum score to use learned recipe
const LEARNED_RECIPE_THRESHOLD: f32 = 0.7;

/// Result of checking learned recipes
#[derive(Debug)]
pub struct LearnedRecipeResult {
    pub matched: bool,
    pub recipe_id: Option<String>,
    pub recipe_name: Option<String>,
    pub score: f32,
    pub match_type: Option<MatchType>,
    pub can_execute: bool,
    pub params_needed: Vec<String>,
}

impl LearnedRecipeResult {
    fn no_match() -> Self {
        Self {
            matched: false,
            recipe_id: None,
            recipe_name: None,
            score: 0.0,
            match_type: None,
            can_execute: false,
            params_needed: vec![],
        }
    }
}

/// Check learned recipes for a match
pub fn check_learned_recipes(query: &str, domain: Option<&str>) -> LearnedRecipeResult {
    let store = get_store();
    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return LearnedRecipeResult::no_match(),
    };

    // Find best match
    if let Some(m) = store.best_match(query, domain, LEARNED_RECIPE_THRESHOLD) {
        if let Some(recipe) = store.get(&m.recipe_id) {
            let params_needed = extract_missing_params(query, recipe);
            let can_execute = params_needed.is_empty() || has_default_params(recipe);

            info!(
                "Learned recipe match: {} (score={:.2}, type={:?})",
                recipe.name, m.score, m.match_type
            );

            return LearnedRecipeResult {
                matched: true,
                recipe_id: Some(recipe.id.clone()),
                recipe_name: Some(recipe.name.clone()),
                score: m.score,
                match_type: Some(m.match_type),
                can_execute,
                params_needed,
            };
        }
    }

    LearnedRecipeResult::no_match()
}

/// Extract params that need to be filled from query
fn extract_missing_params(query: &str, recipe: &LearnedRecipe) -> Vec<String> {
    let query_lower = query.to_lowercase();
    let mut missing = vec![];

    for param in &recipe.parameters {
        if param.required && param.default.is_none() {
            // Try to extract from query
            let extracted = extract_param_value(&query_lower, &param.name, &param.extraction_hint);
            if extracted.is_none() {
                missing.push(param.name.clone());
            }
        }
    }

    missing
}

/// Check if recipe has defaults for all required params
fn has_default_params(recipe: &LearnedRecipe) -> bool {
    recipe.parameters
        .iter()
        .filter(|p| p.required)
        .all(|p| p.default.is_some())
}

/// Try to extract a parameter value from query
pub fn extract_param_value(query: &str, param_name: &str, hint: &str) -> Option<String> {
    let words: Vec<&str> = query.split_whitespace().collect();

    // Common extraction patterns
    match param_name {
        "service_name" | "service" => {
            // Look for word before "service" or known service names
            let services = ["nginx", "sshd", "httpd", "docker", "mysql", "postgresql", "redis"];
            for word in &words {
                let w = word.trim_matches(|c: char| !c.is_alphanumeric());
                if services.contains(&w) {
                    return Some(w.to_string());
                }
            }
            // Word before "service"
            if let Some(pos) = words.iter().position(|&w| w == "service") {
                if pos > 0 {
                    return Some(words[pos - 1].to_string());
                }
            }
        }
        "package_name" | "package" => {
            // Word after "install", "remove", "is X installed"
            if let Some(pos) = words.iter().position(|&w| w == "installed" || w == "install") {
                if words.len() > pos + 1 {
                    return Some(words[pos + 1].to_string());
                }
                if pos > 0 && words[pos] == "installed" {
                    return Some(words[pos - 1].to_string());
                }
            }
        }
        "mount_path" | "path" => {
            // Look for path-like strings
            for word in &words {
                if word.starts_with('/') || word.starts_with("~/") {
                    return Some(word.to_string());
                }
            }
        }
        _ => {}
    }

    None
}

/// Execute a learned recipe
pub fn execute_learned_recipe(
    recipe_id: &str,
    query: &str,
    request_id: &str,
) -> Option<ServiceDeskResult> {
    let store = get_store();
    let mut store = match store.write() {
        Ok(s) => s,
        Err(_) => return None,
    };

    let recipe = store.get(recipe_id)?.clone();

    // Build execution context with extracted params
    let mut ctx = ExecutionContext::default();
    ctx.ticket_id = Some(request_id.to_string());
    ctx.recipe_id = Some(recipe_id.to_string());

    // Extract parameters from query
    for param in &recipe.parameters {
        if let Some(value) = extract_param_value(&query.to_lowercase(), &param.name, &param.extraction_hint) {
            ctx.params.insert(param.name.clone(), value);
        } else if let Some(default) = &param.default {
            ctx.params.insert(param.name.clone(), default.clone());
        }
    }

    // Execute
    let executor = RecipeExecutor::new();
    let result = executor.execute(&recipe, &mut ctx);

    // Update recipe stats
    if let Some(r) = store.get_mut(recipe_id) {
        if result.success {
            r.record_success();
        } else {
            r.record_failure();
        }
    }
    let _ = store.save();

    // Build result
    Some(build_learned_recipe_result(
        request_id.to_string(),
        &recipe,
        result,
        query,
    ))
}

/// Build ServiceDeskResult from recipe execution
fn build_learned_recipe_result(
    request_id: String,
    recipe: &LearnedRecipe,
    exec_result: ExecutionResult,
    _query: &str,
) -> ServiceDeskResult {
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: !recipe.required_evidence.is_empty(),
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: true,
    };

    let domain = match recipe.domain.to_lowercase().as_str() {
        "services" | "system" => SpecialistDomain::System,
        "storage" => SpecialistDomain::Storage,
        "network" => SpecialistDomain::Network,
        "packages" => SpecialistDomain::Packages,
        "desktop" => SpecialistDomain::Desktop,
        _ => SpecialistDomain::System,
    };

    let trace = ExecutionTrace::deterministic_route(
        &format!("learned_recipe:{}", recipe.id),
        ProbeStats::default(),
        vec![],
    );

    // Build answer with sources
    let mut answer = exec_result.answer;
    if !recipe.doc_sources.is_empty() {
        answer.push_str("\n\n**Sources:**\n");
        for src in &recipe.doc_sources {
            answer.push_str(&format!("- {}\n", src));
        }
    }
    answer.push_str(&format!(
        "\n*Used learned recipe: {} (success rate: {:.0}%)*",
        recipe.name,
        recipe.success_rate() * 100.0
    ));

    ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: Some("Anna (Recipe Engine)".to_string()),
        staff_id: Some("recipe_engine".to_string()),
        answer,
        validated: exec_result.success,
        reliability_score: if exec_result.success { 90 } else { 40 },
        reliability_signals: signals,
        reliability_explanation: None,
        domain,
        evidence: EvidenceBlock::default(),
        needs_clarification: false,
        clarification_question: None,
        clarification_request: None,
        transcript: Transcript::new(),
        execution_trace: Some(trace),
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    }
}

/// Get recipe store stats
pub fn get_recipe_stats() -> String {
    let store = get_store();
    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return "Failed to read recipe store".to_string(),
    };

    format!("{}", store.stats())
}

/// List all recipes (for annactl)
pub fn list_recipes() -> Vec<RecipeSummary> {
    let store = get_store();
    let store = match store.read() {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    store.recipes
        .values()
        .map(|r| RecipeSummary {
            id: r.id.clone(),
            name: r.name.clone(),
            domain: r.domain.clone(),
            kind: r.kind.to_string(),
            use_count: r.use_count,
            success_rate: r.success_rate(),
            deprecated: r.deprecated,
            doc_sources: r.doc_sources.clone(),
        })
        .collect()
}

/// Recipe summary for listing
#[derive(Debug, Clone)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub kind: String,
    pub use_count: u32,
    pub success_rate: f32,
    pub deprecated: bool,
    pub doc_sources: Vec<String>,
}

impl std::fmt::Display for RecipeSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.deprecated { "[DEP]" } else { "[ACT]" };
        writeln!(
            f,
            "{} {} ({}) - {}",
            status, self.name, self.id, self.domain
        )?;
        writeln!(
            f,
            "    Uses: {}, Success: {:.0}%, Kind: {}",
            self.use_count,
            self.success_rate * 100.0,
            self.kind
        )?;
        if !self.doc_sources.is_empty() {
            writeln!(f, "    Sources: {}", self.doc_sources.join(", "))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_service_name() {
        assert_eq!(
            extract_param_value("why is nginx service failing", "service_name", ""),
            Some("nginx".to_string())
        );
        assert_eq!(
            extract_param_value("check sshd service status", "service_name", ""),
            Some("sshd".to_string())
        );
    }

    #[test]
    fn test_extract_package_name() {
        assert_eq!(
            extract_param_value("is vim installed", "package_name", ""),
            Some("vim".to_string())
        );
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            extract_param_value("check disk usage on /home", "mount_path", ""),
            Some("/home".to_string())
        );
    }
}
