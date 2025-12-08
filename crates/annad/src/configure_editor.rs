//! ConfigureEditor query handler.
//! v0.0.149: Extracted from rpc_handler.rs for modularization.

use anna_shared::parsers::{get_installed_tools, installed_editors_from_parsed, parse_probe_result};
use anna_shared::rpc::{ProbeResult, ServiceDeskResult, SpecialistDomain, TranslatorTicket};
use anna_shared::trace::{EvidenceKind, ExecutionTrace, ProbeStats, SpecialistOutcome};
use anna_shared::transcript::Transcript;
use tracing::info;

use crate::editor_config::build_editor_config_with_change;
use crate::service_desk::{self, FallbackContext};

/// Result of ConfigureEditor handling
pub enum ConfigureEditorResult {
    /// Handled completely - return this response
    Handled(ServiceDeskResult),
    /// Not applicable - continue with normal pipeline
    NotApplicable,
}

/// Handle ConfigureEditor query class.
/// Uses ONLY current probe evidence, no inventory.
/// v0.0.59: Original implementation
/// v0.0.62: Fixed probe accounting and execution trace for grounded output
/// v0.0.66: Multiple editors - statement with numbered options
/// v0.0.96: Single editor - propose config change using Safe Change Engine
pub fn handle_configure_editor(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> ConfigureEditorResult {
    // Parse probe_results to get installed editors from ToolExists evidence ONLY
    let parsed: Vec<_> = probe_results
        .iter()
        .map(|p| parse_probe_result(p))
        .collect();

    // Use dedicated helper for consistent editor extraction
    let installed_editors = installed_editors_from_parsed(&parsed);

    // Track what we checked (for no-editors-found message)
    let tools = get_installed_tools(&parsed);
    let checked_editors: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();

    // Count valid evidence for proper grounding
    let valid_evidence_count = parsed.iter().filter(|p| p.is_valid_evidence()).count();

    info!(
        "v0.0.149: ConfigureEditor - checked {:?}, found installed: {:?}, valid_evidence={}",
        checked_editors, installed_editors, valid_evidence_count
    );

    // Build execution trace for ConfigureEditor paths
    let probe_stats = ProbeStats::from_results(ticket.needs_probes.len(), probe_results);
    let evidence_kinds = vec![EvidenceKind::ToolExists];

    let fallback_ctx = FallbackContext {
        used_deterministic_fallback: false,
        fallback_route_class: "configure_editor".to_string(),
        evidence_kinds: vec!["tool_exists".to_string()],
        specialist_outcome: Some(SpecialistOutcome::Skipped),
        fallback_used: Some(anna_shared::trace::FallbackUsed::None),
        evidence_required: Some(true),
    };

    if installed_editors.is_empty() {
        // No editors found - grounded negative evidence (we checked, found none)
        let result = build_no_editors_response(
            request_id,
            query,
            ticket,
            probe_results,
            transcript,
            classified_domain,
            &checked_editors,
            valid_evidence_count,
            probe_stats,
            evidence_kinds,
            fallback_ctx,
        );
        ConfigureEditorResult::Handled(result)
    } else if installed_editors.len() == 1 {
        // Single editor - propose config change
        let result = build_single_editor_response(
            request_id,
            query,
            ticket,
            probe_results,
            transcript,
            classified_domain,
            &installed_editors[0],
            valid_evidence_count,
            probe_stats,
            evidence_kinds,
            fallback_ctx,
        );
        ConfigureEditorResult::Handled(result)
    } else {
        // Multiple editors - ask user to choose
        let result = build_multiple_editors_response(
            request_id,
            ticket,
            probe_results,
            transcript,
            &installed_editors,
        );
        ConfigureEditorResult::Handled(result)
    }
}

/// Build response when no editors are found
fn build_no_editors_response(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
    checked_editors: &[String],
    valid_evidence_count: usize,
    probe_stats: ProbeStats,
    evidence_kinds: Vec<EvidenceKind>,
    fallback_ctx: FallbackContext,
) -> ServiceDeskResult {
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

    let mut result = service_desk::build_result_with_flags(
        request_id,
        answer,
        query,
        ticket,
        probe_results.to_vec(),
        transcript,
        classified_domain,
        false,
        true,
        valid_evidence_count,
        false,
        fallback_ctx,
    );

    result.execution_trace = Some(ExecutionTrace::deterministic_route(
        "configure_editor",
        probe_stats,
        evidence_kinds,
    ));

    result
}

/// Build response when exactly one editor is found
fn build_single_editor_response(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
    editor: &str,
    valid_evidence_count: usize,
    probe_stats: ProbeStats,
    evidence_kinds: Vec<EvidenceKind>,
    fallback_ctx: FallbackContext,
) -> ServiceDeskResult {
    let (answer, proposed_change, proposed_changes) = build_editor_config_with_change(editor);

    let mut result = service_desk::build_result_with_flags(
        request_id,
        answer,
        query,
        ticket,
        probe_results.to_vec(),
        transcript,
        classified_domain,
        false,
        true,
        valid_evidence_count,
        false,
        fallback_ctx,
    );

    result.execution_trace = Some(ExecutionTrace::deterministic_route(
        "configure_editor",
        probe_stats,
        evidence_kinds,
    ));

    // Set proposed change for CLI confirmation
    result.proposed_change = proposed_change;
    result.proposed_changes = proposed_changes;

    result
}

/// Build response when multiple editors are found
fn build_multiple_editors_response(
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    installed_editors: &[String],
) -> ServiceDeskResult {
    // Format: "I can configure syntax highlighting for one of these editors:\n1) vim\n2) code\nReply with the number."
    let editors_list: Vec<String> = installed_editors
        .iter()
        .enumerate()
        .map(|(i, e)| format!("{}) {}", i + 1, e))
        .collect();

    let answer = format!(
        "I can configure syntax highlighting for one of these editors:\n{}\nReply with the number.",
        editors_list.join("\n")
    );

    let options: Vec<(String, String)> = installed_editors
        .iter()
        .map(|e| (e.clone(), e.clone()))
        .collect();

    // Build result with clarification but answer text is a statement
    let mut result = service_desk::create_clarification_with_options(
        request_id,
        ticket,
        &answer,
        options,
        probe_results.to_vec(),
        transcript,
    );

    // Override answer with clean statement (no question)
    result.answer = answer;

    result
}
