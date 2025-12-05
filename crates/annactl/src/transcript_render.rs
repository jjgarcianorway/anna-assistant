//! Transcript rendering for consistent pipeline visibility.
//!
//! Two modes:
//! - debug OFF: Clean fly-on-the-wall format (user-facing)
//! - debug ON: Full troubleshooting view with stages and timings

use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, StageOutcome, TranscriptEventKind};
use anna_shared::ui::colors;
use anna_shared::VERSION;

/// Render transcript based on debug mode setting
pub fn render(result: &ServiceDeskResult, debug_mode: bool) {
    if debug_mode {
        render_debug(result);
    } else {
        render_clean(result);
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
fn render_clean(result: &ServiceDeskResult) {
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

    // Show anna's response
    println!("{}[anna]{}", colors::OK, colors::RESET);

    if result.needs_clarification {
        if let Some(question) = &result.clarification_question {
            println!("{}", question);
        } else {
            println!("I need more information to answer your question.");
        }
    } else if result.answer.is_empty() {
        println!("I couldn't find an answer. Please try rephrasing your question.");
    } else {
        println!("{}", result.answer);
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
fn render_debug(result: &ServiceDeskResult) {
    println!();
    println!(
        "{}[transcript]{} request_id={}",
        colors::DIM,
        colors::RESET,
        &result.request_id[..8] // Short ID
    );
    println!();

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
                // Content on next line, indented
                for line in text.lines() {
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
        }
    }

    // Ensure final [anna] block with answer
    println!();
    println!("{}[anna]{}", colors::OK, colors::RESET);
    if result.needs_clarification {
        if let Some(q) = &result.clarification_question {
            println!("{}", q);
        }
    } else if !result.answer.is_empty() {
        println!("{}", result.answer);
    } else {
        println!("(no answer generated)");
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
}
