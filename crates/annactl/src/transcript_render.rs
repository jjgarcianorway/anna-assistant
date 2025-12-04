//! Transcript rendering for consistent pipeline visibility.
//!
//! Two modes:
//! - debug OFF: Human-readable fly-on-the-wall format showing key messages
//! - debug ON: Full troubleshooting view with stage timings and internal details

use anna_shared::rpc::ServiceDeskResult;
use anna_shared::transcript::{Actor, StageOutcome, TranscriptEvent, TranscriptEventKind};
use anna_shared::ui::{colors, HR};

/// Render a complete transcript in debug OFF mode (human readable)
pub fn render_human(result: &ServiceDeskResult) {
    println!();

    // Show user message
    for event in &result.transcript.events {
        if let TranscriptEventKind::Message { text } = &event.kind {
            if event.from == Actor::You {
                println!("{}[you -> anna]{}", colors::CYAN, colors::RESET);
                println!("{}", text);
                println!();
            }
        }
    }

    // Check if clarification needed
    if result.needs_clarification {
        if let Some(question) = &result.clarification_question {
            println!(
                "{}[anna -> you]{} needs clarification",
                colors::WARN,
                colors::RESET
            );
            println!("{}", question);
            println!("{}{}{}", colors::DIM, HR, colors::RESET);
            return;
        }
    }

    // Show Anna's final response
    println!(
        "{}[anna -> you]{} {}",
        colors::OK,
        colors::RESET,
        format_reliability_badge(result.reliability_score)
    );
    println!("{}", result.answer);

    // Show probes used (summary)
    let probes_used: Vec<&str> = result
        .evidence
        .probes_executed
        .iter()
        .filter(|p| p.exit_code == 0)
        .map(|p| p.command.as_str())
        .collect();

    if !probes_used.is_empty() {
        println!();
        println!(
            "{}probes:{} {}",
            colors::DIM,
            colors::RESET,
            probes_used.join(", ")
        );
    }

    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Render a complete transcript in debug ON mode (detailed troubleshooting)
pub fn render_debug(result: &ServiceDeskResult) {
    println!();
    println!(
        "{}[transcript]{} request_id={}",
        colors::DIM,
        colors::RESET,
        result.request_id
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    for event in &result.transcript.events {
        render_event_debug(event);
    }

    // Final summary
    println!();
    println!(
        "{}[summary]{} domain={} reliability={} probes={}",
        colors::DIM,
        colors::RESET,
        result.domain,
        format_reliability_badge(result.reliability_score),
        result.evidence.probes_executed.len()
    );

    // Show reliability signals
    let signals = &result.reliability_signals;
    println!(
        "{}         signals: confident={} coverage={} grounded={} no_invention={} no_clarify={}{}",
        colors::DIM,
        bool_symbol(signals.translator_confident),
        bool_symbol(signals.probe_coverage),
        bool_symbol(signals.answer_grounded),
        bool_symbol(signals.no_invention),
        bool_symbol(signals.clarification_not_needed),
        colors::RESET
    );

    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Render a single transcript event in debug mode
fn render_event_debug(event: &TranscriptEvent) {
    let elapsed = format!("{:>6.1}s", event.elapsed_ms as f64 / 1000.0);

    match &event.kind {
        TranscriptEventKind::Message { text } => {
            let from = format_actor(&event.from);
            let to = event.to.as_ref().map(format_actor).unwrap_or_default();
            let arrow = if to.is_empty() { "" } else { " -> " };
            println!(
                "{}[{}]{} {}{}{}{}{}",
                colors::DIM,
                elapsed,
                colors::RESET,
                from,
                arrow,
                to,
                if to.is_empty() { "" } else { " " },
                truncate(text, 80)
            );
        }
        TranscriptEventKind::StageStart { stage } => {
            println!(
                "{}[{}] [stage]{} {} starting",
                colors::DIM,
                elapsed,
                colors::RESET,
                stage
            );
        }
        TranscriptEventKind::StageEnd { stage, outcome } => {
            let outcome_str = match outcome {
                StageOutcome::Ok => format!("{}ok{}", colors::OK, colors::RESET),
                StageOutcome::Timeout => format!("{}TIMEOUT{}", colors::ERR, colors::RESET),
                StageOutcome::Error => format!("{}ERROR{}", colors::ERR, colors::RESET),
                StageOutcome::Skipped => format!("{}skipped{}", colors::WARN, colors::RESET),
            };
            println!(
                "{}[{}] [stage]{} {} {}",
                colors::DIM,
                elapsed,
                colors::RESET,
                stage,
                outcome_str
            );
        }
        TranscriptEventKind::ProbeStart { probe_id, command } => {
            println!(
                "{}[{}] [probe]{} {} -> {}",
                colors::DIM,
                elapsed,
                colors::RESET,
                probe_id,
                truncate(command, 40)
            );
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
                .map(|s| format!(" \"{}\"", truncate(s, 30)))
                .unwrap_or_default();
            println!(
                "{}[{}] [probe]{} {} {} ({}ms){}",
                colors::DIM,
                elapsed,
                colors::RESET,
                probe_id,
                status,
                timing_ms,
                preview
            );
        }
        TranscriptEventKind::Note { text } => {
            println!(
                "{}[{}] [note] {}{}",
                colors::DIM,
                elapsed,
                text,
                colors::RESET
            );
        }
    }
}

/// Format an actor name with color
fn format_actor(actor: &Actor) -> String {
    match actor {
        Actor::You => format!("{}you{}", colors::CYAN, colors::RESET),
        Actor::Anna => format!("{}anna{}", colors::OK, colors::RESET),
        Actor::Translator => format!("{}translator{}", colors::DIM, colors::RESET),
        Actor::Dispatcher => format!("{}dispatcher{}", colors::DIM, colors::RESET),
        Actor::Probe => format!("{}probe{}", colors::DIM, colors::RESET),
        Actor::Specialist => format!("{}specialist{}", colors::DIM, colors::RESET),
        Actor::Supervisor => format!("{}supervisor{}", colors::DIM, colors::RESET),
        Actor::System => format!("{}system{}", colors::DIM, colors::RESET),
    }
}

/// Format reliability score with color badge
fn format_reliability_badge(score: u8) -> String {
    let color = if score >= 80 {
        colors::OK
    } else if score >= 50 {
        colors::WARN
    } else {
        colors::ERR
    };
    format!("{}reliability {}%{}", color, score, colors::RESET)
}

/// Boolean to checkmark/cross
fn bool_symbol(b: bool) -> &'static str {
    if b {
        "\x1b[32m\u{2713}\x1b[0m"
    } else {
        "\x1b[31m\u{2717}\x1b[0m"
    }
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

/// Render transcript based on debug mode setting
pub fn render(result: &ServiceDeskResult, debug_mode: bool) {
    if debug_mode {
        render_debug(result);
    } else {
        render_human(result);
    }
}
