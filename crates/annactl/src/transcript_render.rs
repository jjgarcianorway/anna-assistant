//! Transcript rendering for consistent pipeline visibility.
//!
//! Two modes:
//! - debug OFF: Clean fly-on-the-wall format (user-facing)
//! - debug ON: Full troubleshooting view with stages and timings
//!
//! INVARIANT: Exactly one [anna] block per request regardless of mode.
//! The final answer source is determined by `get_final_answer()`.
//!
//! TTY awareness: When piped, markdown tables are converted to plain text.

use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, StageOutcome, TranscriptEventKind};
use anna_shared::ui::colors;
use anna_shared::VERSION;

use crate::output::{format_for_output, OutputMode};

/// Source of the final answer for display
enum AnswerSource<'a> {
    /// Answer came from transcript (Anna message already recorded)
    Transcript(&'a str),
    /// Clarification needed (from result)
    Clarification(&'a str),
    /// Direct answer (from result)
    Answer(&'a str),
    /// No answer available
    Empty,
}

/// INVARIANT: Single source of truth for the final answer.
/// Used by both render_clean() and render_debug() to ensure consistency.
///
/// The FinalAnswer event kind IS THE CONTRACT for answer source.
/// Defense-in-depth checks (from == Anna, to == You) are guaranteed by
/// TranscriptEvent::final_answer() but verified here anyway.
fn get_final_answer(result: &ServiceDeskResult) -> AnswerSource<'_> {
    // First, check if FinalAnswer event is present in transcript (THE contract)
    for event in &result.transcript.events {
        if let TranscriptEventKind::FinalAnswer { text } = &event.kind {
            // Defense-in-depth: FinalAnswer should always be Anna -> You
            debug_assert!(event.from == Actor::Anna, "FinalAnswer should be from Anna");
            debug_assert!(event.to == Some(Actor::You), "FinalAnswer should be to You");
            return AnswerSource::Transcript(text);
        }
    }

    // Not in transcript - use result fields
    if result.needs_clarification {
        if let Some(q) = &result.clarification_question {
            return AnswerSource::Clarification(q);
        }
        return AnswerSource::Clarification("I need more information to answer your question.");
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

/// Render in clean mode (debug OFF) - user-facing format
/// Format:
///   anna vX.Y.Z
///   [you]
///   <query>
///   [anna]
///   <final answer>
///   reliability: NN%   domain: <domain>
fn render_clean(result: &ServiceDeskResult, output_mode: OutputMode) {
    println!();
    println!("anna v{}", VERSION);
    println!();

    // Show user query
    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                println!("{}[you]{}", colors::CYAN, colors::RESET);
                println!("{}", text);
                println!();
                break;
            }
        }
    }

    // Show anna's response (unified source, TTY-aware formatting)
    println!("{}[anna]{}", colors::OK, colors::RESET);
    match get_final_answer(result) {
        AnswerSource::Transcript(text) | AnswerSource::Answer(text) | AnswerSource::Clarification(text) => {
            println!("{}", format_for_output(text, output_mode));
        }
        AnswerSource::Empty => {
            println!("I couldn't find an answer. Please try rephrasing your question.");
        }
    }
    println!();

    // Footer with reliability and domain
    let rel_color = reliability_color(result.reliability_score);
    println!(
        "{}reliability:{} {}{}%{}   {}domain:{} {}",
        colors::DIM,
        colors::RESET,
        rel_color,
        result.reliability_score,
        colors::RESET,
        colors::DIM,
        colors::RESET,
        result.domain
    );
}

/// Render in debug mode - full troubleshooting view
fn render_debug(result: &ServiceDeskResult, output_mode: OutputMode) {
    println!();
    println!(
        "{}[transcript]{} request_id={}",
        colors::DIM,
        colors::RESET,
        &result.request_id[..8] // Short ID
    );
    println!();

    // Get answer source ONCE at the start (unified logic)
    let answer_source = get_final_answer(result);
    let answer_in_transcript = matches!(answer_source, AnswerSource::Transcript(_));

    let mut last_actor: Option<Actor> = None;

    for event in &result.transcript.events {
        match &event.kind {
            TranscriptEventKind::Message { text } => {
                let actor = &event.from;
                // New speaker tag on new line
                if last_actor.as_ref() != Some(actor) {
                    println!();
                    println!("{}", format_actor_tag(actor));
                    last_actor = Some(*actor);
                }
                // Content on next line (TTY-aware formatting for Anna messages)
                let formatted = if *actor == Actor::Anna {
                    format_for_output(text, output_mode)
                } else {
                    text.clone()
                };
                for line in formatted.lines() {
                    if !line.trim().is_empty() {
                        println!("{}", line);
                    }
                }
            }
            TranscriptEventKind::StageStart { stage } => {
                println!();
                println!(
                    "{}[{}]{} starting...",
                    colors::DIM,
                    stage,
                    colors::RESET
                );
                last_actor = None;
            }
            TranscriptEventKind::StageEnd { stage, outcome } => {
                let outcome_str = format_outcome(outcome);
                println!(
                    "{}[{}]{} {}",
                    colors::DIM,
                    stage,
                    colors::RESET,
                    outcome_str
                );
            }
            TranscriptEventKind::ProbeStart { probe_id, command } => {
                println!();
                println!("{}[probe]{}", colors::DIM, colors::RESET);
                println!("{} -> {}", probe_id, truncate(command, 50));
                last_actor = Some(Actor::Probe);
            }
            TranscriptEventKind::ProbeEnd {
                probe_id,
                exit_code,
                timing_ms,
                stdout_preview,
            } => {
                let status = if *exit_code == 0 {
                    format!("{}ok{}", colors::OK, colors::RESET)
                } else {
                    format!("{}exit {}{}", colors::WARN, exit_code, colors::RESET)
                };
                let preview = stdout_preview
                    .as_deref()
                    .map(|s| format!(" \"{}\"", truncate(s, 40)))
                    .unwrap_or_default();
                println!("{} {} ({}ms){}", probe_id, status, timing_ms, preview);
            }
            TranscriptEventKind::Note { text } => {
                println!(
                    "{}  note: {}{}",
                    colors::DIM,
                    text,
                    colors::RESET
                );
            }
            TranscriptEventKind::FinalAnswer { text } => {
                // THE final answer - always print with [anna] tag
                println!();
                println!("{}[anna]{}", colors::OK, colors::RESET);
                println!("{}", format_for_output(text, output_mode));
                last_actor = Some(Actor::Anna);
            }
            TranscriptEventKind::Unknown => {
                // Forward-compatible: silently skip unknown events
            }
        }
    }

    // Only print final [anna] block if answer wasn't already in transcript
    if !answer_in_transcript {
        println!();
        println!("{}[anna]{}", colors::OK, colors::RESET);
        match answer_source {
            AnswerSource::Clarification(text) | AnswerSource::Answer(text) => {
                println!("{}", format_for_output(text, output_mode));
            }
            AnswerSource::Empty => {
                println!("(no answer generated)");
            }
            AnswerSource::Transcript(_) => unreachable!(), // Already handled above
        }
    }

    // Summary block
    println!();
    let rel_color = reliability_color(result.reliability_score);
    println!(
        "{}reliability:{} {}{}%{}   {}domain:{} {}   {}probes:{} {}",
        colors::DIM,
        colors::RESET,
        rel_color,
        result.reliability_score,
        colors::RESET,
        colors::DIM,
        colors::RESET,
        result.domain,
        colors::DIM,
        colors::RESET,
        result.evidence.probes_executed.len()
    );

    // Signals in debug mode
    let signals = &result.reliability_signals;
    println!(
        "{}signals: confident={} coverage={} grounded={} no_invention={} no_clarify={}{}",
        colors::DIM,
        bool_symbol(signals.translator_confident),
        bool_symbol(signals.probe_coverage),
        bool_symbol(signals.answer_grounded),
        bool_symbol(signals.no_invention),
        bool_symbol(signals.clarification_not_needed),
        colors::RESET
    );
}

/// Format actor tag for debug mode
fn format_actor_tag(actor: &Actor) -> String {
    match actor {
        Actor::You => format!("{}[you]{}", colors::CYAN, colors::RESET),
        Actor::Anna => format!("{}[anna]{}", colors::OK, colors::RESET),
        Actor::Translator => format!("{}[translator]{}", colors::DIM, colors::RESET),
        Actor::Dispatcher => format!("{}[dispatcher]{}", colors::DIM, colors::RESET),
        Actor::Probe => format!("{}[probe]{}", colors::DIM, colors::RESET),
        Actor::Specialist => format!("{}[specialist]{}", colors::DIM, colors::RESET),
        Actor::Supervisor => format!("{}[supervisor]{}", colors::DIM, colors::RESET),
        Actor::System => format!("{}[system]{}", colors::DIM, colors::RESET),
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
            format!("{}BUDGET_EXCEEDED{} ({}: {}ms > {}ms)",
                colors::ERR, colors::RESET, stage, elapsed_ms, budget_ms)
        }
    }
}

/// Get color for reliability score
fn reliability_color(score: u8) -> &'static str {
    if score >= 80 {
        colors::OK
    } else if score >= 50 {
        colors::WARN
    } else {
        colors::ERR
    }
}

/// Boolean to checkmark/cross symbol
fn bool_symbol(b: bool) -> &'static str {
    if b { "✓" } else { "✗" }
}

/// Truncate text with ellipsis
fn truncate(s: &str, max: usize) -> String {
    let s = s.lines().next().unwrap_or(s);
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_color() {
        assert_eq!(reliability_color(100), colors::OK);
        assert_eq!(reliability_color(80), colors::OK);
        assert_eq!(reliability_color(79), colors::WARN);
        assert_eq!(reliability_color(50), colors::WARN);
        assert_eq!(reliability_color(49), colors::ERR);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_bool_symbol() {
        assert_eq!(bool_symbol(true), "✓");
        assert_eq!(bool_symbol(false), "✗");
    }

    #[test]
    fn test_format_outcome_deterministic() {
        let outcome = format_outcome(&StageOutcome::Deterministic);
        assert!(outcome.contains("skipped"));
        assert!(outcome.contains("deterministic"));
    }

    #[test]
    fn test_format_outcome_all_variants() {
        // Ensure all variants are handled (compile-time check)
        let _ok = format_outcome(&StageOutcome::Ok);
        let _timeout = format_outcome(&StageOutcome::Timeout);
        let _error = format_outcome(&StageOutcome::Error);
        let _skipped = format_outcome(&StageOutcome::Skipped);
        let _det = format_outcome(&StageOutcome::Deterministic);
        let _budget = format_outcome(&StageOutcome::BudgetExceeded {
            stage: "probes".to_string(),
            budget_ms: 12000,
            elapsed_ms: 15000,
        });
    }
}

// Guardrail tests are in tests/transcript_render_tests.rs
