//! Colored terminal output for Transcript System v0.0.70
//!
//! Provides colored print functions for human and debug modes.

use super::events::{ActorV70, EventV70, TranscriptStreamV70};
use crate::transcript_events::TranscriptMode;
use owo_colors::OwoColorize;

/// Print human mode with colors
pub fn print_human_colored(stream: &TranscriptStreamV70) {
    for te in &stream.events {
        match &te.event {
            EventV70::UserToAnna { text } => {
                println!("{} {}", "[you]".white().bold(), text);
            }

            EventV70::StaffMessage {
                from, message_human, ..
            } => {
                if from.visible_in_human() && !message_human.is_empty() {
                    let actor = format!("[{}]", from.display_name());
                    println!("{} {}", style_actor(from, &actor), message_human);
                }
            }

            EventV70::Evidence {
                actor,
                topic,
                summary_human,
                ..
            } => {
                if actor.visible_in_human() {
                    let actor_tag = format!("[{}]", actor.display_name());
                    println!(
                        "{} Evidence from {}: {}",
                        style_actor(actor, &actor_tag),
                        topic.human_description().cyan(),
                        summary_human
                    );
                }
            }

            EventV70::FinalAnswer {
                text,
                reliability,
                reliability_reason,
            } => {
                println!("{} {}", "[service-desk]".green(), text);
                println!();
                let rel_color = reliability_color(*reliability);
                println!("Reliability: {} ({})", rel_color, reliability_reason);
            }

            EventV70::Phase { name } => {
                println!("{}", format!("----- {} -----", name).dimmed());
            }

            EventV70::Working { actor, message } => {
                if actor.visible_in_human() {
                    let actor_tag = format!("[{}]", actor.display_name());
                    println!("{} {}", style_actor(actor, &actor_tag), message.dimmed());
                }
            }

            // Skip debug-only
            _ => {}
        }
    }
}

/// Print debug mode with colors
pub fn print_debug_colored(stream: &TranscriptStreamV70) {
    // Header
    if let Some(case_id) = &stream.case_id {
        println!("{}", format!("=== Case: {} ===", case_id).bold());
    }
    println!(
        "{}",
        format!(
            "Stats: {} tools ({}ms), {} parse warnings, {} retries, {} fallbacks",
            stream.stats.tool_call_count,
            stream.stats.total_tool_ms,
            stream.stats.parse_warning_count,
            stream.stats.retry_count,
            stream.stats.fallback_count
        )
        .dimmed()
    );
    println!();

    for te in &stream.events {
        let ts_str = te.ts.format("%H:%M:%S%.3f").to_string();
        let ts = ts_str.dimmed();

        match &te.event {
            EventV70::UserToAnna { text } => {
                println!("{} {} {}", ts, "[you]".white().bold(), text);
            }

            EventV70::StaffMessage {
                from,
                to,
                message_debug,
                ..
            } => {
                println!(
                    "{} {} -> {}: {}",
                    ts,
                    format!("[{}]", from.display_name()).cyan(),
                    format!("[{}]", to.display_name()).cyan(),
                    message_debug
                );
            }

            EventV70::Evidence {
                actor,
                tool_name,
                evidence_id,
                summary_debug,
                duration_ms,
                ..
            } => {
                let tool = tool_name.as_deref().unwrap_or("?");
                let eid = evidence_id.as_deref().unwrap_or("?");
                println!(
                    "{} {} {} tool={} ({}ms)",
                    ts,
                    format!("[{}]", actor.display_name()).cyan(),
                    format!("[{}]", eid).yellow(),
                    tool.green(),
                    duration_ms
                );
                println!("    {}", summary_debug.dimmed());
            }

            EventV70::ToolCall {
                tool_name,
                duration_ms,
                ..
            } => {
                println!(
                    "{} {} {} ({}ms)",
                    ts,
                    "[tool]".blue(),
                    tool_name.green(),
                    duration_ms
                );
            }

            EventV70::ToolResult {
                tool_name,
                success,
                raw_excerpt,
            } => {
                let status = if *success {
                    "OK".green().to_string()
                } else {
                    "FAIL".red().to_string()
                };
                println!("{} {} {} {}", ts, "[tool]".blue(), tool_name, status);
                if let Some(excerpt) = raw_excerpt {
                    println!("    {}", excerpt.dimmed());
                }
            }

            EventV70::ParseWarning {
                subsystem,
                details,
                fallback_used,
            } => {
                let fb = if *fallback_used {
                    " (fallback)".yellow().to_string()
                } else {
                    String::new()
                };
                println!(
                    "{} {} {}{}: {}",
                    ts,
                    "[WARN]".yellow().bold(),
                    subsystem,
                    fb,
                    details
                );
            }

            EventV70::TranslatorCanonical {
                intent,
                target,
                depth,
                topics,
                actions,
                safety,
            } => {
                println!("{} {} CANONICAL:", ts, "[translator]".magenta());
                println!("    INTENT: {}", intent.cyan());
                println!("    TARGET: {}", target);
                println!("    DEPTH: {}", depth);
                println!("    TOPICS: {}", topics.join(", "));
                println!("    ACTIONS: {}", actions.join(", "));
                println!("    SAFETY: {}", safety);
            }

            EventV70::Reliability {
                score,
                rationale_debug,
                ..
            } => {
                let color = reliability_color(*score);
                println!(
                    "{} {} {}: {}",
                    ts,
                    "[reliability]".cyan(),
                    color,
                    rationale_debug
                );
            }

            EventV70::FinalAnswer {
                text,
                reliability,
                reliability_reason,
            } => {
                println!(
                    "{} {} reliability={}% ({})",
                    ts,
                    "[FINAL]".green().bold(),
                    reliability,
                    reliability_reason
                );
                println!("    {}", text);
            }

            EventV70::Perf {
                total_ms,
                llm_ms,
                tool_ms,
                tool_count,
                retry_count,
            } => {
                println!(
                    "{} {} total={}ms, llm={:?}ms, tools={}ms ({}x), retries={}",
                    ts,
                    "[perf]".dimmed(),
                    total_ms,
                    llm_ms,
                    tool_ms,
                    tool_count,
                    retry_count
                );
            }

            EventV70::Phase { name } => {
                println!(
                    "{} {}",
                    ts,
                    format!("=== {} ===", name.to_uppercase()).bold()
                );
            }

            EventV70::Working { actor, message } => {
                println!(
                    "{} {} {}",
                    ts,
                    format!("[{}]", actor.display_name()).dimmed(),
                    message.dimmed()
                );
            }

            EventV70::Retry {
                subsystem,
                attempt,
                reason,
            } => {
                println!(
                    "{} {} {} attempt #{}: {}",
                    ts,
                    "[RETRY]".yellow(),
                    subsystem,
                    attempt,
                    reason
                );
            }
        }
    }
}

/// Print based on mode
pub fn print_colored(stream: &TranscriptStreamV70, mode: TranscriptMode) {
    match mode {
        TranscriptMode::Human => print_human_colored(stream),
        TranscriptMode::Debug | TranscriptMode::Test => print_debug_colored(stream),
    }
}

// Helper functions

fn style_actor(actor: &ActorV70, text: &str) -> String {
    match actor {
        ActorV70::You => text.white().bold().to_string(),
        ActorV70::ServiceDesk => text.green().to_string(),
        ActorV70::Networking => text.cyan().to_string(),
        ActorV70::Storage => text.blue().to_string(),
        ActorV70::Boot => text.magenta().to_string(),
        ActorV70::Audio => text.yellow().to_string(),
        ActorV70::Graphics => text.purple().to_string(),
        ActorV70::Security => text.red().to_string(),
        ActorV70::Performance => text.bright_cyan().to_string(),
        ActorV70::InfoDesk => text.white().to_string(),
        _ => text.dimmed().to_string(),
    }
}

fn reliability_color(score: u8) -> String {
    if score >= 80 {
        format!("{}%", score).green().to_string()
    } else if score >= 60 {
        format!("{}%", score).yellow().to_string()
    } else {
        format!("{}%", score).red().to_string()
    }
}
