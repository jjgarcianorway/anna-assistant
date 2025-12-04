//! Transcript System v0.0.70 - Dual Mode Renderer
//!
//! Human Mode (default): "Fly on the wall" IT department conversation
//! - No tool names (hw_snapshot_summary, sw_snapshot_summary, etc.)
//! - No evidence IDs ([E1], [E2], etc.)
//! - No raw commands (journalctl, systemctl, etc.)
//! - No parse errors or fallback indicators
//! - Uses topic abstractions: "hardware inventory", "network snapshot", etc.
//!
//! Debug Mode: Full transparency for developers/tests
//! - Canonical translator output (6-line format)
//! - Tool names, evidence IDs, timing
//! - Parse warnings, retries, fallbacks
//! - Raw evidence payloads
//! - Reliability scoring inputs

pub mod colored;
pub mod events;
pub mod render;
pub mod topics;
pub mod validation;

// Re-export main types for convenience
pub use colored::{print_colored, print_debug_colored, print_human_colored};
pub use events::{
    ActorV70, EventV70, TimestampedEventV70, TranscriptStatsV70, TranscriptStreamV70,
};
pub use render::{render, render_debug, render_human, render_to_string, write_transcripts};
pub use topics::{tool_to_evidence_topic, EvidenceTopicV70};
pub use validation::{validate_debug_has_internals, validate_human_output, FORBIDDEN_HUMAN};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_no_tool_names() {
        let mut stream = TranscriptStreamV70::new();
        stream.push(EventV70::Evidence {
            actor: ActorV70::Networking,
            topic: EvidenceTopicV70::NetworkSnapshot,
            summary_human: "WiFi connected".to_string(),
            summary_debug: "wlan0: UP, carrier=true".to_string(),
            tool_name: Some("network_status".to_string()),
            evidence_id: Some("E1".to_string()),
            duration_ms: 42,
        });

        let human = render_human(&stream);
        let human_str = human.join("\n");

        assert!(
            !human_str.contains("network_status"),
            "Human mode must not show tool names"
        );
        assert!(
            !human_str.contains("[E1]"),
            "Human mode must not show evidence IDs"
        );
        // v0.0.71: shorter label
        assert!(human_str.contains("network link and routing signals"));
    }

    #[test]
    fn test_debug_has_internals() {
        let mut stream = TranscriptStreamV70::new();
        stream.push(EventV70::Evidence {
            actor: ActorV70::Networking,
            topic: EvidenceTopicV70::NetworkSnapshot,
            summary_human: "WiFi connected".to_string(),
            summary_debug: "wlan0: UP, carrier=true".to_string(),
            tool_name: Some("network_status".to_string()),
            evidence_id: Some("E1".to_string()),
            duration_ms: 42,
        });

        let debug = render_debug(&stream);
        let debug_str = debug.join("\n");

        assert!(
            debug_str.contains("network_status"),
            "Debug mode must show tool names"
        );
        assert!(
            debug_str.contains("[E1]"),
            "Debug mode must show evidence IDs"
        );
    }

    #[test]
    fn test_internal_actors_hidden() {
        let mut stream = TranscriptStreamV70::new();
        stream.push(EventV70::StaffMessage {
            from: ActorV70::Translator,
            to: ActorV70::ServiceDesk,
            message_human: "Internal parse".to_string(),
            message_debug: "parsed intent: question".to_string(),
        });

        let human = render_human(&stream);
        let human_str = human.join("\n");

        // Translator messages should not appear in human mode
        assert!(
            !human_str.contains("translator"),
            "Internal actors should be hidden in human mode"
        );
    }

    #[test]
    fn test_parse_warning_debug_only() {
        let mut stream = TranscriptStreamV70::new();
        stream.push(EventV70::ParseWarning {
            subsystem: "translator".to_string(),
            details: "failed to parse action".to_string(),
            fallback_used: true,
        });

        let human = render_human(&stream);
        let debug = render_debug(&stream);

        assert!(
            human.is_empty() || !human.join("\n").contains("parse warning"),
            "Parse warnings should not appear in human mode"
        );
        assert!(
            debug.join("\n").contains("parse warning"),
            "Parse warnings should appear in debug mode"
        );
    }

    #[test]
    fn test_translator_canonical_debug_only() {
        let mut stream = TranscriptStreamV70::new();
        stream.push(EventV70::TranslatorCanonical {
            intent: "question".to_string(),
            target: "networking".to_string(),
            depth: "quick".to_string(),
            topics: vec!["wifi".to_string()],
            actions: vec![],
            safety: "read-only".to_string(),
        });

        let human = render_human(&stream);
        let debug = render_debug(&stream);

        assert!(
            human.is_empty() || !human.join("\n").contains("CANONICAL"),
            "Canonical output should not appear in human mode"
        );
        assert!(
            debug.join("\n").contains("CANONICAL"),
            "Canonical output should appear in debug mode"
        );
    }

    #[test]
    fn test_same_facts_both_modes() {
        let mut stream = TranscriptStreamV70::new();
        stream.push(EventV70::FinalAnswer {
            text: "WiFi is connected to network 'MyNetwork'".to_string(),
            reliability: 85,
            reliability_reason: "direct evidence".to_string(),
        });

        let human = render_human(&stream);
        let debug = render_debug(&stream);

        // Both should contain the answer text
        assert!(human.join("\n").contains("WiFi is connected"));
        assert!(debug.join("\n").contains("WiFi is connected"));

        // Both should show reliability
        assert!(human.join("\n").contains("85"));
        assert!(debug.join("\n").contains("85"));
    }
}
