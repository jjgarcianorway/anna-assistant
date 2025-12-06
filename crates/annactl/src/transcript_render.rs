//! Transcript rendering for consistent pipeline visibility (v0.0.88).
//!
//! Two modes:
//! - debug OFF: Theatre mode - cinematic IT department experience (v0.0.81)
//! - debug ON: Full troubleshooting view with stages and timings
//!
//! v0.0.88: Removed unused render_clean functions (theatre_render is used instead).

use anna_shared::narrator::status_indicator;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, StageOutcome, TranscriptEventKind};
use anna_shared::ui::colors;

use crate::output::{format_for_output, OutputMode};
use crate::theatre_render;

/// Source of the final answer for display
enum AnswerSource<'a> {
    /// Answer is in the transcript (FinalAnswer event)
    Transcript,
    Clarification(&'a str),
    Answer(&'a str),
    Empty,
}

/// INVARIANT: Single source of truth for the final answer.
fn get_final_answer(result: &ServiceDeskResult) -> AnswerSource<'_> {
    for event in &result.transcript.events {
        if let TranscriptEventKind::FinalAnswer { .. } = &event.kind {
            debug_assert!(event.from == Actor::Anna, "FinalAnswer should be from Anna");
            debug_assert!(event.to == Some(Actor::You), "FinalAnswer should be to You");
            return AnswerSource::Transcript;
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

/// Render in debug mode - full troubleshooting view
/// v0.0.106: Shows case number and assigned staff
fn render_debug(result: &ServiceDeskResult, output_mode: OutputMode) {
    // v0.0.106: Show case number if present
    let case_info = match (&result.case_number, &result.assigned_staff) {
        (Some(cn), Some(staff)) => format!(" {}case={} staff={}{}", colors::CYAN, cn, staff, colors::RESET),
        (Some(cn), None) => format!(" {}case={}{}", colors::CYAN, cn, colors::RESET),
        _ => String::new(),
    };
    println!("\n{}[transcript]{} request_id={}{}\n", colors::DIM, colors::RESET, &result.request_id[..8], case_info);

    let answer_source = get_final_answer(result);
    let answer_in_transcript = matches!(answer_source, AnswerSource::Transcript);
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
            AnswerSource::Transcript => unreachable!(),
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
