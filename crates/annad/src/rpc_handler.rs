//! RPC request handlers with deterministic routing, triage, and fallback.

use anna_shared::probe_spine::{
    enforce_minimum_probes, enforce_spine_probes, probe_to_command, reduce_probes, Urgency,
};
use anna_shared::progress::RequestStage;
use anna_shared::recipe_learning::try_learn_from_result;
use anna_shared::rpc::{RequestParams, RpcMethod, RpcRequest, RpcResponse, ServiceDeskResult};
use anna_shared::status::LlmState;
use anna_shared::trace::{
    evidence_kinds_from_probes, ExecutionTrace, ProbeStats, SpecialistOutcome,
};
use anna_shared::transcript::TranscriptEvent;
use std::time::Instant;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

use crate::config::LlmConfig;
use crate::deterministic;
use crate::fast_path_handler::{build_fast_path_result, force_fast_path_fallback, is_health_query, try_fast_path_answer};
use crate::handlers;
use crate::probe_runner;
use crate::progress_tracker::ProgressTracker;
use crate::router;
use crate::service_desk;
use crate::specialist_handler::{try_specialist_llm, SpecialistResult};
use crate::state::SharedState;
use crate::triage::{self, TriageResult};
use crate::translator::{self, TranslatorInput};

/// Wrap result in response and try to learn from it (v0.0.94)
fn success_with_learning(id: String, result: ServiceDeskResult) -> RpcResponse {
    // Try to learn recipe from successful result
    let learn_result = try_learn_from_result(&result);
    if learn_result.learned {
        if let Some(recipe_id) = &learn_result.recipe_id {
            debug!("Learned recipe {} from result", recipe_id);
        }
    }
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
}

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

    match timeout(Duration::from_secs(request_timeout), handle_llm_request_inner(state.clone(), id.clone(), params, request_id.clone())).await {
        Ok(response) => response,
        Err(_) => {
            warn!("Global request timeout ({}s)", request_timeout);
            make_timeout_response(id, request_id, request_timeout, query_for_fallback.as_deref())
        }
    }
}

fn make_timeout_response(id: String, request_id: String, timeout_secs: u64, query: Option<&str>) -> RpcResponse {
    // v0.0.40: For health queries, use fast path fallback instead of timeout message
    if let Some(q) = query {
        if is_health_query(q) {
            if let Some(fallback) = force_fast_path_fallback(q) {
                info!("Using fast path fallback for health query on timeout");
                let result = build_fast_path_result(
                    request_id,
                    fallback.answer,
                    fallback.class,
                    fallback.reliability,
                    anna_shared::transcript::Transcript::default(),
                );
                return RpcResponse::success(id, serde_json::to_value(result).unwrap());
            }
        }
    }

    // v0.0.34: Never ask to rephrase - provide a deterministic status answer
    let answer = format!(
        "**Request Timeout**\n\n\
         The request exceeded the {}s budget. This typically means:\n\n\
         - The LLM backend is under load or unavailable\n\
         - The query requires complex analysis\n\n\
         **What you can do:**\n\
         - Check `annactl status` to verify LLM availability\n\
         - Try a more specific question (e.g., \"any errors?\" instead of broad queries)\n\n\
         *Evidence: global timeout, no probes completed*",
        timeout_secs
    );

    let result = anna_shared::rpc::ServiceDeskResult {
        request_id, answer, reliability_score: 20, // Low but not zero - we provided info
        reliability_signals: anna_shared::rpc::ReliabilitySignals::default(),
        reliability_explanation: None,
        domain: anna_shared::rpc::SpecialistDomain::System,
        evidence: anna_shared::rpc::EvidenceBlock::default(),
        needs_clarification: false, // Never ask to rephrase
        clarification_question: None,
        clarification_request: None,
        transcript: anna_shared::transcript::Transcript::default(),
        execution_trace: Some(anna_shared::trace::ExecutionTrace::global_timeout(timeout_secs)),
    };
    RpcResponse::success(id, serde_json::to_value(result).unwrap())
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
    { state.write().await.progress_events.clear(); }

    // Step 0: Fast path check (v0.0.39) - answer health/status queries without LLM
    let fast_path_config = {
        let state = state.read().await;
        (state.config.daemon.fast_path_enabled, state.config.daemon.snapshot_max_age_secs)
    };

    if fast_path_config.0 {
        if let Some(result) = try_fast_path_answer(query, fast_path_config.1) {
            info!("Fast path handled: class={}, reliability={}", result.class, result.reliability);

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
            let fast_result = build_fast_path_result(request_id, result.answer, result.class, result.reliability, progress.take_transcript());
            return RpcResponse::success(id, serde_json::to_value(fast_result).unwrap());
        }
    }

    // Step 1: Deterministic routing (always runs)
    let det_route = router::get_route(query);
    info!("Router: class={:?}, domain={}, probes={:?}", det_route.class, det_route.domain, det_route.probes);

    // Step 2: Route based on query class
    let (mut ticket, triage_result, translator_timed_out) = if det_route.class == router::QueryClass::Unknown {
        // Unknown class -> use triage path with LLM translator
        triage_path(&state, query, &llm_config, &translator_model, hw_cores, hw_ram_gb, has_gpu, &mut progress).await
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
        info!("Probe spine enforced from user text: {}", spine_decision.reason);
        // Apply minimal probe policy (v0.45.3) - max 3 default, 4 for system health
        let urgency = Urgency::Normal; // TODO: detect from query (e.g., "quick" -> Quick)
        let reduced = reduce_probes(spine_decision.probes.clone(), &route_class, urgency);
        if reduced.len() < spine_decision.probes.len() {
            info!("Reduced probes from {} to {} for route {}", spine_decision.probes.len(), reduced.len(), route_class);
        }
        // Convert ProbeId to command strings
        ticket.needs_probes = reduced.iter()
            .map(|p| probe_to_command(p))
            .collect();
    } else if skip_spine_override {
        info!("v0.0.68: ConfigureEditor using router probes: {:?}", ticket.needs_probes);
    } else {
        // FALLBACK: Try route-capability based enforcement
        let (enforced_probes, spine_reason) = enforce_spine_probes(&ticket.needs_probes, &det_route.capability);
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
            10  // v0.0.60: Need all editor probes for grounded selection
        } else {
            3
        };
        if ticket.needs_probes.len() > max_probes {
            info!("Capping probes from {} to {}", ticket.needs_probes.len(), max_probes);
            ticket.needs_probes.truncate(max_probes);
        }
    }

    let classified_domain = ticket.domain;
    let ticket_probes_planned = ticket.needs_probes.len();
    progress.add_translator_message(&format!(
        "domain={}, intent={}, probes={:?}, confidence={:.2}",
        ticket.domain, ticket.intent, ticket.needs_probes, ticket.confidence
    ));

    // Step 3: Check if immediate clarification needed (from triage)
    if let Some(ref triage) = triage_result {
        if triage.needs_immediate_clarification {
            save_progress(&state, &progress).await;
            let question = triage.clarification_question.clone().unwrap_or_else(|| {
                triage::generate_heuristic_clarification(query)
            });
            let result = service_desk::create_clarification_response(
                request_id, ticket, &question, progress.take_transcript(),
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 4: Run probes with timeout
    progress.start_stage(RequestStage::Probes, llm_config.probes_total_timeout_secs);
    let probe_cap_warning = triage_result.as_ref().map(|t| t.probe_cap_applied).unwrap_or(false);
    let probes_start = Instant::now();

    let probe_results = match timeout(
        Duration::from_secs(llm_config.probes_total_timeout_secs),
        probe_runner::run_probes(&state, &ticket, &llm_config, &mut progress),
    ).await {
        Ok(results) => {
            progress.complete_stage(RequestStage::Probes);
            // Record probes latency
            { state.write().await.latency.probes.add(probes_start.elapsed().as_millis() as u64); }
            results
        }
        Err(_) => {
            progress.timeout_stage(RequestStage::Probes);
            save_progress(&state, &progress).await;
            let result = service_desk::create_timeout_response(
                request_id, "probes", Some(ticket), vec![], progress.take_transcript(), classified_domain,
            );
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    };

    // Step 5: v0.45.7 Evidence enforcement - "no evidence, no claims" rule
    // NOTE: For tool/package checks, exit_code=1 is VALID negative evidence!
    // Count probes that produced valid evidence (including negative evidence)
    let valid_evidence_count = {
        use anna_shared::parsers::parse_probe_result;
        probe_results.iter()
            .filter(|p| parse_probe_result(p).is_valid_evidence())
            .count()
    };
    if det_route.capability.evidence_required && valid_evidence_count == 0 {
        info!("v0.45.7: No valid evidence collected but evidence required - returning deterministic failure");
        save_progress(&state, &progress).await;
        let required_evidence: Vec<String> = det_route.capability.required_evidence
            .iter()
            .map(|k| k.to_string())
            .collect();
        let result = service_desk::create_no_evidence_response(
            request_id, ticket, probe_results, progress.take_transcript(), classified_domain, &required_evidence,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 5.5: v0.0.59 ConfigureEditor - use ONLY current probe evidence, no inventory
    // v0.0.62: Fixed probe accounting and execution trace for grounded output
    if det_route.class == router::QueryClass::ConfigureEditor {
        use anna_shared::parsers::{parse_probe_result, get_installed_tools, installed_editors_from_parsed};
        use anna_shared::trace::{ProbeStats, ExecutionTrace, EvidenceKind};

        // v0.0.59: Parse probe_results to get installed editors from ToolExists evidence ONLY
        let parsed: Vec<_> = probe_results.iter().map(|p| parse_probe_result(p)).collect();

        // v0.0.59: Use dedicated helper for consistent editor extraction
        let installed_editors = installed_editors_from_parsed(&parsed);

        // Track what we checked (for no-editors-found message)
        let tools = get_installed_tools(&parsed);
        let checked_editors: Vec<String> = tools.iter()
            .map(|t| t.name.clone())
            .collect();

        // v0.0.62: Count valid evidence for proper grounding
        let valid_evidence_count = parsed.iter()
            .filter(|p| p.is_valid_evidence())
            .count();

        info!("v0.0.62: ConfigureEditor - checked {:?}, found installed: {:?}, valid_evidence={}",
            checked_editors, installed_editors, valid_evidence_count);

        // v0.0.62: Build execution trace for ConfigureEditor paths
        let probe_stats = ProbeStats::from_results(ticket.needs_probes.len(), &probe_results);
        let evidence_kinds = vec![EvidenceKind::ToolExists];

        if installed_editors.is_empty() {
            // v0.0.59: No editors found - grounded negative evidence (we checked, found none)
            let checked_list = if checked_editors.is_empty() {
                "vim, nano, emacs, code".to_string()
            } else {
                checked_editors.join(", ")
            };
            let answer = format!(
                "No supported text editors were detected.\n\n\
                Checked: {}\n\n\
                Install vim, nano, or another editor and retry.",
                checked_list
            );
            save_progress(&state, &progress).await;
            // v0.0.62: Use valid_evidence_count for proper grounding
            let mut result = service_desk::build_result_with_flags(
                request_id, answer, query, ticket, probe_results.clone(), progress.transcript_clone(),
                classified_domain, false, true, valid_evidence_count, false,
                service_desk::FallbackContext { used_deterministic_fallback: false, fallback_route_class: "configure_editor".to_string(), evidence_kinds: vec!["tool_exists".to_string()], specialist_outcome: Some(SpecialistOutcome::Skipped), fallback_used: Some(anna_shared::trace::FallbackUsed::None), evidence_required: Some(true) },
            );
            // v0.0.62: Set execution trace
            result.execution_trace = Some(ExecutionTrace::deterministic_route(
                "configure_editor",
                probe_stats,
                evidence_kinds,
            ));
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        } else if installed_editors.len() == 1 {
            // v0.0.59: Exactly one editor - deterministic answer, NO questions
            let editor = &installed_editors[0];
            let answer = build_editor_config_answer(editor);
            save_progress(&state, &progress).await;
            // v0.0.62: Use valid_evidence_count for proper grounding
            let mut result = service_desk::build_result_with_flags(
                request_id, answer, query, ticket, probe_results.clone(), progress.transcript_clone(),
                classified_domain, false, true, valid_evidence_count, false,
                service_desk::FallbackContext { used_deterministic_fallback: false, fallback_route_class: "configure_editor".to_string(), evidence_kinds: vec!["tool_exists".to_string()], specialist_outcome: Some(SpecialistOutcome::Skipped), fallback_used: Some(anna_shared::trace::FallbackUsed::None), evidence_required: Some(true) },
            );
            // v0.0.62: Set execution trace
            result.execution_trace = Some(ExecutionTrace::deterministic_route(
                "configure_editor",
                probe_stats,
                evidence_kinds,
            ));
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        } else {
            // v0.0.66: Multiple editors - statement with numbered options, no question mark
            // Format: "I can configure syntax highlighting for one of these editors:\n1) vim\n2) code\nReply with the number."
            let editors_list: Vec<String> = installed_editors.iter()
                .enumerate()
                .map(|(i, e)| format!("{}) {}", i + 1, e))
                .collect();
            let answer = format!(
                "I can configure syntax highlighting for one of these editors:\n{}\nReply with the number.",
                editors_list.join("\n")
            );

            let options: Vec<(String, String)> = installed_editors.iter()
                .map(|e| (e.clone(), e.clone()))
                .collect();
            save_progress(&state, &progress).await;

            // v0.0.66: Build result with clarification but answer text is a statement
            let mut result = service_desk::create_clarification_with_options(
                request_id,
                ticket.clone(),
                &answer,  // Use the full statement as the "question"
                options,
                probe_results.clone(),
                progress.take_transcript(),
            );
            // v0.0.66: Override answer with clean statement (no question)
            result.answer = answer;
            return RpcResponse::success(id, serde_json::to_value(result).unwrap());
        }
    }

    // Step 6: Build context with summarized probes
    let context = {
        let state = state.read().await;
        service_desk::build_context(&state.hardware, &probe_results)
    };

    // Step 7: Try deterministic answer FIRST for known query classes
    let specialist_result = if det_route.can_answer_deterministically() {
        if let Some(det) = deterministic::try_answer(query, &context, &probe_results) {
            if det.parsed_data_count > 0 {
                info!("Deterministic answer produced ({} entries)", det.parsed_data_count);
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
                try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, debug_mode, &mut progress).await
            }
        } else {
            try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, debug_mode, &mut progress).await
        }
    } else {
        try_specialist_llm(&state, query, &context, &probe_results, &ticket, &llm_config, &specialist_model, debug_mode, &mut progress).await
    };
    let SpecialistResult { answer, used_deterministic, det_result, prompt_truncated, outcome, fallback_route_class } = specialist_result;

    // Step 8: Handle no answer case
    if answer.is_empty() {
        save_progress(&state, &progress).await;
        let result = service_desk::create_no_data_response(
            request_id, ticket, probe_results, progress.take_transcript(), classified_domain,
        );
        return RpcResponse::success(id, serde_json::to_value(result).unwrap());
    }

    // Step 9: Build final result with proper scoring
    progress.start_stage(RequestStage::Supervisor, llm_config.supervisor_timeout_secs);
    progress.add_final_answer(&answer);

    let parsed_data_count = det_result.as_ref().map(|d| d.parsed_data_count).unwrap_or(0);

    // Build fallback context for TRUST+ explanations and v0.0.24 trace-based scoring
    let used_deterministic_fallback = matches!(outcome, SpecialistOutcome::Timeout | SpecialistOutcome::Error)
        && used_deterministic && parsed_data_count > 0;
    let fallback_used = if used_deterministic_fallback {
        Some(anna_shared::trace::FallbackUsed::Deterministic {
            route_class: fallback_route_class.clone().unwrap_or_default(),
        })
    } else {
        Some(anna_shared::trace::FallbackUsed::None)
    };

    // v0.45.2: Derive evidence kinds from ACTUAL probes, not route class
    let actual_evidence_kinds = evidence_kinds_from_probes(&probe_results);

    let fallback_ctx = service_desk::FallbackContext {
        used_deterministic_fallback,
        fallback_route_class: fallback_route_class.clone().unwrap_or_default(),
        evidence_kinds: actual_evidence_kinds.iter().map(|k| k.to_string()).collect(),
        specialist_outcome: Some(outcome),
        fallback_used,
        // v0.45.x: Pass route capability's evidence_required for proper spine enforcement
        evidence_required: Some(det_route.capability.evidence_required),
    };

    let mut result = service_desk::build_result_with_flags(
        request_id, answer, query, ticket, probe_results.clone(), progress.transcript_clone(),
        classified_domain, translator_timed_out, used_deterministic,
        parsed_data_count, prompt_truncated, fallback_ctx,
    );

    // Step 10: Build execution trace
    let probe_stats = ProbeStats::from_results(ticket_probes_planned, &probe_results);
    let evidence_kinds = actual_evidence_kinds;

    result.execution_trace = Some(match outcome {
        SpecialistOutcome::Skipped => {
            // Deterministic route answered directly
            ExecutionTrace::deterministic_route(
                fallback_route_class.as_deref().unwrap_or("unknown"),
                probe_stats,
                evidence_kinds,
            )
        }
        SpecialistOutcome::Ok => {
            // Specialist LLM answered successfully
            ExecutionTrace::specialist_ok(probe_stats)
        }
        SpecialistOutcome::Timeout => {
            if used_deterministic && parsed_data_count > 0 {
                // Timeout with successful fallback
                ExecutionTrace::specialist_timeout_with_fallback(
                    fallback_route_class.as_deref().unwrap_or("unknown"),
                    probe_stats,
                    evidence_kinds,
                )
            } else {
                // Timeout without successful fallback
                ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
            }
        }
        SpecialistOutcome::Error => {
            if used_deterministic && parsed_data_count > 0 {
                // Error with successful fallback
                ExecutionTrace::specialist_error_with_fallback(
                    fallback_route_class.as_deref().unwrap_or("unknown"),
                    probe_stats,
                    evidence_kinds,
                )
            } else {
                // Error without successful fallback - treat like timeout
                ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
            }
        }
        SpecialistOutcome::BudgetExceeded => {
            // Budget exceeded - similar to timeout
            ExecutionTrace::specialist_timeout_no_fallback(probe_stats)
        }
    });

    // Add probe cap warning to evidence
    if probe_cap_warning {
        result.evidence.last_error = Some("probe_cap_applied".to_string());
    }

    progress.complete_stage(RequestStage::Supervisor);

    // Record total request latency
    let total_ms = request_start.elapsed().as_millis() as u64;
    {
        let mut state = state.write().await;
        state.latency.total.add(total_ms);
        // v0.0.79: Record stats
        let specialist_timeout = matches!(outcome, SpecialistOutcome::Timeout);
        state.record_request(used_deterministic, translator_timed_out, specialist_timeout);
    }

    info!("Request completed: domain={}, reliability={}, deterministic={}, trace={}, latency={}ms",
          result.domain, result.reliability_score, used_deterministic,
          result.execution_trace.as_ref().map(|t| t.to_string()).unwrap_or_default(), total_ms);

    save_progress(&state, &progress).await;
    // v0.0.94: Try to learn recipe from successful result
    success_with_learning(id, result)
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
) -> (anna_shared::rpc::TranslatorTicket, Option<TriageResult>, bool) {
    progress.start_stage(RequestStage::Translator, config.translator_timeout_secs);
    let translator_input = TranslatorInput::new(query, hw_cores, hw_ram_gb, has_gpu);
    let stage_start = Instant::now();

    let (llm_ticket, translator_timed_out) = match timeout(
        Duration::from_secs(config.translator_timeout_secs),
        translator::translate_with_context(translator_model, &translator_input, config.translator_timeout_secs),
    ).await {
        Ok(Ok(t)) => { progress.complete_stage(RequestStage::Translator); (Some(t), false) }
        Ok(Err(e)) => { warn!("Translator error: {}", e); progress.error_stage(RequestStage::Translator, &e); (None, false) }
        Err(_) => { warn!("Translator timeout"); progress.timeout_stage(RequestStage::Translator); (None, true) }
    };

    // Record translator latency
    { state.write().await.latency.translator.add(stage_start.elapsed().as_millis() as u64); }

    // If translator failed completely, use fallback
    let ticket = llm_ticket.unwrap_or_else(|| triage::create_fallback_ticket(query));

    // Apply triage rules
    let triage_result = triage::apply_triage_rules(ticket.clone());

    (triage_result.ticket.clone(), Some(triage_result), translator_timed_out)
}

/// Save progress events to state for polling
async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    state.write().await.progress_events = progress.events().to_vec();
}

/// v0.0.74: Build editor-specific syntax highlighting config answer using EditorRecipes.
/// Returns deterministic steps for the detected editor, no questions, no markdown.
fn build_editor_config_answer(editor: &str) -> String {
    use anna_shared::editor_recipes::{ConfigFeature, Editor, get_recipe};

    // Try to get recipe from EditorRecipes module
    if let Some(editor_enum) = Editor::from_tool_name(editor) {
        if let Some(recipe) = get_recipe(editor_enum, ConfigFeature::SyntaxHighlighting) {
            // Build answer from recipe
            let lines: Vec<String> = recipe.lines.iter()
                .map(|l| format!("   {}", l.line))
                .collect();

            return format!(
                "Detected {} installed. To enable syntax highlighting:\n\
                1. Edit ~/{}\n\
                2. Add the following:\n{}\n\
                3. Save and reopen {}\n\n\
                To undo: {}",
                editor_enum.display_name(),
                editor_enum.config_path(),
                lines.join("\n"),
                editor,
                recipe.rollback_hint
            );
        }
    }

    // Fallback for GUI editors without recipes (VS Code, Kate, Gedit)
    match editor {
        "code" => String::from(
            "Detected VS Code installed. Syntax highlighting is automatic based on file type.\n\
            To configure:\n\
            1. Open a file - VS Code detects language from extension\n\
            2. Click language indicator (bottom-right) to change mode\n\
            3. Install language extensions for better support (Ctrl+Shift+X)\n\
            Theme: File > Preferences > Color Theme"
        ),
        "kate" => String::from(
            "Detected Kate installed. Syntax highlighting is enabled by default.\n\
            To configure:\n\
            1. Settings > Configure Kate > Fonts & Colors\n\
            2. Select a color scheme\n\
            Line numbers: Settings > Configure Kate > Appearance > Show line numbers"
        ),
        "gedit" => String::from(
            "Detected gedit installed. Syntax highlighting is enabled by default.\n\
            To configure:\n\
            1. Preferences > Font & Colors\n\
            2. Select a color scheme\n\
            Line numbers: Preferences > View > Display line numbers"
        ),
        "helix" | "hx" => String::from(
            "Detected Helix installed. Syntax highlighting is enabled by default.\n\
            To customize themes:\n\
            1. Edit ~/.config/helix/config.toml\n\
            2. Add: theme = \"gruvbox\" (or another theme name)\n\
            3. Save and reopen helix\n\
            List themes with :theme command inside helix"
        ),
        "micro" => String::from(
            "Detected micro installed. Syntax highlighting is enabled by default.\n\
            To customize:\n\
            1. Edit ~/.config/micro/settings.json\n\
            2. Set \"colorscheme\": \"monokai\" (or another scheme)\n\
            For line numbers: set \"ruler\": true"
        ),
        _ => format!(
            "Detected {} installed. Check its documentation for syntax highlighting configuration.",
            editor
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.0.57: Vim answer must not contain question marks.
    #[test]
    fn test_vim_answer_no_questions() {
        let answer = build_editor_config_answer("vim");
        assert!(answer.contains("vim"), "Must mention vim");
        assert!(answer.contains(".vimrc"), "Must mention .vimrc");
        assert!(!answer.contains('?'), "Must not contain question marks");
    }

    /// v0.0.57: Nvim answer has correct config paths.
    #[test]
    fn test_nvim_answer_correct_paths() {
        let answer = build_editor_config_answer("nvim");
        assert!(answer.contains("nvim"), "Must mention nvim");
        assert!(answer.contains("init.vim") || answer.contains("init.lua"),
            "Must mention nvim config file");
        assert!(!answer.contains('?'), "Must not contain question marks");
    }

    /// v0.0.57: Each editor has specific answer without other editors' paths.
    #[test]
    fn test_editor_answers_are_specific() {
        let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit"];

        for editor in editors {
            let answer = build_editor_config_answer(editor);

            // Must mention the detected editor
            assert!(answer.to_lowercase().contains(editor) ||
                    (editor == "code" && answer.contains("VS Code")),
                "Answer for {} must mention the editor", editor);

            // Must NOT contain question marks
            assert!(!answer.contains('?'),
                "Answer for {} must not contain question marks", editor);

            // Answers should be distinct (not generic)
            assert!(answer.len() > 100,
                "Answer for {} should be detailed (got {} chars)", editor, answer.len());
        }
    }

    /// v0.0.57: vi is treated like vim.
    #[test]
    fn test_vi_uses_vim_config() {
        let answer = build_editor_config_answer("vi");
        assert!(answer.contains(".vimrc"), "vi should use vim config");
    }

    /// v0.0.66: Editor answers must not use markdown formatting.
    #[test]
    fn test_v066_editor_answers_no_markdown() {
        let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit"];

        for editor in editors {
            let answer = build_editor_config_answer(editor);
            // No markdown bold
            assert!(!answer.contains("**"),
                "Answer for {} must not contain markdown bold **", editor);
            // No markdown backticks for inline code
            // (Note: backticks are OK for literal shell commands)
        }
    }

    /// v0.0.66: Editor answers start with "Detected" statement.
    #[test]
    fn test_v066_editor_answers_start_with_detected() {
        let editors = ["vim", "nvim", "nano", "emacs", "helix", "micro", "code", "kate", "gedit"];

        for editor in editors {
            let answer = build_editor_config_answer(editor);
            assert!(answer.starts_with("Detected"),
                "Answer for {} must start with 'Detected', got: {}", editor, &answer[..40.min(answer.len())]);
        }
    }
}
