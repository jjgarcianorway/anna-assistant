//! Text rendering functions for Transcript System v0.0.70
//!
//! Human Mode: IT department dialogue without internal details
//! Debug Mode: Full transparency with tool names, evidence IDs, timing

use super::events::{EventV70, TranscriptStreamV70};
use crate::transcript_events::TranscriptMode;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

/// Render transcript for human mode - no internal details
pub fn render_human(stream: &TranscriptStreamV70) -> Vec<String> {
    let mut lines = Vec::new();

    for te in &stream.events {
        match &te.event {
            EventV70::UserToAnna { text } => {
                lines.push(format!("[you] {}", text));
            }

            EventV70::StaffMessage {
                from, message_human, ..
            } => {
                if from.visible_in_human() && !message_human.is_empty() {
                    lines.push(format!("[{}] {}", from.display_name(), message_human));
                }
            }

            EventV70::Evidence {
                actor,
                topic,
                summary_human,
                ..
            } => {
                if actor.visible_in_human() {
                    lines.push(format!(
                        "[{}] Evidence from {}: {}",
                        actor.display_name(),
                        topic.human_description(),
                        summary_human
                    ));
                }
            }

            // Debug-only events - skip in human mode
            EventV70::ToolCall { .. }
            | EventV70::ToolResult { .. }
            | EventV70::ParseWarning { .. }
            | EventV70::TranslatorCanonical { .. }
            | EventV70::Perf { .. }
            | EventV70::Retry { .. } => {}

            EventV70::Reliability {
                score,
                rationale_human,
                ..
            } => {
                let desc = reliability_desc(*score);
                lines.push(format!(
                    "Reliability: {}% ({}, {})",
                    score, desc, rationale_human
                ));
            }

            EventV70::FinalAnswer {
                text,
                reliability,
                reliability_reason,
            } => {
                lines.push(format!("[service-desk] {}", text));
                lines.push(String::new());
                lines.push(format!(
                    "Reliability: {}% ({})",
                    reliability, reliability_reason
                ));
            }

            EventV70::Phase { name } => {
                lines.push(format!("----- {} -----", name));
            }

            EventV70::Working { actor, message } => {
                if actor.visible_in_human() {
                    lines.push(format!("[{}] {}", actor.display_name(), message));
                }
            }
        }
    }

    lines
}

/// Render transcript for debug mode - full transparency
pub fn render_debug(stream: &TranscriptStreamV70) -> Vec<String> {
    let mut lines = Vec::new();

    // Header with stats
    if let Some(case_id) = &stream.case_id {
        lines.push(format!("=== Case: {} ===", case_id));
    }
    lines.push(format!(
        "Stats: {} tools ({}ms), {} parse warnings, {} retries, {} fallbacks",
        stream.stats.tool_call_count,
        stream.stats.total_tool_ms,
        stream.stats.parse_warning_count,
        stream.stats.retry_count,
        stream.stats.fallback_count
    ));
    lines.push(String::new());

    for te in &stream.events {
        let ts = te.ts.format("%H:%M:%S%.3f");

        match &te.event {
            EventV70::UserToAnna { text } => {
                lines.push(format!("{} [you] {}", ts, text));
            }

            EventV70::StaffMessage {
                from,
                to,
                message_debug,
                ..
            } => {
                lines.push(format!(
                    "{} [{}] -> [{}]: {}",
                    ts,
                    from.display_name(),
                    to.display_name(),
                    message_debug
                ));
            }

            EventV70::Evidence {
                actor,
                topic,
                summary_debug,
                tool_name,
                evidence_id,
                duration_ms,
                ..
            } => {
                let tool = tool_name.as_deref().unwrap_or("?");
                let eid = evidence_id.as_deref().unwrap_or("?");
                lines.push(format!(
                    "{} [{}] EVIDENCE [{}] tool={} topic={:?} ({}ms)",
                    ts,
                    actor.display_name(),
                    eid,
                    tool,
                    topic,
                    duration_ms
                ));
                lines.push(format!("    {}", summary_debug));
            }

            EventV70::ToolCall {
                tool_name,
                args,
                duration_ms,
            } => {
                let args_str = args.as_deref().unwrap_or("");
                lines.push(format!(
                    "{} [tool] CALL {} {} ({}ms)",
                    ts, tool_name, args_str, duration_ms
                ));
            }

            EventV70::ToolResult {
                tool_name,
                success,
                raw_excerpt,
            } => {
                let status = if *success { "OK" } else { "FAIL" };
                lines.push(format!("{} [tool] RESULT {} {}", ts, tool_name, status));
                if let Some(excerpt) = raw_excerpt {
                    lines.push(format!("    {}", excerpt));
                }
            }

            EventV70::ParseWarning {
                subsystem,
                details,
                fallback_used,
            } => {
                let fb = if *fallback_used {
                    " (deterministic fallback)"
                } else {
                    ""
                };
                lines.push(format!(
                    "{} [WARN] {} parse warning{}: {}",
                    ts, subsystem, fb, details
                ));
            }

            EventV70::TranslatorCanonical {
                intent,
                target,
                depth,
                topics,
                actions,
                safety,
            } => {
                lines.push(format!("{} [translator] CANONICAL OUTPUT:", ts));
                lines.push(format!("    INTENT: {}", intent));
                lines.push(format!("    TARGET: {}", target));
                lines.push(format!("    DEPTH: {}", depth));
                lines.push(format!("    TOPICS: {}", topics.join(", ")));
                lines.push(format!(
                    "    ACTIONS: {}",
                    if actions.is_empty() {
                        "none".to_string()
                    } else {
                        actions.join(", ")
                    }
                ));
                lines.push(format!("    SAFETY: {}", safety));
            }

            EventV70::Reliability {
                score,
                rationale_debug,
                ..
            } => {
                lines.push(format!(
                    "{} [reliability] score={}: {}",
                    ts, score, rationale_debug
                ));
            }

            EventV70::Perf {
                total_ms,
                llm_ms,
                tool_ms,
                tool_count,
                retry_count,
            } => {
                lines.push(format!(
                    "{} [perf] total={}ms, llm={:?}ms, tools={}ms ({}x), retries={}",
                    ts, total_ms, llm_ms, tool_ms, tool_count, retry_count
                ));
            }

            EventV70::Retry {
                subsystem,
                attempt,
                reason,
            } => {
                lines.push(format!(
                    "{} [RETRY] {} attempt #{}: {}",
                    ts, subsystem, attempt, reason
                ));
            }

            EventV70::FinalAnswer {
                text,
                reliability,
                reliability_reason,
            } => {
                lines.push(format!(
                    "{} [FINAL] reliability={}% ({})",
                    ts, reliability, reliability_reason
                ));
                lines.push(format!("    {}", text));
            }

            EventV70::Phase { name } => {
                lines.push(format!("{} === {} ===", ts, name.to_uppercase()));
            }

            EventV70::Working { actor, message } => {
                lines.push(format!(
                    "{} [{}] WORKING: {}",
                    ts,
                    actor.display_name(),
                    message
                ));
            }
        }
    }

    lines
}

/// Render based on mode
pub fn render(stream: &TranscriptStreamV70, mode: TranscriptMode) -> Vec<String> {
    match mode {
        TranscriptMode::Human => render_human(stream),
        TranscriptMode::Debug | TranscriptMode::Test => render_debug(stream),
    }
}

/// Render to string
pub fn render_to_string(stream: &TranscriptStreamV70, mode: TranscriptMode) -> String {
    render(stream, mode).join("\n")
}

/// Write both transcripts to case directory
pub fn write_transcripts(stream: &TranscriptStreamV70, case_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(case_dir)?;

    // Human transcript
    let human_path = case_dir.join("human.log");
    let human_content = render_to_string(stream, TranscriptMode::Human);
    let mut human_file = BufWriter::new(File::create(&human_path)?);
    human_file.write_all(human_content.as_bytes())?;
    human_file.flush()?;

    // Debug transcript
    let debug_path = case_dir.join("debug.log");
    let debug_content = render_to_string(stream, TranscriptMode::Debug);
    let mut debug_file = BufWriter::new(File::create(&debug_path)?);
    debug_file.write_all(debug_content.as_bytes())?;
    debug_file.flush()?;

    Ok(())
}

// Helper function
fn reliability_desc(score: u8) -> &'static str {
    if score >= 80 {
        "good evidence coverage"
    } else if score >= 60 {
        "some evidence gaps"
    } else {
        "limited evidence"
    }
}
