//! Transcript rendering for consistent pipeline visibility.
//!
//! Three modes:
//! - debug OFF: Theatre mode - cinematic IT department experience (v0.0.81)
//! - debug ON: Full troubleshooting view with stages and timings
//! - internal ON: Theatre mode with internal IT communications visible
//!
//! INVARIANT: Exactly one [anna] block per request regardless of mode.
//! The final answer source is determined by `get_final_answer()`.

use anna_shared::narrator::{it_confidence, it_domain_context, status_indicator};
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, StageOutcome, TranscriptEventKind};
use anna_shared::ui::colors;

use crate::output::{format_for_output, OutputMode};
use crate::theatre_render;

/// Source of the final answer for display
enum AnswerSource<'a> {
    Transcript(&'a str),
    Clarification(&'a str),
    Answer(&'a str),
    Empty,
}

/// INVARIANT: Single source of truth for the final answer.
fn get_final_answer(result: &ServiceDeskResult) -> AnswerSource<'_> {
    for event in &result.transcript.events {
        if let TranscriptEventKind::FinalAnswer { text } = &event.kind {
            debug_assert!(event.from == Actor::Anna, "FinalAnswer should be from Anna");
            debug_assert!(event.to == Some(Actor::You), "FinalAnswer should be to You");
            return AnswerSource::Transcript(text);
        }
    }
    if result.needs_clarification {
        return AnswerSource::Clarification(
            result.clarification_question.as_deref()
                .unwrap_or("I need more information to answer your question."),
        );
    }
    if !result.answer.is_empty() {
        return AnswerSource::Answer(&result.answer);
    }
    AnswerSource::Empty
}

/// Render transcript based on debug mode setting
/// v0.0.81: Clean mode now uses theatre rendering
pub fn render(result: &ServiceDeskResult, debug_mode: bool) {
    render_with_options(result, debug_mode, false)
}

/// Render with explicit internal communications option
pub fn render_with_options(result: &ServiceDeskResult, debug_mode: bool, show_internal: bool) {
    if debug_mode {
        let output_mode = OutputMode::detect();
        render_debug(result, output_mode);
    } else {
        // v0.0.81: Use theatre renderer for cinematic experience
        theatre_render::render_theatre(result, show_internal);
    }
}

/// Render in clean mode (debug OFF) - user-facing IT department format (v0.0.28)
/// v0.0.63: Service Desk Theatre with narrative flow
fn render_clean(result: &ServiceDeskResult, output_mode: OutputMode) {
    println!();

    // Show user query (v0.45.x: bracketed labels [you]/[anna])
    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                println!("{}[you]{} {}\n", colors::CYAN, colors::RESET, text);
                break;
            }
        }
    }

    // v0.0.63: Show evidence summary (what we checked, not raw output)
    render_evidence_summary_clean(&result.evidence);

    // Show anna's response with IT department style (v0.45.x: bracketed label)
    println!("{}[anna]{}", colors::OK, colors::RESET);
    match get_final_answer(result) {
        AnswerSource::Transcript(t) | AnswerSource::Answer(t) => {
            println!("{}", format_for_output(t, output_mode));
        }
        AnswerSource::Clarification(t) => {
            // v0.0.63: Show clarification options if available
            println!("{}", format_for_output(t, output_mode));
            render_clarification_options_clean(result);
        }
        AnswerSource::Empty => println!("I need more information to help with this request."),
    }
    println!();

    // IT department style footer with v0.0.63 evidence source
    let rel_color = reliability_color(result.reliability_score);
    let confidence_note = it_confidence(result.reliability_score);
    let domain_str = result.domain.to_string();
    let domain_context = it_domain_context(&domain_str);

    // v0.0.63: Add evidence source to footer if grounded
    let evidence_source = if result.reliability_signals.answer_grounded {
        format_evidence_source(&result.evidence, result.execution_trace.as_ref())
    } else {
        String::new()
    };

    if evidence_source.is_empty() {
        println!(
            "{}{} | {} | {}{}%{}",
            colors::DIM, domain_context, confidence_note,
            rel_color, result.reliability_score, colors::RESET
        );
    } else {
        println!(
            "{}{} | {} | {}{}%{} | {}{}",
            colors::DIM, domain_context, confidence_note,
            rel_color, result.reliability_score, colors::RESET,
            colors::DIM, evidence_source
        );
    }
}

/// v0.0.63: Show brief evidence summary without raw probe output
fn render_evidence_summary_clean(evidence: &anna_shared::rpc::EvidenceBlock) {
    if evidence.probes_executed.is_empty() {
        return;
    }

    let probe_count = evidence.probes_executed.len();
    let success_count = evidence.probes_executed.iter().filter(|p| p.exit_code == 0).count();

    // Describe what we checked based on probe commands
    let check_description = describe_probes_checked(&evidence.probes_executed);

    if !check_description.is_empty() {
        println!("{}{}...{}\n", colors::DIM, check_description, colors::RESET);
    } else if success_count > 0 {
        println!(
            "{}Checked {} data source{}...{}\n",
            colors::DIM,
            probe_count,
            if probe_count == 1 { "" } else { "s" },
            colors::RESET
        );
    }
}

/// v0.0.63: Describe what probes checked in human terms
fn describe_probes_checked(probes: &[anna_shared::rpc::ProbeResult]) -> String {
    // Categorize probes by what they check
    let mut has_audio = false;
    let mut has_editor = false;
    let mut has_memory = false;
    let mut has_disk = false;
    let mut has_cpu = false;
    let mut has_network = false;
    let mut has_services = false;

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if cmd.contains("audio") || cmd.contains("pactl") || cmd.contains("lspci") && cmd.contains("audio") {
            has_audio = true;
        }
        if cmd.contains("command -v") {
            has_editor = true;
        }
        if cmd.contains("free") {
            has_memory = true;
        }
        if cmd.contains("df") || cmd.contains("lsblk") {
            has_disk = true;
        }
        if cmd.contains("lscpu") || cmd.contains("sensors") {
            has_cpu = true;
        }
        if cmd.contains("ip ") || cmd.contains("ss ") {
            has_network = true;
        }
        if cmd.contains("systemctl") || cmd.contains("journalctl") {
            has_services = true;
        }
    }

    // Build description
    let mut parts = Vec::new();
    if has_audio { parts.push("audio hardware"); }
    if has_editor { parts.push("installed editors"); }
    if has_memory { parts.push("memory"); }
    if has_disk { parts.push("disk"); }
    if has_cpu { parts.push("CPU"); }
    if has_network { parts.push("network"); }
    if has_services { parts.push("services"); }

    if parts.is_empty() {
        return String::new();
    }

    format!("Checking {}", parts.join(", "))
}

/// v0.0.63: Show clarification options if available
fn render_clarification_options_clean(result: &ServiceDeskResult) {
    if let Some(ref clarify) = result.clarification_request {
        if !clarify.options.is_empty() {
            for (i, opt) in clarify.options.iter().enumerate() {
                println!("  {}. {}", i + 1, opt.label);
            }
        }
    }
}

/// v0.0.63: Format evidence source for footer
fn format_evidence_source(
    evidence: &anna_shared::rpc::EvidenceBlock,
    trace: Option<&anna_shared::trace::ExecutionTrace>,
) -> String {
    // Get evidence kinds from trace if available
    if let Some(t) = trace {
        if !t.evidence_kinds.is_empty() {
            let kinds: Vec<&str> = t.evidence_kinds.iter()
                .map(|k| match k {
                    anna_shared::trace::EvidenceKind::Audio => "lspci+pactl",
                    anna_shared::trace::EvidenceKind::ToolExists => "command -v",
                    anna_shared::trace::EvidenceKind::Memory => "free",
                    anna_shared::trace::EvidenceKind::Disk => "df",
                    anna_shared::trace::EvidenceKind::Cpu => "lscpu",
                    anna_shared::trace::EvidenceKind::Processes => "ps",
                    anna_shared::trace::EvidenceKind::Network => "ip",
                    anna_shared::trace::EvidenceKind::Services => "systemctl",
                    anna_shared::trace::EvidenceKind::Journal => "journalctl",
                    _ => "probe",
                })
                .collect();
            return format!("Verified from {}", kinds.join("+"));
        }
    }

    // Fallback: just say verified if we have probes
    if !evidence.probes_executed.is_empty() {
        let success = evidence.probes_executed.iter().filter(|p| p.exit_code == 0).count();
        if success > 0 {
            return format!("Verified from {} probe{}", success, if success == 1 { "" } else { "s" });
        }
    }

    String::new()
}

/// Render in debug mode - full troubleshooting view
fn render_debug(result: &ServiceDeskResult, output_mode: OutputMode) {
    println!("\n{}[transcript]{} request_id={}\n", colors::DIM, colors::RESET, &result.request_id[..8]);

    let answer_source = get_final_answer(result);
    let answer_in_transcript = matches!(answer_source, AnswerSource::Transcript(_));
    let mut last_actor: Option<Actor> = None;

    for event in &result.transcript.events {
        match &event.kind {
            TranscriptEventKind::Message { text } => {
                let actor = &event.from;
                if last_actor.as_ref() != Some(actor) {
                    println!("\n{}", format_actor_tag(actor));
                    last_actor = Some(*actor);
                }
                let formatted = if *actor == Actor::Anna {
                    format_for_output(text, output_mode)
                } else {
                    text.clone()
                };
                for line in formatted.lines().filter(|l| !l.trim().is_empty()) {
                    println!("{}", line);
                }
            }
            TranscriptEventKind::StageStart { stage } => {
                println!("\n{}[{}]{} starting...", colors::DIM, stage, colors::RESET);
                last_actor = None;
            }
            TranscriptEventKind::StageEnd { stage, outcome } => {
                println!("{}[{}]{} {}", colors::DIM, stage, colors::RESET, format_outcome(outcome));
            }
            TranscriptEventKind::ProbeStart { probe_id, command } => {
                println!("\n{}[probe]{}\n{} -> {}", colors::DIM, colors::RESET, probe_id, truncate(command, 50));
                last_actor = Some(Actor::Probe);
            }
            TranscriptEventKind::ProbeEnd { probe_id, exit_code, timing_ms, stdout_preview } => {
                let status = if *exit_code == 0 {
                    format!("{}ok{}", colors::OK, colors::RESET)
                } else {
                    format!("{}exit {}{}", colors::WARN, exit_code, colors::RESET)
                };
                let preview = stdout_preview.as_deref()
                    .map(|s| format!(" \"{}\"", truncate(s, 40)))
                    .unwrap_or_default();
                println!("{} {} ({}ms){}", probe_id, status, timing_ms, preview);
            }
            TranscriptEventKind::Note { text } => {
                println!("{}  note: {}{}", colors::DIM, text, colors::RESET);
            }
            TranscriptEventKind::FinalAnswer { text } => {
                println!("\n{}[anna]{}\n{}", colors::OK, colors::RESET, format_for_output(text, output_mode));
                last_actor = Some(Actor::Anna);
            }
            // Ticket lifecycle events
            TranscriptEventKind::TicketCreated { ticket_id, domain, intent, evidence_required } => {
                println!("\n{}[ticket]{} {} created (domain={}, intent={}, evidence={})",
                    colors::CYAN, colors::RESET, &ticket_id[..8.min(ticket_id.len())],
                    domain, intent, if *evidence_required { "required" } else { "optional" });
                last_actor = None;
            }
            TranscriptEventKind::TicketStatusChanged { from_status, to_status, .. } => {
                println!("{}[ticket]{} {} -> {}", colors::DIM, colors::RESET, from_status, to_status);
            }
            TranscriptEventKind::JuniorReview { attempt, score, verified, issues } => {
                let status = if *verified {
                    format!("{}verified{}", colors::OK, colors::RESET)
                } else {
                    format!("{}needs revision{}", colors::WARN, colors::RESET)
                };
                println!("\n{}[junior]{} attempt {} -> {} (score={})",
                    colors::CYAN, colors::RESET, attempt, status, score);
                if !issues.is_empty() && !*verified {
                    println!("{}  issues: {}{}", colors::DIM, issues.join(", "), colors::RESET);
                }
                last_actor = Some(Actor::Junior);
            }
            TranscriptEventKind::SeniorEscalation { successful, reason } => {
                let status = if *successful {
                    format!("{}provided guidance{}", colors::OK, colors::RESET)
                } else {
                    format!("{}could not help{}", colors::WARN, colors::RESET)
                };
                println!("\n{}[senior]{} escalation -> {}", colors::WARN, colors::RESET, status);
                if let Some(r) = reason {
                    println!("{}  reason: {}{}", colors::DIM, r, colors::RESET);
                }
                last_actor = Some(Actor::Senior);
            }
            TranscriptEventKind::RevisionApplied { changes_made } => {
                if !changes_made.is_empty() {
                    println!("{}[revision]{} {} change{}",
                        colors::DIM, colors::RESET, changes_made.len(),
                        if changes_made.len() == 1 { "" } else { "s" });
                    for change in changes_made {
                        println!("{}  - {}{}", colors::DIM, change, colors::RESET);
                    }
                }
            }
            // Review gate events (v0.0.26)
            TranscriptEventKind::ReviewGateDecision { decision, score, requires_llm } => {
                let llm_tag = if *requires_llm { " [needs LLM]" } else { "" };
                println!("\n{}[gate]{} {} (score={}){}",
                    colors::CYAN, colors::RESET, decision, score, llm_tag);
                last_actor = None;
            }
            TranscriptEventKind::TeamReview { team, reviewer, decision, issues_count } => {
                let issues_str = if *issues_count > 0 {
                    format!(", {} issue{}", issues_count, if *issues_count == 1 { "" } else { "s" })
                } else { String::new() };
                println!("{}[{}/{}]{} {}{}",
                    colors::CYAN, team, reviewer, colors::RESET, decision, issues_str);
                last_actor = None;
            }
            // Clarification events (v0.0.31)
            TranscriptEventKind::ClarificationAsked { question_id: _, prompt, choices, reason } => {
                println!("\n{}[clarify]{} {}", colors::WARN, colors::RESET, prompt);
                if !choices.is_empty() {
                    println!("{}  options: {}{}", colors::DIM, choices.join(", "), colors::RESET);
                }
                println!("{}  ({}){}", colors::DIM, reason, colors::RESET);
                last_actor = None;
            }
            TranscriptEventKind::ClarificationAnswered { question_id: _, answer } => {
                println!("{}[you]{} {}", colors::DIM, colors::RESET, answer);
                last_actor = Some(Actor::You);
            }
            TranscriptEventKind::ClarificationVerified { question_id: _, verified, source, alternatives } => {
                if *verified {
                    println!("{}[verify]{} {}confirmed{} ({})",
                        colors::DIM, colors::RESET, colors::OK, colors::RESET, source);
                } else {
                    println!("{}[verify]{} {}not found{} ({})",
                        colors::DIM, colors::RESET, colors::WARN, colors::RESET, source);
                    if !alternatives.is_empty() {
                        println!("{}  alternatives: {}{}", colors::DIM, alternatives.join(", "), colors::RESET);
                    }
                }
                last_actor = None;
            }
            TranscriptEventKind::FactStored { key, value, source } => {
                println!("{}[fact]{} {} = {} (via {})", colors::DIM, colors::RESET, key, value, source);
                last_actor = None;
            }
            // Fast path events (v0.0.39)
            TranscriptEventKind::FastPath { handled, class, reason, probes_needed } => {
                if *handled {
                    println!("{}[fast]{} {} {} (no LLM needed)",
                        colors::OK, colors::RESET, class, if *probes_needed { "(probes run)" } else { "(cached)" });
                } else {
                    println!("{}[fast]{} skipped: {}", colors::DIM, colors::RESET, reason);
                }
                last_actor = None;
            }
            // Timeout fallback events (v0.0.41)
            TranscriptEventKind::LlmTimeoutFallback { stage, timeout_secs, elapsed_secs, fallback_action } => {
                println!("{}[timeout]{} {} timed out ({}s > {}s) -> {}",
                    colors::WARN, colors::RESET, stage, elapsed_secs, timeout_secs, fallback_action);
                last_actor = None;
            }
            TranscriptEventKind::GracefulDegradation { reason, original_type, fallback_type } => {
                println!("{}[fallback]{} {} -> {} ({})",
                    colors::WARN, colors::RESET, original_type, fallback_type, reason);
                last_actor = None;
            }
            // Service Desk Theatre events (v0.0.63)
            TranscriptEventKind::EvidenceSummary { evidence_kinds, probe_count, key_findings } => {
                println!("{}[evidence]{} {} probe{}, kinds: {:?}",
                    colors::DIM, colors::RESET, probe_count,
                    if *probe_count == 1 { "" } else { "s" }, evidence_kinds);
                if !key_findings.is_empty() {
                    for finding in key_findings {
                        println!("{}  - {}{}", colors::DIM, finding, colors::RESET);
                    }
                }
                last_actor = None;
            }
            TranscriptEventKind::DeterministicPath { route_class, evidence_used } => {
                println!("{}[deterministic]{} {} (evidence: {:?})",
                    colors::OK, colors::RESET, route_class, evidence_used);
                last_actor = None;
            }
            TranscriptEventKind::ProposedAction { action_id, description, risk_level, rollback_available } => {
                let risk_color = match risk_level.as_str() {
                    "high" => colors::ERR,
                    "medium" => colors::WARN,
                    _ => colors::OK,
                };
                println!("\n{}[action]{} {} (risk: {}{}{})",
                    colors::WARN, colors::RESET, &action_id[..8.min(action_id.len())],
                    risk_color, risk_level, colors::RESET);
                println!("{}  {}{}", colors::DIM, description, colors::RESET);
                if *rollback_available {
                    println!("{}  rollback: available{}", colors::DIM, colors::RESET);
                }
                last_actor = None;
            }
            TranscriptEventKind::ActionConfirmationRequest { action_id: _, prompt, options } => {
                println!("{}[confirm]{} {}", colors::WARN, colors::RESET, prompt);
                if !options.is_empty() {
                    println!("{}  options: {}{}", colors::DIM, options.join(", "), colors::RESET);
                }
                last_actor = None;
            }
            TranscriptEventKind::Unknown => {}
        }
    }

    // Final answer if not in transcript
    if !answer_in_transcript {
        println!("\n{}[anna]{}", colors::OK, colors::RESET);
        match answer_source {
            AnswerSource::Clarification(t) | AnswerSource::Answer(t) => {
                println!("{}", format_for_output(t, output_mode));
            }
            AnswerSource::Empty => println!("(no answer generated)"),
            AnswerSource::Transcript(_) => unreachable!(),
        }
    }

    // Summary block
    let rel_color = reliability_color(result.reliability_score);
    println!("\n{}reliability:{} {}{}%{}   {}domain:{} {}   {}probes:{} {}",
        colors::DIM, colors::RESET, rel_color, result.reliability_score, colors::RESET,
        colors::DIM, colors::RESET, result.domain,
        colors::DIM, colors::RESET, result.evidence.probes_executed.len());

    let s = &result.reliability_signals;
    println!("{}signals: confident={} coverage={} grounded={} no_invention={} no_clarify={}{}",
        colors::DIM,
        status_indicator(s.translator_confident), status_indicator(s.probe_coverage),
        status_indicator(s.answer_grounded), status_indicator(s.no_invention),
        status_indicator(s.clarification_not_needed), colors::RESET);

    if let Some(trace) = &result.execution_trace {
        println!("{}trace: {}{}", colors::DIM, trace, colors::RESET);
    }
}

/// Format actor tag for debug mode
fn format_actor_tag(actor: &Actor) -> String {
    match actor {
        Actor::You => format!("{}[you]{}", colors::CYAN, colors::RESET),
        Actor::Anna => format!("{}[anna]{}", colors::OK, colors::RESET),
        Actor::Junior => format!("{}[junior]{}", colors::CYAN, colors::RESET),
        Actor::Senior => format!("{}[senior]{}", colors::WARN, colors::RESET),
        _ => format!("{}[{}]{}", colors::DIM, actor, colors::RESET),
    }
}

/// Format stage outcome
fn format_outcome(outcome: &StageOutcome) -> String {
    match outcome {
        StageOutcome::Ok => format!("{}ok{}", colors::OK, colors::RESET),
        StageOutcome::Timeout => format!("{}TIMEOUT{}", colors::ERR, colors::RESET),
        StageOutcome::Error => format!("{}ERROR{}", colors::ERR, colors::RESET),
        StageOutcome::Skipped => format!("{}skipped{}", colors::WARN, colors::RESET),
        StageOutcome::Deterministic => format!("{}skipped{} (deterministic)", colors::OK, colors::RESET),
        StageOutcome::BudgetExceeded { stage, budget_ms, elapsed_ms } => {
            format!("{}BUDGET_EXCEEDED{} ({}: {}ms > {}ms)", colors::ERR, colors::RESET, stage, elapsed_ms, budget_ms)
        }
        StageOutcome::ClarificationRequired { question, choices } => {
            format!("{}CLARIFY{} ({}, {} choices)", colors::WARN, colors::RESET, question, choices.len())
        }
    }
}

/// Get color for reliability score
fn reliability_color(score: u8) -> &'static str {
    match score {
        80..=100 => colors::OK,
        50..=79 => colors::WARN,
        _ => colors::ERR,
    }
}

/// Truncate text with ellipsis
fn truncate(s: &str, max: usize) -> String {
    let s = s.lines().next().unwrap_or(s);
    if s.len() > max { format!("{}...", &s[..max.saturating_sub(3)]) } else { s.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_color() {
        assert_eq!(reliability_color(100), colors::OK);
        assert_eq!(reliability_color(80), colors::OK);
        assert_eq!(reliability_color(79), colors::WARN);
        assert_eq!(reliability_color(49), colors::ERR);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_format_outcome_all_variants() {
        let _ok = format_outcome(&StageOutcome::Ok);
        let _timeout = format_outcome(&StageOutcome::Timeout);
        let _det = format_outcome(&StageOutcome::Deterministic);
        let _budget = format_outcome(&StageOutcome::BudgetExceeded {
            stage: "probes".to_string(), budget_ms: 12000, elapsed_ms: 15000,
        });
    }
}
// Guardrail tests are in tests/transcript_render_tests.rs
