//! Transcript System v0.0.72 - Unified Dual Mode Rendering
//!
//! Default: Human mode ("fly on the wall" IT department dialogue)
//! Debug: Full internal details for troubleshooting
//!
//! Both modes are generated from the SAME event stream, ensuring they
//! cannot diverge. Events contain both human-friendly and debug info.
//!
//! ## Human Mode (default)
//! - No tool names (hw_snapshot_summary, sw_snapshot_summary, etc.)
//! - No evidence IDs ([E1], [E2], etc.)
//! - No raw commands (journalctl, systemctl, etc.)
//! - No parse errors or fallback indicators
//! - Human-friendly equivalents for internal operations:
//!   - "Translator struggled to classify this; we used house rules."
//!   - "Hardware: Pulled inventory from the latest hardware snapshot."
//!
//! ## Debug Mode
//! - Canonical translator output with canonical lines
//! - Tool names, evidence IDs, timing
//! - Parse warnings, retries, fallbacks
//! - Raw evidence payloads
//! - Reliability scoring inputs and uncited claims
//!
//! ## Enable Debug Mode
//! - Config: `/etc/anna/config.toml` with `transcript_mode = "debug"`
//! - Env: `ANNA_UI_TRANSCRIPT_MODE=debug`
//! - Env: `ANNA_DEBUG_TRANSCRIPT=1` (shorthand for tests)
//!
//! ## Key Types
//! - `TranscriptEventV72`: Single event with both human and debug data
//! - `TranscriptStreamV72`: Event collector
//! - `EventDataV72`: Event variant (UserMessage, Evidence, etc.)
//! - `render_event_v72()`: Render single event based on mode
//! - `render_stream_v72()`: Render all events

pub mod events;
pub mod output;
pub mod render;
pub mod validation;

// Re-exports
pub use events::{
    EventDataV72, PerfBreakdownV72, RiskLevelV72, RoleV72, ToneV72, TranscriptEventV72,
    TranscriptStreamV72, WarningCategoryV72,
};
pub use output::{
    format_transcript_v72, print_transcript_v72, write_case_logs_v72, write_debug_log_v72,
    write_human_log_v72, write_line_v72,
};
pub use render::{render_event_v72, render_stream_v72, render_to_string_v72, RenderedLineV72};
pub use validation::{
    is_line_clean_for_human, strip_ansi, validate_debug_has_internals, validate_human_output,
    DebugValidation, FORBIDDEN_HUMAN_LITERALS, FORBIDDEN_HUMAN_PATTERNS,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript_events::TranscriptMode;

    /// Integration test: Complete workflow produces clean human output
    #[test]
    fn test_complete_workflow_human_clean() {
        let mut stream = TranscriptStreamV72::new().with_case_id("test-integration");

        // User message
        stream.push_data(EventDataV72::UserMessage {
            text: "What is my CPU?".to_string(),
        });

        // Phase
        stream.push_data(EventDataV72::Phase {
            name: "investigation".to_string(),
        });

        // Classification
        stream.push_data(EventDataV72::Classification {
            understood_human: "Looking up CPU information".to_string(),
            canonical_lines: Some(vec![
                "intent: question".to_string(),
                "target: hardware".to_string(),
            ]),
            parse_attempts: Some(1),
            fallback_used: false,
        });

        // Tool call
        stream.push_data(EventDataV72::ToolCall {
            tool_name: "hw_snapshot_summary".to_string(),
            action_human: "checking hardware inventory".to_string(),
            args: None,
        });

        // Evidence
        stream.push_data(EventDataV72::Evidence {
            evidence_id: "E1".to_string(),
            tool_name: "hw_snapshot_summary".to_string(),
            human_label: "hardware inventory".to_string(),
            summary_human: "Intel i9-14900HX processor".to_string(),
            summary_debug: Some("cpu_model=Intel i9-14900HX cores=24 threads=32".to_string()),
            duration_ms: 45,
        });

        // Staff message
        stream.push_data(EventDataV72::StaffMessage {
            role: RoleV72::Anna,
            tone: ToneV72::Neutral,
            content_human: "Your CPU is an Intel i9-14900HX with 24 cores.".to_string(),
            content_debug: Some("Final answer based on E1".to_string()),
        });

        // Reliability
        stream.push_data(EventDataV72::Reliability {
            score: 92,
            rationale_short: "direct hardware evidence".to_string(),
            rationale_full: Some("Score based on hw_snapshot_summary evidence [E1]".to_string()),
            uncited_claims: None,
        });

        // Render human mode
        let human_lines: Vec<String> = render_stream_v72(&stream, TranscriptMode::Human)
            .into_iter()
            .map(|l| l.text)
            .collect();

        // Validate human output is clean
        let violations = validate_human_output(&human_lines);
        assert!(
            violations.is_empty(),
            "Human mode has violations: {:?}",
            violations
        );

        // Should contain human-friendly content
        let human_text = human_lines.join("\n");
        assert!(human_text.contains("hardware inventory"));
        assert!(human_text.contains("Intel i9-14900HX"));
        assert!(human_text.contains("92%"));

        // Should NOT contain internal details
        assert!(!human_text.contains("[E1]"));
        assert!(!human_text.contains("hw_snapshot_summary"));
        assert!(!human_text.contains("parse_attempts"));
    }

    /// Integration test: Debug mode has expected internals
    #[test]
    fn test_complete_workflow_debug_has_internals() {
        let mut stream = TranscriptStreamV72::new();

        stream.push_data(EventDataV72::Evidence {
            evidence_id: "E1".to_string(),
            tool_name: "hw_snapshot_summary".to_string(),
            human_label: "hardware inventory".to_string(),
            summary_human: "Intel i9-14900HX".to_string(),
            summary_debug: Some("cpu=Intel(R) Core(TM) i9-14900HX".to_string()),
            duration_ms: 42,
        });

        stream.push_data(EventDataV72::Classification {
            understood_human: "Question about hardware".to_string(),
            canonical_lines: Some(vec!["intent: question".to_string()]),
            parse_attempts: Some(2),
            fallback_used: true,
        });

        // Render debug mode
        let debug_lines: Vec<String> = render_stream_v72(&stream, TranscriptMode::Debug)
            .into_iter()
            .map(|l| strip_ansi(&l.text))
            .collect();

        // Validate debug has internals
        let validation = validate_debug_has_internals(&debug_lines);
        assert!(validation.has_tool_names, "Debug should have tool names");
        assert!(
            validation.has_evidence_ids,
            "Debug should have evidence IDs"
        );
        assert!(validation.is_valid());

        // Should contain internal details
        let debug_text = debug_lines.join("\n");
        assert!(
            debug_text.contains("[E1]"),
            "Missing [E1] in: {}",
            debug_text
        );
        assert!(
            debug_text.contains("hw_snapshot_summary"),
            "Missing tool name"
        );
        assert!(debug_text.contains("42ms"), "Missing duration");
        assert!(
            debug_text.contains("parse_attempts"),
            "Missing parse_attempts"
        );
        assert!(debug_text.contains("fallback=true"), "Missing fallback");
    }

    /// Test: Same event stream produces both modes
    #[test]
    fn test_same_stream_dual_output() {
        let mut stream = TranscriptStreamV72::new();

        stream.push_data(EventDataV72::Evidence {
            evidence_id: "E1".to_string(),
            tool_name: "network_status".to_string(),
            human_label: "network snapshot".to_string(),
            summary_human: "WiFi connected".to_string(),
            summary_debug: Some("wlan0: UP carrier=true".to_string()),
            duration_ms: 30,
        });

        // Both modes from same stream
        let human = strip_ansi(&render_to_string_v72(&stream, TranscriptMode::Human));
        let debug = strip_ansi(&render_to_string_v72(&stream, TranscriptMode::Debug));

        // Human: clean
        assert!(
            human.contains("network snapshot"),
            "Human missing label: {}",
            human
        );
        assert!(!human.contains("[E1]"), "Human has evidence ID");
        assert!(!human.contains("network_status"), "Human has tool name");

        // Debug: full details
        assert!(
            debug.contains("[E1]"),
            "Debug missing evidence ID: {}",
            debug
        );
        assert!(debug.contains("network_status"), "Debug missing tool name");
        assert!(debug.contains("30ms"), "Debug missing duration");
    }

    /// Test: Confirmation phrase unchanged in both modes
    #[test]
    fn test_confirmation_phrase_unchanged() {
        let event = TranscriptEventV72::new(EventDataV72::Confirmation {
            change_description: "Install nginx package".to_string(),
            risk_level: RiskLevelV72::Medium,
            confirm_phrase: "I understand the risks".to_string(),
            rollback_summary: "pacman -R nginx".to_string(),
            rollback_details: Some(
                "Full rollback: pacman -R nginx && systemctl reset-failed".to_string(),
            ),
        });

        let stream = {
            let mut s = TranscriptStreamV72::new();
            s.push(event);
            s
        };

        let human = render_to_string_v72(&stream, TranscriptMode::Human);
        let debug = render_to_string_v72(&stream, TranscriptMode::Debug);

        // Both modes must have the exact confirmation phrase
        assert!(human.contains("I understand the risks"));
        assert!(debug.contains("I understand the risks"));
    }

    /// Test: Parse warnings hidden in human mode
    #[test]
    fn test_parse_warnings_human_hidden() {
        let mut stream = TranscriptStreamV72::new();

        stream.push_data(EventDataV72::Warning {
            message_human: "Request was unclear".to_string(),
            details_debug: Some("Parse error: Invalid Translator output format".to_string()),
            category: WarningCategoryV72::Parse,
        });

        let human = render_to_string_v72(&stream, TranscriptMode::Human);
        let debug = render_to_string_v72(&stream, TranscriptMode::Debug);

        // Human: no parse warning
        assert!(!human.contains("Parse error"));
        assert!(human.is_empty() || !human.contains("Warning")); // Parse warnings filtered

        // Debug: has parse warning
        assert!(debug.contains("Parse error"));
    }

    /// Test: Fallback humanized correctly
    #[test]
    fn test_fallback_humanized() {
        let mut stream = TranscriptStreamV72::new();

        stream.push_data(EventDataV72::Classification {
            understood_human: "Looking up information".to_string(),
            canonical_lines: None,
            parse_attempts: Some(3),
            fallback_used: true,
        });

        let human = render_to_string_v72(&stream, TranscriptMode::Human);
        let debug = render_to_string_v72(&stream, TranscriptMode::Debug);

        // Human: friendly message, no "deterministic fallback"
        assert!(human.contains("house rules"));
        assert!(!human.contains("deterministic"));
        assert!(!human.contains("fallback_used"));

        // Debug: technical details
        assert!(debug.contains("fallback=true"));
        assert!(debug.contains("parse_attempts=3"));
    }
}
