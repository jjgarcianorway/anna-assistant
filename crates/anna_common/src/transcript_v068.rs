//! Transcript System v0.0.68 - Two-Layer Transcript Renderer
//!
//! Human Mode (default): Natural IT department conversation
//! - No evidence IDs ([E1]), no tool names, no raw commands
//! - Shows topic-based evidence summaries
//! - Readable, professional dialogue
//!
//! Debug Mode: Full fidelity for troubleshooting
//! - Exact prompts, tool names, evidence IDs
//! - Raw refs, parse warnings, timings
//! - Complete traceability

use crate::case_lifecycle::{ActionRisk, Department};
use crate::transcript_events::TranscriptMode;
use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

// ============================================================================
// v0.0.68 Event Types
// ============================================================================

/// Transcript event types for v0.0.68
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TranscriptEventV2 {
    /// User's message
    UserMessage {
        text: String,
    },

    /// Role-to-role message
    RoleMessage {
        from: TranscriptRole,
        to: TranscriptRole,
        text_human: String,
        text_debug: String,
    },

    /// Evidence gathered for a topic
    EvidenceTopicSummary {
        role: TranscriptRole,
        topic_title: String,
        summary_human: String,
        summary_debug: String,
        evidence_ids: Vec<String>,
        tool_names: Vec<String>,
        duration_ms: u64,
    },

    /// A decision point
    Decision {
        role: TranscriptRole,
        decision_human: String,
        decision_debug: String,
    },

    /// Action proposal presented to user
    ActionProposalPresented {
        proposal_id: String,
        proposal_title: String,
        risk: ActionRisk,
        rollback_human: String,
        confirmation_phrase: Option<String>,
        steps_human: Vec<String>,
        steps_debug: Vec<String>,
    },

    /// User's confirmation result
    ConfirmationResult {
        accepted: bool,
        phrase_entered: Option<String>,
    },

    /// Final answer to user
    FinalAnswer {
        text: String,
        reliability: u8,
        reliability_reason: String,
    },

    /// Performance metrics (debug only)
    Perf {
        total_ms: u64,
        llm_ms: Option<u64>,
        tool_ms: u64,
        tool_count: usize,
    },

    /// Working/progress indicator
    Working {
        role: TranscriptRole,
        message: String,
    },

    /// Phase separator
    Phase {
        name: String,
    },

    /// Warning (fallback used, uncertainty, etc.)
    Warning {
        role: TranscriptRole,
        message_human: String,
        message_debug: String,
    },
}

// ============================================================================
// Transcript Roles
// ============================================================================

/// Roles in the transcript
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptRole {
    You,
    ServiceDesk,
    Networking,
    Storage,
    Boot,
    Audio,
    Graphics,
    Security,
    Performance,
    // Internal roles - mostly invisible in human mode
    Translator,
    Junior,
    Annad,
}

impl TranscriptRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            TranscriptRole::You => "you",
            TranscriptRole::ServiceDesk => "service-desk",
            TranscriptRole::Networking => "networking",
            TranscriptRole::Storage => "storage",
            TranscriptRole::Boot => "boot",
            TranscriptRole::Audio => "audio",
            TranscriptRole::Graphics => "graphics",
            TranscriptRole::Security => "security",
            TranscriptRole::Performance => "performance",
            TranscriptRole::Translator => "translator",
            TranscriptRole::Junior => "junior",
            TranscriptRole::Annad => "annad",
        }
    }

    /// Whether this role should be visible in human mode
    pub fn visible_in_human_mode(&self) -> bool {
        !matches!(self, TranscriptRole::Translator | TranscriptRole::Junior | TranscriptRole::Annad)
    }

    /// From Department
    pub fn from_department(dept: Department) -> Self {
        match dept {
            Department::ServiceDesk => TranscriptRole::ServiceDesk,
            Department::Networking => TranscriptRole::Networking,
            Department::Storage => TranscriptRole::Storage,
            Department::Boot => TranscriptRole::Boot,
            Department::Audio => TranscriptRole::Audio,
            Department::Graphics => TranscriptRole::Graphics,
            Department::Security => TranscriptRole::Security,
            Department::Performance => TranscriptRole::Performance,
        }
    }
}

// ============================================================================
// Transcript Stream
// ============================================================================

/// Collects events during case execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptStreamV2 {
    pub case_id: Option<String>,
    pub events: Vec<TimestampedEvent>,
    pub evidence_coverage: Option<u8>,
    pub final_reliability: Option<u8>,
}

/// Event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEvent {
    pub ts: DateTime<Utc>,
    pub event: TranscriptEventV2,
}

impl TranscriptStreamV2 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_case_id(mut self, case_id: &str) -> Self {
        self.case_id = Some(case_id.to_string());
        self
    }

    pub fn push(&mut self, event: TranscriptEventV2) {
        self.events.push(TimestampedEvent {
            ts: Utc::now(),
            event,
        });
    }

    pub fn set_coverage(&mut self, coverage: u8) {
        self.evidence_coverage = Some(coverage);
    }

    pub fn set_reliability(&mut self, reliability: u8) {
        self.final_reliability = Some(reliability);
    }
}

// ============================================================================
// Rendering
// ============================================================================

/// Render transcript for human mode
pub fn render_human(stream: &TranscriptStreamV2) -> Vec<String> {
    let mut lines = Vec::new();

    for te in &stream.events {
        match &te.event {
            TranscriptEventV2::UserMessage { text } => {
                lines.push(format!("[you] {}", text));
            }

            TranscriptEventV2::RoleMessage { from, to: _, text_human, .. } => {
                if from.visible_in_human_mode() && !text_human.is_empty() {
                    lines.push(format!("[{}] {}", from.display_name(), text_human));
                }
            }

            TranscriptEventV2::EvidenceTopicSummary { role, topic_title, summary_human, .. } => {
                if role.visible_in_human_mode() {
                    lines.push(format!("[{}] {}: {}", role.display_name(), topic_title, summary_human));
                }
            }

            TranscriptEventV2::Decision { role, decision_human, .. } => {
                if role.visible_in_human_mode() && !decision_human.is_empty() {
                    lines.push(format!("[{}] {}", role.display_name(), decision_human));
                }
            }

            TranscriptEventV2::ActionProposalPresented {
                proposal_title,
                risk,
                rollback_human,
                confirmation_phrase,
                steps_human,
                ..
            } => {
                lines.push(format!("[service-desk] Proposed action: {}", proposal_title));
                lines.push(format!("  Risk level: {:?}", risk));
                for step in steps_human {
                    lines.push(format!("  - {}", step));
                }
                lines.push(format!("  Rollback: {}", rollback_human));
                if let Some(phrase) = confirmation_phrase {
                    lines.push(format!("  To proceed, type: \"{}\"", phrase));
                }
            }

            TranscriptEventV2::ConfirmationResult { accepted, .. } => {
                if *accepted {
                    lines.push("[you] Confirmed.".to_string());
                } else {
                    lines.push("[you] Declined.".to_string());
                }
            }

            TranscriptEventV2::FinalAnswer { text, reliability, reliability_reason } => {
                lines.push(format!("[service-desk] {}", text));
                lines.push(String::new());
                lines.push(format!("Reliability: {}% ({})", reliability, reliability_reason));
            }

            TranscriptEventV2::Perf { .. } => {
                // Skip in human mode
            }

            TranscriptEventV2::Working { role, message } => {
                if role.visible_in_human_mode() {
                    lines.push(format!("[{}] {}", role.display_name(), message));
                }
            }

            TranscriptEventV2::Phase { name } => {
                lines.push(format!("----- {} -----", name));
            }

            TranscriptEventV2::Warning { role, message_human, .. } => {
                // Show warnings from service-desk only in human mode
                if matches!(role, TranscriptRole::ServiceDesk) && !message_human.is_empty() {
                    lines.push(format!("[service-desk] Note: {}", message_human));
                }
            }
        }
    }

    lines
}

/// Render transcript for debug mode
pub fn render_debug(stream: &TranscriptStreamV2) -> Vec<String> {
    let mut lines = Vec::new();

    for te in &stream.events {
        let ts = te.ts.format("%H:%M:%S%.3f");

        match &te.event {
            TranscriptEventV2::UserMessage { text } => {
                lines.push(format!("{} [you] {}", ts, text));
            }

            TranscriptEventV2::RoleMessage { from, to, text_debug, .. } => {
                lines.push(format!("{} [{}] -> [{}]: {}", ts, from.display_name(), to.display_name(), text_debug));
            }

            TranscriptEventV2::EvidenceTopicSummary {
                role,
                topic_title,
                summary_debug,
                evidence_ids,
                tool_names,
                duration_ms,
                ..
            } => {
                lines.push(format!(
                    "{} [{}] Evidence: {} (tools: {}, ids: {}) ({}ms)",
                    ts,
                    role.display_name(),
                    topic_title,
                    tool_names.join(", "),
                    evidence_ids.join(", "),
                    duration_ms
                ));
                lines.push(format!("    {}", summary_debug));
            }

            TranscriptEventV2::Decision { role, decision_debug, .. } => {
                lines.push(format!("{} [{}] Decision: {}", ts, role.display_name(), decision_debug));
            }

            TranscriptEventV2::ActionProposalPresented {
                proposal_id,
                proposal_title,
                risk,
                steps_debug,
                ..
            } => {
                lines.push(format!("{} [action] {} - {} (risk: {:?})", ts, proposal_id, proposal_title, risk));
                for step in steps_debug {
                    lines.push(format!("    $ {}", step));
                }
            }

            TranscriptEventV2::ConfirmationResult { accepted, phrase_entered } => {
                lines.push(format!(
                    "{} [confirm] accepted={}, phrase={:?}",
                    ts, accepted, phrase_entered
                ));
            }

            TranscriptEventV2::FinalAnswer { text, reliability, reliability_reason } => {
                lines.push(format!("{} [final] reliability={}% ({})", ts, reliability, reliability_reason));
                lines.push(format!("    {}", text));
            }

            TranscriptEventV2::Perf { total_ms, llm_ms, tool_ms, tool_count } => {
                lines.push(format!(
                    "{} [perf] total={}ms, llm={:?}ms, tools={}ms ({}x)",
                    ts, total_ms, llm_ms, tool_ms, tool_count
                ));
            }

            TranscriptEventV2::Working { role, message } => {
                lines.push(format!("{} [{}] WORKING: {}", ts, role.display_name(), message));
            }

            TranscriptEventV2::Phase { name } => {
                lines.push(format!("{} === {} ===", ts, name.to_uppercase()));
            }

            TranscriptEventV2::Warning { role, message_debug, .. } => {
                lines.push(format!("{} [{}] WARN: {}", ts, role.display_name(), message_debug));
            }
        }
    }

    lines
}

/// Render based on mode
pub fn render(stream: &TranscriptStreamV2, mode: TranscriptMode) -> Vec<String> {
    match mode {
        TranscriptMode::Human => render_human(stream),
        TranscriptMode::Debug | TranscriptMode::Test => render_debug(stream),
    }
}

/// Render to string
pub fn render_to_string(stream: &TranscriptStreamV2, mode: TranscriptMode) -> String {
    render(stream, mode).join("\n")
}

// ============================================================================
// File I/O
// ============================================================================

/// Write both transcripts to case directory
pub fn write_transcripts(stream: &TranscriptStreamV2, case_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(case_dir)?;

    // Write human transcript
    let human_path = case_dir.join("human.log");
    let human_content = render_to_string(stream, TranscriptMode::Human);
    let mut human_file = BufWriter::new(File::create(&human_path)?);
    human_file.write_all(human_content.as_bytes())?;
    human_file.flush()?;

    // Write debug transcript
    let debug_path = case_dir.join("debug.log");
    let debug_content = render_to_string(stream, TranscriptMode::Debug);
    let mut debug_file = BufWriter::new(File::create(&debug_path)?);
    debug_file.write_all(debug_content.as_bytes())?;
    debug_file.flush()?;

    Ok(())
}

// ============================================================================
// Colored Terminal Output
// ============================================================================

/// Print human mode with colors
pub fn print_human_colored(stream: &TranscriptStreamV2) {
    for te in &stream.events {
        match &te.event {
            TranscriptEventV2::UserMessage { text } => {
                println!("{} {}", "[you]".white().bold(), text);
            }

            TranscriptEventV2::RoleMessage { from, text_human, .. } => {
                if from.visible_in_human_mode() && !text_human.is_empty() {
                    let actor = format!("[{}]", from.display_name());
                    println!("{} {}", style_role(from, &actor), text_human);
                }
            }

            TranscriptEventV2::EvidenceTopicSummary { role, topic_title, summary_human, .. } => {
                if role.visible_in_human_mode() {
                    let actor = format!("[{}]", role.display_name());
                    println!("{} {}: {}", style_role(role, &actor), topic_title.cyan(), summary_human);
                }
            }

            TranscriptEventV2::Decision { role, decision_human, .. } => {
                if role.visible_in_human_mode() && !decision_human.is_empty() {
                    let actor = format!("[{}]", role.display_name());
                    println!("{} {}", style_role(role, &actor), decision_human);
                }
            }

            TranscriptEventV2::FinalAnswer { text, reliability, reliability_reason } => {
                let actor = "[service-desk]".green();
                println!("{} {}", actor, text);
                println!();
                let rel_color = if *reliability >= 80 {
                    format!("{}%", reliability).green().to_string()
                } else if *reliability >= 60 {
                    format!("{}%", reliability).yellow().to_string()
                } else {
                    format!("{}%", reliability).red().to_string()
                };
                println!("Reliability: {} ({})", rel_color, reliability_reason);
            }

            TranscriptEventV2::Phase { name } => {
                println!("{}", format!("----- {} -----", name).dimmed());
            }

            TranscriptEventV2::Working { role, message } => {
                if role.visible_in_human_mode() {
                    let actor = format!("[{}]", role.display_name());
                    println!("{} {}", style_role(role, &actor), message.dimmed());
                }
            }

            TranscriptEventV2::Warning { role, message_human, .. } => {
                if matches!(role, TranscriptRole::ServiceDesk) && !message_human.is_empty() {
                    println!("{} {}", "[service-desk]".green(), format!("Note: {}", message_human).yellow());
                }
            }

            _ => {}
        }
    }
}

fn style_role(role: &TranscriptRole, text: &str) -> String {
    match role {
        TranscriptRole::You => text.white().bold().to_string(),
        TranscriptRole::ServiceDesk => text.green().to_string(),
        TranscriptRole::Networking => text.cyan().to_string(),
        TranscriptRole::Storage => text.blue().to_string(),
        TranscriptRole::Boot => text.magenta().to_string(),
        TranscriptRole::Audio => text.yellow().to_string(),
        TranscriptRole::Graphics => text.purple().to_string(),
        TranscriptRole::Security => text.red().to_string(),
        TranscriptRole::Performance => text.bright_cyan().to_string(),
        TranscriptRole::Translator => text.dimmed().to_string(),
        TranscriptRole::Junior => text.dimmed().to_string(),
        TranscriptRole::Annad => text.dimmed().to_string(),
    }
}

// ============================================================================
// Validation Helpers
// ============================================================================

/// Check if rendered human output contains any forbidden terms
pub fn validate_human_output(lines: &[String]) -> Vec<String> {
    let forbidden = [
        // Evidence IDs
        "[E1]", "[E2]", "[E3]", "[E4]", "[E5]", "[E6]", "[E7]", "[E8]", "[E9]",
        // Tool names
        "_summary", "_snapshot", "_probe", "_check", "_info",
        "journalctl", "systemctl", "nmcli", "ip ", "iw ", "btrfs ", "smartctl",
        "pacman ", "resolvectl", "hostnamectl",
    ];

    let mut violations = Vec::new();
    for line in lines {
        for term in &forbidden {
            if line.contains(term) {
                violations.push(format!("Found '{}' in: {}", term, line));
            }
        }
    }
    violations
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_mode_no_evidence_ids() {
        let mut stream = TranscriptStreamV2::new();
        stream.push(TranscriptEventV2::EvidenceTopicSummary {
            role: TranscriptRole::Networking,
            topic_title: "Link State".to_string(),
            summary_human: "WiFi is connected".to_string(),
            summary_debug: "wlan0: UP, carrier=true".to_string(),
            evidence_ids: vec!["E1".to_string()],
            tool_names: vec!["network_status".to_string()],
            duration_ms: 42,
        });

        let human = render_human(&stream);
        let human_str = human.join("\n");

        assert!(!human_str.contains("[E1]"), "Human mode should not contain evidence IDs");
        assert!(!human_str.contains("network_status"), "Human mode should not contain tool names");
        assert!(human_str.contains("WiFi is connected"));
    }

    #[test]
    fn test_debug_mode_has_evidence_ids() {
        let mut stream = TranscriptStreamV2::new();
        stream.push(TranscriptEventV2::EvidenceTopicSummary {
            role: TranscriptRole::Networking,
            topic_title: "Link State".to_string(),
            summary_human: "WiFi is connected".to_string(),
            summary_debug: "wlan0: UP, carrier=true".to_string(),
            evidence_ids: vec!["E1".to_string()],
            tool_names: vec!["network_status".to_string()],
            duration_ms: 42,
        });

        let debug = render_debug(&stream);
        let debug_str = debug.join("\n");

        assert!(debug_str.contains("E1"), "Debug mode should contain evidence IDs");
        assert!(debug_str.contains("network_status"), "Debug mode should contain tool names");
    }

    #[test]
    fn test_internal_roles_hidden_in_human_mode() {
        let mut stream = TranscriptStreamV2::new();
        stream.push(TranscriptEventV2::RoleMessage {
            from: TranscriptRole::Translator,
            to: TranscriptRole::ServiceDesk,
            text_human: "Internal parse".to_string(),
            text_debug: "INTENT: SYSTEM_QUERY, confidence=95".to_string(),
        });

        let human = render_human(&stream);
        let human_str = human.join("\n");

        assert!(!human_str.contains("translator"), "Translator should be hidden in human mode");
        assert!(!human_str.contains("Internal parse"));
    }

    #[test]
    fn test_reliability_line_human_mode() {
        let mut stream = TranscriptStreamV2::new();
        stream.push(TranscriptEventV2::FinalAnswer {
            text: "Your CPU is an AMD Ryzen 9.".to_string(),
            reliability: 84,
            reliability_reason: "good evidence coverage".to_string(),
        });

        let human = render_human(&stream);
        let human_str = human.join("\n");

        assert!(human_str.contains("84%"));
        assert!(human_str.contains("good evidence coverage"));
    }

    #[test]
    fn test_validate_human_output() {
        let good_lines = vec![
            "[networking] WiFi is connected".to_string(),
            "[service-desk] Here is your answer.".to_string(),
        ];
        assert!(validate_human_output(&good_lines).is_empty());

        let bad_lines = vec![
            "[networking] Evidence [E1]: network status".to_string(),
            "[annad] Running network_status_summary".to_string(),
        ];
        let violations = validate_human_output(&bad_lines);
        assert!(!violations.is_empty());
    }
}
