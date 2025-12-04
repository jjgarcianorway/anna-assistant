//! Transcript Renderer v0.0.60 - 3-Tier Rendering System
//!
//! Renders transcript events for display based on mode:
//! - Human: Professional IT department dialogue, no internals
//! - Debug: Full internal details (tool names, evidence IDs, prompts)
//! - Test: Same as debug, for automated testing
//!
//! Also handles writing to log files in case directories.

use crate::human_labels::{tool_action, tool_evidence_desc, tool_working_msg, department_working_msg};
use crate::transcript_events::{EventActor, EventKind, TranscriptEvent, TranscriptEventStream, TranscriptMode};
use owo_colors::OwoColorize;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

// ============================================================================
// Render Output
// ============================================================================

/// A single rendered line for display
#[derive(Debug, Clone)]
pub struct RenderedLine {
    /// The rendered text (may include ANSI colors)
    pub text: String,
    /// Whether this is a progress/working indicator (can be overwritten)
    pub is_progress: bool,
}

impl RenderedLine {
    pub fn new(text: String) -> Self {
        Self { text, is_progress: false }
    }

    pub fn progress(text: String) -> Self {
        Self { text, is_progress: true }
    }
}

// ============================================================================
// Event Renderer
// ============================================================================

/// Renders a single event based on mode
pub fn render_event(event: &TranscriptEvent, mode: TranscriptMode) -> Option<RenderedLine> {
    match mode {
        TranscriptMode::Human => render_human(event),
        TranscriptMode::Debug | TranscriptMode::Test => render_debug(event),
    }
}

/// Render event for human mode (no internals)
fn render_human(event: &TranscriptEvent) -> Option<RenderedLine> {
    let actor_str = format!("[{}]", event.actor);
    let actor_colored = style_actor(&event.actor, &actor_str);

    match event.kind {
        EventKind::Phase => {
            // Phase separators
            let sep = format!("----- {} -----", event.summary.message);
            Some(RenderedLine::new(sep.dimmed().to_string()))
        }

        EventKind::Working => {
            // Progress indicators
            let msg = event.summary.progress.as_deref()
                .or(event.summary.message.as_str().into())
                .unwrap_or("Working...");
            let line = format!("{} {}", actor_colored, msg.dimmed());
            Some(RenderedLine::progress(line))
        }

        EventKind::ToolCall => {
            // Show human description of what's being checked
            if let Some(ref tool) = event.raw.tool_name {
                let action = tool_action(tool);
                let msg = format!("I'm {}.", action);
                Some(RenderedLine::new(format!("{} {}", actor_colored, msg)))
            } else if !event.summary.message.is_empty() {
                Some(RenderedLine::new(format!("{} {}", actor_colored, event.summary.message)))
            } else {
                None
            }
        }

        EventKind::ToolResult => {
            // Show human description of evidence found
            if let Some(ref tool) = event.raw.tool_name {
                let evidence = tool_evidence_desc(tool);
                let msg = format!("Evidence: {}", evidence);
                Some(RenderedLine::new(format!("{} {}", actor_colored, msg)))
            } else if let Some(ref desc) = event.summary.evidence_description {
                let msg = format!("Evidence: {}", desc);
                Some(RenderedLine::new(format!("{} {}", actor_colored, msg)))
            } else {
                None
            }
        }

        EventKind::Handoff => {
            // Department handoff
            if !event.summary.message.is_empty() {
                Some(RenderedLine::new(format!("{} {}", actor_colored, event.summary.message)))
            } else {
                None
            }
        }

        EventKind::Decision => {
            // Decision point
            if !event.summary.message.is_empty() {
                Some(RenderedLine::new(format!("{} {}", actor_colored, event.summary.message)))
            } else {
                None
            }
        }

        EventKind::Draft => {
            // Skip drafts in human mode
            None
        }

        EventKind::Final => {
            // Final answer
            if !event.summary.message.is_empty() {
                Some(RenderedLine::new(format!("{} {}", actor_colored, event.summary.message)))
            } else {
                None
            }
        }

        EventKind::Warning => {
            // Warnings (humanized, not raw)
            if !event.summary.message.is_empty() {
                let msg = format!("Note: {}", event.summary.message);
                Some(RenderedLine::new(format!("{} {}", actor_colored, msg.yellow())))
            } else {
                None
            }
        }

        EventKind::Confirmation => {
            // Confirmation requests
            if !event.summary.message.is_empty() {
                Some(RenderedLine::new(format!("{} {}", actor_colored, event.summary.message)))
            } else {
                None
            }
        }

        EventKind::Planning => {
            // Skip planning in human mode
            None
        }
    }
}

/// Render event for debug/test mode (full details)
fn render_debug(event: &TranscriptEvent) -> Option<RenderedLine> {
    let ts = event.ts.format("%H:%M:%S%.3f");
    let actor_str = format!("[{}]", event.actor);
    let kind_str = format!("[{}]", event.kind.as_str());

    let mut parts = vec![
        ts.to_string().dimmed().to_string(),
        kind_str.blue().to_string(),
        style_actor(&event.actor, &actor_str),
    ];

    // Add tool name if present
    if let Some(ref tool) = event.raw.tool_name {
        parts.push(format!("tool={}", tool).cyan().to_string());
    }

    // Add evidence ID if present
    if let Some(ref eid) = event.raw.evidence_id {
        parts.push(format!("[{}]", eid).green().to_string());
    }

    // Add duration if present
    if let Some(ms) = event.raw.duration_ms {
        parts.push(format!("({}ms)", ms).dimmed().to_string());
    }

    // Add message
    if !event.summary.message.is_empty() {
        parts.push(event.summary.message.clone());
    }

    // Add warnings
    for warning in &event.raw.warnings {
        parts.push(format!("WARN: {}", warning).yellow().to_string());
    }

    // Add error if present
    if let Some(ref err) = event.raw.error {
        parts.push(format!("ERROR: {}", err).red().to_string());
    }

    let line = parts.join(" ");
    Some(RenderedLine::new(line))
}

/// Style actor name with color
fn style_actor(actor: &EventActor, text: &str) -> String {
    match actor {
        EventActor::You => text.white().to_string(),
        EventActor::Anna => text.cyan().to_string(),
        EventActor::Translator => text.yellow().to_string(),
        EventActor::Junior => text.magenta().to_string(),
        EventActor::Annad => text.dimmed().to_string(),
        EventActor::Networking
        | EventActor::Storage
        | EventActor::Boot
        | EventActor::Audio
        | EventActor::Graphics
        | EventActor::Security
        | EventActor::Performance => text.green().to_string(),
    }
}

// ============================================================================
// Stream Renderer
// ============================================================================

/// Render all events in a stream
pub fn render_stream(stream: &TranscriptEventStream, mode: TranscriptMode) -> Vec<RenderedLine> {
    stream.events()
        .iter()
        .filter_map(|e| render_event(e, mode))
        .collect()
}

/// Render stream to string
pub fn render_stream_to_string(stream: &TranscriptEventStream, mode: TranscriptMode) -> String {
    render_stream(stream, mode)
        .into_iter()
        .map(|l| l.text)
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// File Writing
// ============================================================================

/// Write events to debug log file (full detail, always)
pub fn write_debug_log(stream: &TranscriptEventStream, case_dir: &Path) -> std::io::Result<()> {
    let path = case_dir.join("transcript.debug.log");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    for event in stream.events() {
        // Write JSON for each event
        let json = serde_json::to_string(&event).unwrap_or_default();
        writeln!(writer, "{}", json)?;
    }

    writer.flush()?;
    Ok(())
}

/// Write events to human log file (filtered, humanized)
pub fn write_human_log(stream: &TranscriptEventStream, case_dir: &Path) -> std::io::Result<()> {
    let path = case_dir.join("transcript.human.log");
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    for event in stream.events() {
        if let Some(line) = render_human(event) {
            // Strip ANSI codes for file output
            let clean = strip_ansi(&line.text);
            writeln!(writer, "{}", clean)?;
        }
    }

    writer.flush()?;
    Ok(())
}

/// Write both log files to a case directory
pub fn write_case_logs(stream: &TranscriptEventStream, case_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(case_dir)?;
    write_debug_log(stream, case_dir)?;
    write_human_log(stream, case_dir)?;
    Ok(())
}

/// Strip ANSI escape codes from string
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }

    result
}

// ============================================================================
// Helper Functions for Event Creation
// ============================================================================

/// Create a phase separator event
pub fn phase_event(phase_name: &str) -> TranscriptEvent {
    TranscriptEvent::new(EventActor::Anna, EventKind::Phase)
        .with_message(phase_name)
}

/// Create a working indicator event
pub fn working_event(actor: EventActor, message: &str) -> TranscriptEvent {
    TranscriptEvent::new(actor, EventKind::Working)
        .with_progress(message)
}

/// Create a tool call event
pub fn tool_call_event(actor: EventActor, tool_name: &str) -> TranscriptEvent {
    let working = tool_working_msg(tool_name);
    TranscriptEvent::new(actor, EventKind::ToolCall)
        .with_tool(tool_name)
        .with_progress(&working)
}

/// Create a tool result event
pub fn tool_result_event(
    actor: EventActor,
    tool_name: &str,
    evidence_id: &str,
    success: bool,
    duration_ms: u64,
) -> TranscriptEvent {
    let evidence_desc = tool_evidence_desc(tool_name);
    let mut event = TranscriptEvent::new(actor, EventKind::ToolResult)
        .with_tool(tool_name)
        .with_evidence_id(evidence_id)
        .with_evidence_desc(&evidence_desc)
        .with_duration(duration_ms);

    if !success {
        event.summary.message = "Could not retrieve this information.".to_string();
    }

    event
}

/// Create a handoff event
pub fn handoff_event(from: EventActor, to: EventActor, reason: &str) -> TranscriptEvent {
    let msg = format!("Assigning to [{}]: {}", to, reason);
    TranscriptEvent::new(from, EventKind::Handoff)
        .with_message(&msg)
}

/// Create a decision event
pub fn decision_event(actor: EventActor, decision: &str) -> TranscriptEvent {
    TranscriptEvent::new(actor, EventKind::Decision)
        .with_message(decision)
}

/// Create a final answer event
pub fn final_event(actor: EventActor, answer: &str) -> TranscriptEvent {
    TranscriptEvent::new(actor, EventKind::Final)
        .with_message(answer)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_mode_hides_internals() {
        let event = TranscriptEvent::new(EventActor::Annad, EventKind::ToolResult)
            .with_tool("hw_snapshot_summary")
            .with_evidence_id("E1")
            .with_evidence_desc("hardware inventory snapshot");

        let rendered = render_human(&event).unwrap();

        // Should show human description
        assert!(rendered.text.contains("hardware inventory"));

        // Should NOT show internal details
        assert!(!rendered.text.contains("hw_snapshot_summary"));
        assert!(!rendered.text.contains("[E1]"));
    }

    #[test]
    fn test_debug_mode_shows_internals() {
        let event = TranscriptEvent::new(EventActor::Annad, EventKind::ToolResult)
            .with_tool("hw_snapshot_summary")
            .with_evidence_id("E1")
            .with_duration(42);

        let rendered = render_debug(&event).unwrap();

        // Should show tool name
        assert!(rendered.text.contains("hw_snapshot_summary"));

        // Should show evidence ID
        assert!(rendered.text.contains("E1"));

        // Should show duration
        assert!(rendered.text.contains("42ms"));
    }

    #[test]
    fn test_phase_separator() {
        let event = phase_event("investigation");
        let rendered = render_human(&event).unwrap();

        assert!(rendered.text.contains("investigation"));
        assert!(rendered.text.contains("-----"));
    }

    #[test]
    fn test_strip_ansi() {
        let colored = "\x1b[36mtest\x1b[0m";
        assert_eq!(strip_ansi(colored), "test");
    }
}
