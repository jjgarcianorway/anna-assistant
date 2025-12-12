//! Routing stage for the RPC handler pipeline.
//!
//! v0.0.167: Extracted from rpc_handler.rs for modularization.
//! v0.0.392: Simplified - translator is primary, removed semantic similarity.

use anna_shared::probe_spine::{
    enforce_minimum_probes, enforce_spine_probes, probe_to_command, reduce_probes, Urgency,
};
use anna_shared::progress::RequestStage;
use anna_shared::rpc::TranslatorTicket;
use anna_shared::specialist_learning::SpecialistLearningStore;
use anna_shared::specialist_patterns::match_generic_pattern;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

use crate::config::LlmConfig;
use crate::progress_tracker::ProgressTracker;
use crate::recipe_fast_path::{self, RecipeFastPathResult};
use crate::router::{self, DeterministicRoute};
use crate::state::SharedState;
use crate::translator::{self, TranslatorInput};
use crate::triage::{self, TriageResult};

/// Result of query routing
pub struct RoutingResult {
    pub ticket: TranslatorTicket,
    pub triage_result: Option<TriageResult>,
    pub translator_timed_out: bool,
    pub recipe_result: Option<RecipeFastPathResult>,
}

/// Route the query through LLM translator (primary) with recipe/learning support
/// v0.0.318: Added debug_mode for LLM call visibility
/// v0.0.391: REFACTORED - LLM translator is now PRIMARY, not pattern matching
pub async fn route_query(
    state: &SharedState,
    query: &str,
    _det_route: &DeterministicRoute, // v0.0.391: Ignored - translator is primary now
    llm_config: &LlmConfig,
    translator_model: &str,
    hw_cores: u32,
    hw_ram_gb: f64,
    has_gpu: bool,
    debug_mode: bool,
    progress: &mut ProgressTracker,
) -> RoutingResult {
    // v0.0.391: ALWAYS use LLM translator - no more pattern matching override
    // Recipe check first for fast-path on known queries
    // v0.0.400: FIX DEADLOCK - release read lock before calling triage_path
    let recipe_result = {
        let recipe_index = &state.read().await.recipe_index;
        recipe_fast_path::check_recipe_fast_path(query, recipe_index)
    }; // Read lock released here

    // If recipe can answer directly, return with recipe result
    if recipe_fast_path::can_answer_directly(&recipe_result) {
        let recipe = recipe_result.recipe.as_ref().unwrap();
        info!(
            "Recipe direct answer: id={}, score={}",
            recipe.id, recipe_result.score
        );
        // Return early with recipe for caller to handle
        return RoutingResult {
            ticket: recipe_result
                .ticket
                .clone()
                .unwrap_or_else(|| router::apply_deterministic_routing(query, None)),
            triage_result: None,
            translator_timed_out: false,
            recipe_result: Some(recipe_result),
        };
    }

    if recipe_result.skip_llm {
        // Recipe matched but no direct answer - skip LLM, continue with probes
        info!(
            "Recipe fast path hit: score={}, tokens={:?}",
            recipe_result.score, recipe_result.matched_tokens
        );
        let ticket = recipe_result
            .ticket
            .clone()
            .unwrap_or_else(|| router::apply_deterministic_routing(query, None));
        return RoutingResult {
            ticket,
            triage_result: None,
            translator_timed_out: false,
            recipe_result: None,
        };
    }

    // v0.0.392: REMOVED semantic similarity - it was redundant with translator
    // The translator IS the brain that understands user intent
    // Semantic similarity was another LLM call doing the same job worse

    // No exact recipe match - let the TRANSLATOR understand the user
    let (ticket, triage, timeout) = triage_path(
        state,
        query,
        llm_config,
        translator_model,
        hw_cores,
        hw_ram_gb,
        has_gpu,
        debug_mode,
        progress,
    )
    .await;
    RoutingResult {
        ticket,
        triage_result: triage,
        translator_timed_out: timeout,
        recipe_result: None,
    }
}

/// Enforce probe spine constraints on ticket probes
pub fn enforce_probe_spine(
    ticket: &mut TranslatorTicket,
    query: &str,
    det_route: &DeterministicRoute,
) {
    let route_class = det_route.class.to_string();
    let skip_spine_override = route_class == "configure_editor" && !ticket.needs_probes.is_empty();

    let spine_decision = enforce_minimum_probes(query, &ticket.needs_probes);
    if spine_decision.enforced && !skip_spine_override {
        info!(
            "Probe spine enforced from user text: {}",
            spine_decision.reason
        );
        // Apply minimal probe policy - max 3 default, 4 for system health
        let urgency = Urgency::Normal;
        let reduced = reduce_probes(spine_decision.probes.clone(), &route_class, urgency);
        if reduced.len() < spine_decision.probes.len() {
            info!(
                "Reduced probes from {} to {} for route {}",
                spine_decision.probes.len(),
                reduced.len(),
                route_class
            );
        }
        // Convert ProbeId to command strings
        ticket.needs_probes = reduced.iter().map(probe_to_command).collect();
    } else if skip_spine_override {
        info!(
            "v0.0.68: ConfigureEditor using router probes: {:?}",
            ticket.needs_probes
        );
    } else {
        // FALLBACK: Try route-capability based enforcement
        let (enforced_probes, spine_reason) =
            enforce_spine_probes(&ticket.needs_probes, &det_route.capability);
        if let Some(ref reason) = spine_reason {
            info!("Probe spine enforced from route: {}", reason);
            ticket.needs_probes = enforced_probes;
        }
        // Apply probe cap for non-spine-enforced probes too
        let max_probes = if route_class.contains("health") {
            4
        } else if route_class == "configure_editor" {
            10 // Need all editor probes for grounded selection
        } else {
            3
        };
        if ticket.needs_probes.len() > max_probes {
            info!(
                "Capping probes from {} to {}",
                ticket.needs_probes.len(),
                max_probes
            );
            ticket.needs_probes.truncate(max_probes);
        }
    }
}

/// Triage path for unknown queries - v0.0.402 REWRITE
/// Now tries deterministic fallback FIRST, only uses LLM for low-confidence cases.
/// This makes common queries fast and reliable while keeping LLM for ambiguous cases.
async fn triage_path(
    state: &SharedState,
    query: &str,
    config: &LlmConfig,
    translator_model: &str,
    hw_cores: u32,
    hw_ram_gb: f64,
    has_gpu: bool,
    debug_mode: bool,
    progress: &mut ProgressTracker,
) -> (TranslatorTicket, Option<TriageResult>, bool) {
    let stage_start = Instant::now();

    // v0.0.402: TRY DETERMINISTIC FALLBACK FIRST
    // This is fast and accurate for common queries with clear keywords
    let fallback_ticket = triage::create_fallback_ticket(query);

    // If fallback has high confidence (good keyword match), skip LLM entirely
    // This makes common queries like "disk space", "memory usage", "webcam" instant
    if fallback_ticket.confidence >= 0.75 && !fallback_ticket.needs_probes.is_empty() {
        info!(
            "v0.0.402: Using deterministic route (confidence={:.2}, probes={})",
            fallback_ticket.confidence,
            fallback_ticket.needs_probes.len()
        );
        progress.start_stage(RequestStage::Translator, 0);
        progress.complete_stage(RequestStage::Translator);

        // Record minimal latency
        {
            state
                .write()
                .await
                .latency
                .translator
                .add(stage_start.elapsed().as_millis() as u64);
        }

        let triage_result = triage::apply_triage_rules(fallback_ticket.clone());
        let mut enriched_ticket = triage_result.ticket.clone();
        enrich_with_learned_probes(&mut enriched_ticket, query);

        return (enriched_ticket, Some(triage_result), false);
    }

    // Fallback has low confidence - use LLM translator
    info!(
        "v0.0.402: Fallback confidence low ({:.2}), using LLM translator",
        fallback_ticket.confidence
    );
    progress.start_stage(RequestStage::Translator, config.translator_timeout_secs);
    let translator_input = TranslatorInput::new(query, hw_cores, hw_ram_gb, has_gpu);

    let (llm_ticket, translator_timed_out) = match timeout(
        Duration::from_secs(config.translator_timeout_secs),
        translator::translate_with_debug(
            translator_model,
            &translator_input,
            config.translator_timeout_secs,
        ),
    )
    .await
    {
        Ok(Ok(result)) => {
            progress.complete_stage(RequestStage::Translator);
            // v0.0.318: Record translator LLM call in debug mode
            if debug_mode {
                progress.add_llm_call(
                    "translator",
                    translator_model,
                    &result.prompt,
                    &result.response,
                    result.duration_ms,
                    None,
                );
            }
            (Some(result.ticket), false)
        }
        Ok(Err(e)) => {
            warn!("Translator error: {}", e);
            progress.error_stage(RequestStage::Translator, &e);
            (None, false)
        }
        Err(_) => {
            warn!("Translator timeout");
            progress.timeout_stage(RequestStage::Translator);
            (None, true)
        }
    };

    // Record translator latency
    {
        state
            .write()
            .await
            .latency
            .translator
            .add(stage_start.elapsed().as_millis() as u64);
    }

    // If LLM failed, use the fallback ticket we already computed
    let ticket = llm_ticket.unwrap_or(fallback_ticket);

    // Apply triage rules
    let triage_result = triage::apply_triage_rules(ticket.clone());

    // v0.0.401: Enrich ticket with learned probes from specialist lessons
    let mut enriched_ticket = triage_result.ticket.clone();
    enrich_with_learned_probes(&mut enriched_ticket, query);

    (enriched_ticket, Some(triage_result), translator_timed_out)
}

/// v0.0.401: Enrich ticket with probes from specialist lessons
/// Checks if we have learned patterns that match this query, and adds their effective probes
fn enrich_with_learned_probes(ticket: &mut TranslatorTicket, query: &str) {
    let store = SpecialistLearningStore::load();

    // Check for generic pattern match first (e.g., "check X config" pattern)
    if let Some((pattern, target)) = match_generic_pattern(&store, query) {
        let learned_probes: Vec<String> = pattern
            .probe_templates
            .iter()
            .map(|t| t.replace("{target}", &target))
            .collect();

        if !learned_probes.is_empty() {
            // Add learned probes that aren't already in the ticket
            let mut added = 0;
            for probe in learned_probes {
                if !ticket.needs_probes.contains(&probe) {
                    ticket.needs_probes.push(probe);
                    added += 1;
                }
            }
            if added > 0 {
                info!(
                    "v0.0.401: Added {} learned probes from pattern for '{}'",
                    added, target
                );
            }
            return;
        }
    }

    // Fall back to keyword-based lesson matching
    let lessons = store.find_lessons(query);
    if !lessons.is_empty() {
        let best = &lessons[0];
        if best.confidence >= 70 && best.success_count >= 2 {
            let mut added = 0;
            for probe in &best.effective_probes {
                if !ticket.needs_probes.contains(probe) {
                    ticket.needs_probes.push(probe.clone());
                    added += 1;
                }
            }
            if added > 0 {
                info!("v0.0.401: Added {} learned probes from lesson", added);
            }
        }
    }
}
