//! Transcript rendering for consistent pipeline visibility.
//!
//! Two modes:
//! - debug OFF: Clean fly-on-the-wall format (user-facing)
//! - debug ON: Full troubleshooting view with stages and timings
//!
//! INVARIANT: Exactly one [anna] block per request regardless of mode.
//! The final answer source is determined by `get_final_answer()`.

use anna_shared::narrator::{it_confidence, it_domain_context, status_indicator};
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, StageOutcome, TranscriptEventKind};
use anna_shared::ui::colors;

use crate::output::{format_for_output, OutputMode};

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
pub fn render(result: &ServiceDeskResult, debug_mode: bool) {
    let output_mode = OutputMode::detect();
    if debug_mode {
        render_debug(result, output_mode);
    } else {
        render_clean(result, output_mode);
    }
}

/// Render in clean mode (debug OFF) - user-facing IT department format (v0.0.28)
fn render_clean(result: &ServiceDeskResult, output_mode: OutputMode) {
    println!();

    // Show user query
    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                println!("{}You:{} {}\n", colors::CYAN, colors::RESET, text);
                break;
            }
        }
    }

    // Show anna's response with IT department style
    println!("{}Anna:{}", colors::OK, colors::RESET);
    match get_final_answer(result) {
        AnswerSource::Transcript(t) | AnswerSource::Answer(t) => {
            println!("{}", format_for_output(t, output_mode));
        }
        AnswerSource::Clarification(t) => {
            println!("{}", format_for_output(t, output_mode));
        }
        AnswerSource::Empty => println!("I need more information to help with this request."),
    }
    println!();

    // IT department style footer
    let rel_color = reliability_color(result.reliability_score);
    let confidence_note = it_confidence(result.reliability_score);
    let domain_str = result.domain.to_string();
    let domain_context = it_domain_context(&domain_str);

    println!(
        "{}{} | {} | {}{}%{}",
        colors::DIM, domain_context, confidence_note,
        rel_color, result.reliability_score, colors::RESET
    );
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
