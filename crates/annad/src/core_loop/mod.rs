//! Core request loop - the simple path (v0.0.816).
//!
//! This module implements the VISION.md core loop:
//!
//! ```text
//! User Query → Translator → Check Recipes →
//!   If found: Execute recipe → Answer
//!   If not: Knowledge lookup → Specialist solves → Anna learns → Answer
//! ```
//!
//! This is intentionally simple. No 10 special-case handlers.
//! The translator understands the query, recipes handle known patterns,
//! specialists handle new problems and teach Anna.
//!
//! v0.0.812: Added IT Department with named specialists.
//! v0.0.813: Added knowledge lookup (Arch Wiki, man pages, --help).
//! v0.0.815: Added stats tracking for recipe hits vs LLM calls.
//! v0.0.816: Don't learn recipes for dynamic queries (storage, memory, etc).

use anna_shared::doc_fetcher;
use anna_shared::learning_engine::RecipeLibrary;
use std::time::Instant;
use tracing::{info, warn};

use crate::specialists::{ITDepartment, SpecialistRole};
use crate::state::SharedState;

// Submodules
mod types;
mod translator;
mod recipe_handler;
mod specialist_handler;
mod learning;
mod helpers;

// Public exports
pub use types::{AnswerSource, CoreLoopResult, InternalComm, ParsedQuery};

// Internal imports
use types::SpecialistSolution;
use translator::{translate_query, extract_knowledge_tags};
use recipe_handler::{find_matching_recipe, execute_recipe};
use specialist_handler::ask_specialist;
use learning::{create_recipe_from_solution, is_dynamic_query};
use helpers::gather_evidence;

/// The core loop - simple and clear
pub async fn handle_query(state: SharedState, query: &str) -> CoreLoopResult {
    let start = Instant::now();
    let mut comms: Vec<InternalComm> = Vec::new();

    info!("Core loop: query=\"{}\"", query);

    // Step 1: Translate query to structured form
    let translator_model = {
        let s = state.read().await;
        s.config.llm.translator_model.clone()
    };

    comms.push(InternalComm {
        from: "Anna".to_string(),
        message: "Analyzing your query...".to_string(),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    let parsed = match translate_query(&translator_model, query).await {
        Ok(p) => p,
        Err(e) => {
            warn!("Translator failed: {}", e);
            return CoreLoopResult {
                answer: format!("I couldn't understand that query: {}", e),
                source: AnswerSource::Failed,
                recipe_id: None,
                reliability: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
                internal_comms: comms,
            };
        }
    };

    info!(
        "Translated: intent={}, domain={}, probes={:?}",
        parsed.intent, parsed.domain, parsed.probes
    );

    comms.push(InternalComm {
        from: "Anna".to_string(),
        message: format!(
            "Understood: {} query about {}",
            parsed.intent, parsed.domain
        ),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    // Step 2: Check recipes
    let recipe_path = RecipeLibrary::default_path();
    let mut library = RecipeLibrary::load(&recipe_path).unwrap_or_default();

    if let Some(recipe) = find_matching_recipe(&library, &parsed) {
        info!("Recipe found: {}", recipe.id);

        comms.push(InternalComm {
            from: "Anna".to_string(),
            message: format!("Found recipe '{}' - executing", recipe.id),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        // Execute recipe
        let answer = execute_recipe(&recipe, &parsed).await;

        // Record success and update stats
        library.record_success(&recipe.id);
        let _ = library.save(&recipe_path);

        // v0.0.815: Record recipe hit in global stats
        {
            let mut s = state.write().await;
            s.stats.record_request_received();
            s.stats.record_recipe_hit();
        }

        return CoreLoopResult {
            answer,
            source: AnswerSource::Recipe,
            recipe_id: Some(recipe.id.clone()),
            reliability: 90,
            elapsed_ms: start.elapsed().as_millis() as u64,
            internal_comms: comms,
        };
    }

    info!("No recipe found, asking specialist");

    // Step 3: Get specialist from IT Department
    let (junior_model, senior_model) = {
        let s = state.read().await;
        (
            s.config.llm.translator_model.clone(), // Junior uses lighter model
            s.config.llm.specialist_model.clone(), // Senior uses deeper model
        )
    };

    let it_dept = ITDepartment::new(&junior_model, &senior_model);
    let team = it_dept.get_team(&parsed.domain);

    // Get the junior specialist first (per VISION.md escalation flow)
    let (specialist, specialist_name) = match team {
        Some(t) => (&t.junior, ITDepartment::display_name(&t.junior)),
        None => {
            // Fallback to system team if domain not found
            let system_team = it_dept.get_team("system").unwrap();
            (&system_team.junior, ITDepartment::display_name(&system_team.junior))
        }
    };

    comms.push(InternalComm {
        from: "Anna".to_string(),
        message: format!(
            "No recipe for '{}', asking {} for help",
            parsed.intent, specialist_name
        ),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    // Gather evidence first
    let evidence = gather_evidence(&parsed.probes).await;

    comms.push(InternalComm {
        from: specialist_name.clone(),
        message: format!("Gathered {} pieces of evidence", evidence.len()),
        elapsed_ms: start.elapsed().as_millis() as u64,
    });

    // Step 3.5: Look up knowledge sources (Arch Wiki, man pages, --help)
    let knowledge_tags = extract_knowledge_tags(&parsed);
    let knowledge = doc_fetcher::fetch_docs(&knowledge_tags, 3);

    if !knowledge.is_empty() {
        let sources: Vec<_> = knowledge.iter().map(|k| k.title.clone()).collect();
        comms.push(InternalComm {
            from: specialist_name.clone(),
            message: format!("Found {} knowledge sources: {}", knowledge.len(), sources.join(", ")),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Junior specialist attempts first
    let mut solution = ask_specialist(&specialist.model, query, &parsed, &evidence, &knowledge, &specialist_name).await;

    // If junior has low confidence, escalate to senior
    let mut final_specialist_name = specialist_name.clone();
    if solution.confidence < 0.7 {
        info!("Junior low confidence ({}), escalating to senior", solution.confidence);

        // Get senior specialist
        let (senior, senior_name) = match team {
            Some(t) => (&t.senior, ITDepartment::display_name(&t.senior)),
            None => {
                let system_team = it_dept.get_team("system").unwrap();
                (&system_team.senior, ITDepartment::display_name(&system_team.senior))
            }
        };

        comms.push(InternalComm {
            from: specialist_name.clone(),
            message: format!(
                "I'm not confident enough ({}%), escalating to {}",
                (solution.confidence * 100.0) as u8,
                senior_name
            ),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        // Senior specialist takes over
        solution = ask_specialist(&senior.model, query, &parsed, &evidence, &knowledge, &senior_name).await;
        final_specialist_name = senior_name.clone();

        comms.push(InternalComm {
            from: senior_name,
            message: format!(
                "Analyzed the problem. Confidence: {}%",
                (solution.confidence * 100.0) as u8
            ),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });
    }

    // If still low confidence after senior, return with warning
    if solution.confidence < 0.5 {
        warn!("Even senior has low confidence: {}", solution.confidence);

        comms.push(InternalComm {
            from: final_specialist_name.clone(),
            message: "I'm not confident in this answer, might need more investigation".to_string(),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        return CoreLoopResult {
            answer: solution.answer,
            source: AnswerSource::Specialist {
                name: final_specialist_name,
                learned: false,
            },
            recipe_id: None,
            reliability: (solution.confidence * 100.0) as u8,
            elapsed_ms: start.elapsed().as_millis() as u64,
            internal_comms: comms,
        };
    }

    // Step 4: Learn recipe from successful solution
    // v0.0.816: Only learn recipes for NON-DYNAMIC queries
    // Dynamic queries (disk space, memory, processes) return different values each time
    // so storing the literal answer as a recipe is wrong.
    let learned = if solution.confidence >= 0.8 && !is_dynamic_query(&parsed) {
        let recipe = create_recipe_from_solution(&parsed, &evidence, &solution);
        info!("Learning new recipe: {}", recipe.id);

        comms.push(InternalComm {
            from: "Anna".to_string(),
            message: format!("Learning recipe '{}' for next time", recipe.id),
            elapsed_ms: start.elapsed().as_millis() as u64,
        });

        match library.add(recipe.clone()) {
            Ok(_) => {
                let _ = library.save(&recipe_path);
                true
            }
            Err(e) => {
                warn!("Failed to save recipe: {}", e);
                false
            }
        }
    } else {
        if is_dynamic_query(&parsed) {
            info!("Skipping recipe learning for dynamic query (domain={}, probes={:?})",
                  parsed.domain, parsed.probes);
        }
        false
    };

    // v0.0.815: Record LLM usage in global stats (this path used specialist, not recipe)
    {
        let mut s = state.write().await;
        s.stats.record_request_received();
        // Note: NOT a recipe_hit - this went to LLM specialist
    }

    CoreLoopResult {
        answer: solution.answer,
        source: AnswerSource::Specialist {
            name: final_specialist_name,
            learned,
        },
        recipe_id: None,
        reliability: (solution.confidence * 100.0) as u8,
        elapsed_ms: start.elapsed().as_millis() as u64,
        internal_comms: comms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parsed_query() {
        let parsed = ParsedQuery {
            intent: "check_ram".to_string(),
            domain: "system".to_string(),
            probes: vec!["free -h".to_string()],
            entities: HashMap::new(),
        };

        assert_eq!(parsed.intent, "check_ram");
        assert_eq!(parsed.probes.len(), 1);
    }
}
