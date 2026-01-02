//! Recipe learning from specialist solutions.
//!
//! This module handles:
//! - Creating recipes from successful specialist solutions
//! - Detecting dynamic queries that shouldn't be learned as recipes
//! - Managing recipe origin metadata

use anna_shared::learning_engine::{
    AnswerKind, AnswerTemplate, LearnedRecipe, LogicType, RecipeLogic,
    RecipeOrigin, RecipePattern, RecipeProbe,
};
use std::collections::HashMap;
use tracing::info;

use super::types::{ParsedQuery, SpecialistSolution};

/// Create a recipe from a successful specialist solution
pub fn create_recipe_from_solution(
    parsed: &ParsedQuery,
    _evidence: &HashMap<String, String>,
    solution: &SpecialistSolution,
) -> LearnedRecipe {
    let id = format!("{}-{}", parsed.intent, chrono::Utc::now().timestamp());

    let mut recipe = LearnedRecipe::new(&id, &parsed.domain);

    // Set pattern from intent keywords
    let keywords: Vec<&str> = parsed
        .intent
        .split('_')
        .filter(|s| !s.is_empty())
        .collect();
    recipe.pattern = RecipePattern::new(&parsed.intent).with_keywords(&keywords);

    // Set probes from what we used
    for probe_cmd in &parsed.probes {
        recipe
            .probes
            .push(RecipeProbe::new(probe_cmd, probe_cmd));
    }

    // Set answer template
    recipe.answer_template = AnswerTemplate::new(
        &solution.answer,
        &format!("{}\n\n(Learned from specialist)", solution.answer),
    );

    // Set logic
    recipe.logic = RecipeLogic {
        logic_type: LogicType::Template,
        answer_kind: AnswerKind::Diagnostic,
        steps: vec![solution.explanation.clone()],
        conditionals: HashMap::new(),
    };

    // Set origin
    recipe.origin =
        RecipeOrigin::learned("specialist", &format!("{} Specialist", parsed.domain));

    recipe
}

/// v0.0.816: Check if domain produces dynamic (changing) results
/// These queries should NOT be learned as recipes because the answer changes
pub fn is_dynamic_domain(domain: &str) -> bool {
    matches!(
        domain.to_lowercase().as_str(),
        "storage" | "memory" | "performance" | "system" | "processes"
    )
}

/// v0.0.816: Check if probe produces dynamic (changing) results
pub fn is_dynamic_probe(probe: &str) -> bool {
    let dynamic_probes = [
        "largest_dirs", "largest_home", "disk_usage", "df",
        "free", "memory_info", "top_memory", "top_cpu",
        "ps", "uptime", "load_average", "who",
        "running_services", "failed_services",
        "network_stats", "listening_ports",
    ];

    dynamic_probes.iter().any(|p| probe.contains(p))
}

/// Check if a query is dynamic (shouldn't be learned as recipe)
pub fn is_dynamic_query(parsed: &ParsedQuery) -> bool {
    is_dynamic_domain(&parsed.domain) ||
        parsed.probes.iter().any(|p| is_dynamic_probe(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dynamic_domain() {
        assert!(is_dynamic_domain("storage"));
        assert!(is_dynamic_domain("memory"));
        assert!(is_dynamic_domain("performance"));
        assert!(is_dynamic_domain("system"));
        assert!(is_dynamic_domain("processes"));
        assert!(!is_dynamic_domain("network"));
        assert!(!is_dynamic_domain("desktop"));
    }

    #[test]
    fn test_is_dynamic_probe() {
        assert!(is_dynamic_probe("free -h"));
        assert!(is_dynamic_probe("df -h"));
        assert!(is_dynamic_probe("ps aux"));
        assert!(!is_dynamic_probe("uname -r"));
        assert!(!is_dynamic_probe("which vim"));
    }
}
