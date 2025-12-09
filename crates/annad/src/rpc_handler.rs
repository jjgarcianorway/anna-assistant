//! RPC request handlers with deterministic routing, triage, and fallback.
//!
//! v0.0.166: Integrated stage modules for modularization.

use anna_shared::probe_spine::{
    enforce_minimum_probes, enforce_spine_probes, probe_to_command, reduce_probes, Urgency,
};
use anna_shared::progress::RequestStage;
use anna_shared::rpc::{RequestParams, RpcMethod, RpcRequest, RpcResponse};
use anna_shared::status::LlmState;
use anna_shared::trace::SpecialistOutcome;
use anna_shared::transcript::TranscriptEvent;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

use crate::comms::{team_from_domain, CommsGenerator};
use crate::config::LlmConfig;
use crate::configure_editor::{handle_configure_editor, ConfigureEditorResult};
use crate::deterministic;
use crate::fast_path_handler::{build_fast_path_result, try_fast_path_answer};
use crate::handlers;
use crate::probe_runner;
use crate::progress_tracker::ProgressTracker;
use crate::recipe_fast_path;
use crate::result_stage::{build_final_result, wrap_with_theatre};
use crate::router;
use crate::service_desk;
use crate::specialist_handler::{try_specialist_llm, SpecialistResult};
use crate::state::SharedState;
use crate::theatre::TheatreContext;
use crate::timeout_handler::make_timeout_response;
use crate::translator::{self, TranslatorInput};
use crate::triage::{self, TriageResult};

/// Handle an RPC request
pub async fn handle_request(state: SharedState, request: RpcRequest) -> RpcResponse {
    let id = request.id.clone();

    match request.method {
        RpcMethod::Status => handlers::handle_status(state, id).await,
        RpcMethod::Request => handle_llm_request(state, id, request.params).await,
        RpcMethod::Reset => handlers::handle_reset(state, id).await,
        RpcMethod::Uninstall => handlers::handle_uninstall(state, id).await,
        RpcMethod::Autofix => handlers::handle_autofix(state, id).await,
        RpcMethod::Probe => handlers::handle_probe(state, id, request.params).await,
        RpcMethod::Progress => handlers::handle_progress(state, id).await,
        RpcMethod::Stats => handlers::handle_stats(state, id).await,
        RpcMethod::StatusSnapshot => handlers::handle_status_snapshot(state, id).await,
        RpcMethod::GetDaemonInfo => handlers::handle_get_daemon_info(state, id).await,
        RpcMethod::PlanChange => handlers::handle_plan_change(id, request.params).await,
        RpcMethod::ApplyChange => handlers::handle_apply_change(id, request.params).await,
        RpcMethod::RollbackChange => handlers::handle_rollback_change(id, request.params).await,
    }
}

/// Service desk pipeline with deterministic routing, triage, and fallback
async fn handle_llm_request(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    let request_id = uuid::Uuid::new_v4().to_string();
    let request_timeout = { state.read().await.config.daemon.request_timeout_secs };

    // Extract query for timeout fallback (v0.0.40)
    let query_for_fallback = params
        .as_ref()
        .and_then(|p| p.get("prompt"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match timeout(
        Duration::from_secs(request_timeout),
        handle_llm_request_inner(state.clone(), id.clone(), params, request_id.clone()),
    )
    .await
    {
        Ok(response) => response,
        Err(_) => {
            warn!("Global request timeout ({}s)", request_timeout);
            make_timeout_response(
                id,
                request_id,
                request_timeout,
                query_for_fallback.as_deref(),
            )
        }
    }
}

/// Inner request handler (wrapped by global timeout)
async fn handle_llm_request_inner(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
    request_id: String,
) -> RpcResponse {
    let request_start = Instant::now();
    let mut progress = ProgressTracker::new();

    // Get config, models, and hardware from state
    let (llm_config, translator_model, specialist_model, hw_cores, hw_ram_gb, has_gpu, debug_mode) = {
        let state = state.read().await;
        if state.llm.state != LlmState::Ready {
            return RpcResponse::error(id, -32002, format!("LLM not ready: {}", state.llm.state));
        }
        (
            state.config.llm.clone(),
            state.config.llm.translator_model.clone(),
            state.config.llm.specialist_model.clone(),
            state.hardware.cpu_cores,
            state.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            state.hardware.gpu.is_some(),
            state.config.debug_mode(),
        )
    };

    // Parse parameters
    let params: RequestParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => return RpcResponse::error(id, -32602, format!("Invalid params: {}", e)),
        },
        None => return RpcResponse::error(id, -32602, "Missing params".to_string()),
    };

    let query = &params.prompt;
    progress.add_user_message(query);
    {
        state.write().await.progress_events.clear();
    }

    // Step 0: Fast path check (v0.0.39) - answer health/status queries without LLM
    let fast_path_config = {
        let state = state.read().await;
        (
            state.config.daemon.fast_path_enabled,
            state.config.daemon.snapshot_max_age_secs,
        )
    };

    if fast_path_config.0 {
        if let Some(result) = try_fast_path_answer(query, fast_path_config.1) {
            info!(
                "Fast path handled: class={}, reliability={}",
                result.class, result.reliability
            );

            // Add fast path event to transcript
            let elapsed = progress.elapsed_ms();
            progress.transcript_mut().push(TranscriptEvent::fast_path(
                elapsed,
                true,
                result.class.to_string(),
                &result.trace_note,
                false, // No probes needed if we had fresh snapshot
            ));

            // Build result and return immediately
            let fast_result = build_fast_path_result(
                request_id,
                result.answer,
                result.class,
                result.reliability,
                progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(fast_result).unwrap());
        }
    }

    // Step 1: Deterministic routing (always runs)
    let det_route = router::get_route(query);
    info!(
        "Router: class={:?}, domain={}, probes={:?}",
        det_route.class, det_route.domain, det_route.probes
    );

    // Step 2: Route based on query class (v0.0.101: check recipes first for Unknown)
    // v0.0.102: Also check for ConfigureShell/ConfigureGit which are recipe-first
    let should_check_recipes = det_route.class == router::QueryClass::Unknown
        || det_route.class == router::QueryClass::ConfigureShell
        || det_route.class == router::QueryClass::ConfigureGit;

    let (mut ticket, triage_result, translator_timed_out) = if should_check_recipes {
        // v0.0.101: Check recipe index BEFORE calling LLM translator
        let recipe_index = &state.read().await.recipe_index;
        let recipe_result = recipe_fast_path::check_recipe_fast_path(query, recipe_index);

        // v0.0.102: If recipe can answer directly, return immediately!
        if recipe_fast_path::can_answer_directly(&recipe_result) {
            let recipe = recipe_result.recipe.as_ref().unwrap();
            info!(
                "Recipe direct answer: id={}, score={}",
                recipe.id, recipe_result.score
            );

            progress.add_translator_message(&format!(
                "Recipe match: {} (score {})",
                recipe.id, recipe_result.score
            ));

            let result = recipe_fast_path::build_recipe_result(
                request_id,
                recipe,
                &recipe_result.matched_tokens,
                progress.take_transcript(),
            );
            return wrap_with_theatre(id, result, None);
        }

        if recipe_result.skip_llm {
            // Recipe matched but no direct answer - skip LLM, continue with probes
            info!(
                "Recipe fast path hit: score={}, tokens={:?}",
                recipe_result.score, recipe_result.matched_tokens
            );
            let ticket = recipe_result
                .ticket
                .unwrap_or_else(|| router::apply_deterministic_routing(query, None));
            (ticket, None, false)
        } else {
            // No recipe match or low confidence - fall back to LLM translator
            let (ticket, triage, timeout) = triage_path(
                &state,
                query,
                &llm_config,
                &translator_model,
                hw_cores,
                hw_ram_gb,
                has_gpu,
                &mut progress,
            )
            .await;
            (ticket, triage, timeout)
        }
    } else {
        // Known class -> deterministic ticket
        let ticket = router::apply_deterministic_routing(query, None);
        (ticket, None, false)
    };

    // Step 2.5: Enforce probe spine (v0.45.2 - user text based)
    // v0.0.68: ConfigureEditor already has correct probes from router - skip spine override
    // FIRST: Use keyword matching on user text to force probes (last line of defense)
    let route_class = det_route.class.to_string();
    let skip_spine_override = route_class == "configure_editor" && !ticket.needs_probes.is_empty();

    let spine_decision = enforce_minimum_probes(query, &ticket.needs_probes);
    if spine_decision.enforced && !skip_spine_override {
        info!(
            "Probe spine enforced from user text: {}",
            spine_decision.reason
        );
        // Apply minimal probe policy (v0.45.3) - max 3 default, 4 for system health
        let urgency = Urgency::Normal; // TODO: detect from query (e.g., "quick" -> Quick)
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
        ticket.needs_probes = reduced.iter().map(|p| probe_to_command(p)).collect();
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
        // Apply probe cap for non-spine-enforced probes too (v0.45.3)
        // v0.0.60: ConfigureEditor needs 10 probes for all editors
        let route_class = det_route.class.to_string();
        let max_probes = if route_class.contains("health") {
            4
        } else if route_class == "configure_editor" {
            10 // v0.0.60: Need all editor probes for grounded selection
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

    let classified_domain = ticket.domain;
    let ticket_probes_planned = ticket.needs_probes.len();
    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}, confidence={:.2}",
        ticket.domain, ticket.intent, ticket.needs_probes, ticket.confidence
    ));

    // v0.0.148: Create comms generator for fly-on-wall experience
    let team = team_from_domain(&classified_domain.to_string());
    let mut comms = CommsGenerator::new(team, &request_id);

    // v0.0.148: Anna dispatches to team and junior acknowledges
    comms.dispatch(&mut progress);
    comms.junior_ack(&mut progress);
    save_progress(&state, &progress).await;

    // Step 3: Check if immediate clarification needed (from triage)
    if let Some(ref triage) = triage_result {
        if triage.needs_immediate_clarification {
            save_progress(&state, &progress).await;
            let question = triage
                .clarification_question
                .clone()
                .unwrap_or_else(|| triage::generate_heuristic_clarification(query));
            let result = service_desk::create_clarification_response(
                request_id,
                ticket,
                &question,
                progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 4: Run probes with timeout
    progress.start_stage(RequestStage::Probes, llm_config.probes_total_timeout_secs);

    // v0.0.148: Junior reports probe progress
    if !ticket.needs_probes.is_empty() {
        comms.junior_probing(&mut progress, ticket.needs_probes.len());
        save_progress(&state, &progress).await;
    }

    let probe_cap_warning = triage_result
        .as_ref()
        .map(|t| t.probe_cap_applied)
        .unwrap_or(false);
    let probes_start = Instant::now();

    let probe_results = match timeout(
        Duration::from_secs(llm_config.probes_total_timeout_secs),
        probe_runner::run_probes(&state, &ticket, &llm_config, &mut progress),
    )
    .await
    {
        Ok(results) => {
            progress.complete_stage(RequestStage::Probes);
            // Record probes latency
            {
                state
                    .write()
                    .await
                    .latency
                    .probes
                    .add(probes_start.elapsed().as_millis() as u64);
            }
            // v0.0.152: Report probe completion
            let success_count = results.iter().filter(|p| p.exit_code == 0).count();
            comms.junior_probes_done(&mut progress, success_count);
            save_progress(&state, &progress).await;
            results
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Probes);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                request_id,
                "probes",
                Some(ticket),
                vec![],
                progress.take_transcript(),
                classified_domain,
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    // Step 5: v0.45.7 Evidence enforcement - "no evidence, no claims" rule
    // NOTE: For tool/package checks, exit_code=1 is VALID negative evidence!
    // Count probes that produced valid evidence (including negative evidence)
    let valid_evidence_count = {
        use anna_shared::parsers::parse_probe_result;
        probe_results
            .iter()
            .filter(|p| parse_probe_result(p).is_valid_evidence())
            .count()
    };
    if det_route.capability.evidence_required && valid_evidence_count == 0 {
        info!("v0.45.7: No valid evidence collected but evidence required - returning deterministic failure");
        save_progress(&state, &progress).await;
        let required_evidence: Vec<String> = det_route
            .capability
            .required_evidence
            .iter()
            .map(|k| k.to_string())
            .collect();
        let result = service_desk::create_no_evidence_response(
            request_id,
            ticket,
            probe_results,
            progress.take_transcript(),
            classified_domain,
            &required_evidence,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 5.5: v0.0.149 ConfigureEditor - extracted to separate module
    if det_route.class == router::QueryClass::ConfigureEditor {
        let editor_result = handle_configure_editor(
            request_id.clone(),
            query,
            ticket.clone(),
            &probe_results,
            progress.transcript_clone(),
            classified_domain,
        );

        if let ConfigureEditorResult::Handled(result) = editor_result {
            save_progress(&state, &progress).await;
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 6: Build context with summarized probes
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    // v0.0.148: Junior reviewing the data
    comms.junior_reviewing(&mut progress);
    save_progress(&state, &progress).await;

    // Step 7: Try deterministic answer FIRST for known query classes
    let specialist_result = if det_route.can_answer_deterministically() {
        if let Some(det) = deterministic::try_answer(query, &context, &probe_results) {
            if det.parsed_data_count > 0 {
                info!(
                    "Deterministic answer produced ({} entries)",
                    det.parsed_data_count
                );
                // Skip specialist stage - deterministic router answered
                progress.skip_stage_deterministic(RequestStage::Specialist);
                let route_class = det.route_class.clone();
                SpecialistResult {
                    answer: det.answer.clone(),
                    used_deterministic: true,
                    det_result: Some(det),
                    prompt_truncated: false, // No prompt for deterministic path
                    outcome: SpecialistOutcome::Skipped,
                    fallback_route_class: Some(route_class),
                }
            } else {
                warn!("Deterministic parser produced empty result");
                try_specialist_llm(
                    &state,
                    query,
                    &context,
                    &probe_results,
                    &ticket,
                    &llm_config,
                    &specialist_model,
                    debug_mode,
                    &mut progress,
                )
                .await
            }
        } else {
            try_specialist_llm(
                &state,
                query,
                &context,
                &probe_results,
                &ticket,
                &llm_config,
                &specialist_model,
                debug_mode,
                &mut progress,
            )
            .await
        }
    } else {
        try_specialist_llm(
            &state,
            query,
            &context,
            &probe_results,
            &ticket,
            &llm_config,
            &specialist_model,
            debug_mode,
            &mut progress,
        )
        .await
    };

    // Step 8: Handle no answer case
    if specialist_result.answer.is_empty() {
        save_progress(&state, &progress).await;
        let result = service_desk::create_no_data_response(
            request_id,
            ticket,
            probe_results,
            progress.take_transcript(),
            classified_domain,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 9: Build final result with proper scoring
    progress.start_stage(RequestStage::Supervisor, llm_config.supervisor_timeout_secs);
    progress.add_final_answer(&specialist_result.answer);

    // v0.0.148: Junior confirms answer or escalates based on outcome
    match specialist_result.outcome {
        SpecialistOutcome::Ok | SpecialistOutcome::Skipped => {
            let approx_confidence = if specialist_result.used_deterministic { 85 } else { 75 };
            comms.junior_done(&mut progress, approx_confidence);
        }
        SpecialistOutcome::Timeout | SpecialistOutcome::Error => {
            comms.junior_escalate(&mut progress, "LLM had trouble, used fallback");
            comms.senior_response(&mut progress, specialist_result.used_deterministic);
        }
        SpecialistOutcome::BudgetExceeded => {
            comms.junior_escalate(&mut progress, "Query too complex");
            comms.senior_response(&mut progress, false);
        }
    }
    comms.anna_returning(&mut progress);
    save_progress(&state, &progress).await;

    // v0.0.166: Use result_stage module for final result building
    let result = build_final_result(
        request_id,
        query,
        ticket,
        probe_results.clone(),
        progress.transcript_clone(),
        classified_domain,
        translator_timed_out,
        &specialist_result,
        det_route.capability.evidence_required,
        ticket_probes_planned,
        probe_cap_warning,
    );

    progress.complete_stage(RequestStage::Supervisor);

    // Record total request latency
    let total_ms = request_start.elapsed().as_millis() as u64;
    {
        let mut state = state.write().await;
        state.latency.total.add(total_ms);
        // v0.0.79: Record stats
        let specialist_timeout = matches!(specialist_result.outcome, SpecialistOutcome::Timeout);
        state.record_request(
            specialist_result.used_deterministic,
            translator_timed_out,
            specialist_timeout,
        );
    }

    info!(
        "Request completed: domain={}, reliability={}, deterministic={}, trace={}, latency={}ms",
        result.domain,
        result.reliability_score,
        specialist_result.used_deterministic,
        result
            .execution_trace
            .as_ref()
            .map(|t| t.to_string())
            .unwrap_or_default(),
        total_ms
    );

    save_progress(&state, &progress).await;

    // v0.0.106: Create theatre context for Service Desk Theatre
    let mut theatre = TheatreContext::new(query, classified_domain);
    theatre.start_work();

    // v0.0.108: Escalate to senior if reliability is low
    if result.reliability_score < 60 && !result.needs_clarification {
        theatre.escalate();
    }

    theatre.resolve(result.answer.clone(), result.reliability_score, total_ms);

    // v0.0.107: Record topic to user profile for personalized greetings
    theatre.record_topic_to_profile();

    // v0.0.107: Record staff performance metrics
    theatre.record_staff_stats(result.reliability_score, total_ms);

    // v0.0.166: Return with theatre context using result_stage module
    wrap_with_theatre(id, result, Some(theatre))
}

/// Triage path for unknown queries - uses LLM translator with confidence threshold
async fn triage_path(
    state: &SharedState,
    query: &str,
    config: &LlmConfig,
    translator_model: &str,
    hw_cores: u32,
    hw_ram_gb: f64,
    has_gpu: bool,
    progress: &mut ProgressTracker,
) -> (
    anna_shared::rpc::TranslatorTicket,
    Option<TriageResult>,
    bool,
) {
    progress.start_stage(RequestStage::Translator, config.translator_timeout_secs);
    let translator_input = TranslatorInput::new(query, hw_cores, hw_ram_gb, has_gpu);
    let stage_start = Instant::now();

    let (llm_ticket, translator_timed_out) = match timeout(
        Duration::from_secs(config.translator_timeout_secs),
        translator::translate_with_context(
            translator_model,
            &translator_input,
            config.translator_timeout_secs,
        ),
    )
    .await
    {
        Ok(Ok(t)) => {
            progress.complete_stage(RequestStage::Translator);
            (Some(t), false)
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

    // If translator failed completely, use fallback
    let ticket = llm_ticket.unwrap_or_else(|| triage::create_fallback_ticket(query));

    // Apply triage rules
    let triage_result = triage::apply_triage_rules(ticket.clone());

    (
        triage_result.ticket.clone(),
        Some(triage_result),
        translator_timed_out,
    )
}

/// Save progress events to state for polling
async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    state.write().await.progress_events = progress.events().to_vec();
}
