//! Transcript Mode Integration Tests v0.0.60
//!
//! Tests that human mode hides internals and debug mode shows them.

#[cfg(test)]
mod tests {
    use crate::transcript_events::{
        EventActor, EventKind, TranscriptEvent, TranscriptEventStream, TranscriptMode,
    };
    use crate::transcript_renderer::{render_event, render_stream_to_string};

    /// Test that human mode hides tool names
    #[test]
    fn test_human_mode_hides_tool_names() {
        let event = TranscriptEvent::new(EventActor::Annad, EventKind::ToolResult)
            .with_tool("hw_snapshot_summary")
            .with_evidence_id("E1")
            .with_evidence_desc("hardware inventory snapshot")
            .with_message("Found hardware info");

        let rendered = render_event(&event, TranscriptMode::Human);
        assert!(rendered.is_some());

        let text = rendered.unwrap().text;

        // Human mode should NOT show tool names
        assert!(
            !text.contains("hw_snapshot_summary"),
            "Human mode should not show tool name 'hw_snapshot_summary', got: {}",
            text
        );

        // Human mode should NOT show evidence IDs
        assert!(
            !text.contains("[E1]"),
            "Human mode should not show evidence ID '[E1]', got: {}",
            text
        );

        // Human mode SHOULD show human-readable evidence description
        assert!(
            text.contains("hardware inventory"),
            "Human mode should show human description, got: {}",
            text
        );
    }

    /// Test that debug mode shows tool names and evidence IDs
    #[test]
    fn test_debug_mode_shows_internals() {
        let event = TranscriptEvent::new(EventActor::Annad, EventKind::ToolResult)
            .with_tool("hw_snapshot_summary")
            .with_evidence_id("E1")
            .with_duration(42);

        let rendered = render_event(&event, TranscriptMode::Debug);
        assert!(rendered.is_some());

        let text = rendered.unwrap().text;

        // Debug mode should show tool name
        assert!(
            text.contains("hw_snapshot_summary"),
            "Debug mode should show tool name 'hw_snapshot_summary', got: {}",
            text
        );

        // Debug mode should show evidence ID
        assert!(
            text.contains("E1"),
            "Debug mode should show evidence ID 'E1', got: {}",
            text
        );

        // Debug mode should show timing
        assert!(
            text.contains("42ms"),
            "Debug mode should show duration '42ms', got: {}",
            text
        );
    }

    /// Test that test mode behaves like debug mode
    #[test]
    fn test_test_mode_shows_internals() {
        let event = TranscriptEvent::new(EventActor::Anna, EventKind::ToolCall)
            .with_tool("sw_snapshot_summary");

        let rendered = render_event(&event, TranscriptMode::Test);
        assert!(rendered.is_some());

        let text = rendered.unwrap().text;

        // Test mode should show tool name (like debug)
        assert!(
            text.contains("sw_snapshot_summary"),
            "Test mode should show tool name, got: {}",
            text
        );
    }

    /// Test stream rendering in human mode
    #[test]
    fn test_stream_human_mode() {
        let mut stream = TranscriptEventStream::new();

        stream.push(
            TranscriptEvent::new(EventActor::Anna, EventKind::Phase).with_message("investigation"),
        );

        stream.push(
            TranscriptEvent::new(EventActor::Annad, EventKind::ToolCall)
                .with_tool("hw_snapshot_summary"),
        );

        stream.push(
            TranscriptEvent::new(EventActor::Annad, EventKind::ToolResult)
                .with_tool("hw_snapshot_summary")
                .with_evidence_id("E1"),
        );

        stream.push(
            TranscriptEvent::new(EventActor::Anna, EventKind::Final)
                .with_message("Your CPU is an AMD Ryzen."),
        );

        let output = render_stream_to_string(&stream, TranscriptMode::Human);

        // Should have [anna] actor
        assert!(output.contains("[anna]"), "Should contain [anna] actor");

        // Should NOT have tool names
        assert!(
            !output.contains("hw_snapshot_summary"),
            "Human stream should not show tool names"
        );

        // Should NOT have evidence IDs
        assert!(
            !output.contains("[E1]"),
            "Human stream should not show evidence IDs"
        );
    }

    /// Test stream rendering in debug mode
    #[test]
    fn test_stream_debug_mode() {
        let mut stream = TranscriptEventStream::new();

        stream.push(
            TranscriptEvent::new(EventActor::Annad, EventKind::ToolResult)
                .with_tool("hw_snapshot_summary")
                .with_evidence_id("E1")
                .with_duration(15),
        );

        let output = render_stream_to_string(&stream, TranscriptMode::Debug);

        // Debug mode should show tool names
        assert!(
            output.contains("hw_snapshot_summary"),
            "Debug stream should show tool names"
        );

        // Debug mode should show evidence IDs
        assert!(
            output.contains("E1"),
            "Debug stream should show evidence IDs"
        );

        // Debug mode should show timing
        assert!(output.contains("15ms"), "Debug stream should show timing");
    }

    /// Test that planning events are hidden in human mode
    #[test]
    fn test_planning_hidden_in_human_mode() {
        let event = TranscriptEvent::new(EventActor::Translator, EventKind::Planning)
            .with_message("Analyzing request intent");

        let rendered = render_event(&event, TranscriptMode::Human);

        // Planning should be hidden in human mode
        assert!(
            rendered.is_none(),
            "Planning events should be hidden in human mode"
        );
    }

    /// Test that planning events are shown in debug mode
    #[test]
    fn test_planning_shown_in_debug_mode() {
        let event = TranscriptEvent::new(EventActor::Translator, EventKind::Planning)
            .with_message("Analyzing request intent");

        let rendered = render_event(&event, TranscriptMode::Debug);

        // Planning should be shown in debug mode
        assert!(
            rendered.is_some(),
            "Planning events should be shown in debug mode"
        );
    }

    /// Test transcript mode from environment variable
    /// Note: Ignored because env vars cause race conditions in parallel test runs
    #[test]
    #[ignore = "Modifies env vars - causes race conditions in parallel test runs"]
    fn test_transcript_mode_from_env() {
        // Save original value
        let original = std::env::var("ANNA_UI_TRANSCRIPT_MODE").ok();

        // Test debug mode
        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "debug");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Debug);

        // Test test mode
        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "test");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Test);

        // Test human mode
        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "human");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Human);

        // Test case insensitivity
        std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", "DEBUG");
        assert_eq!(TranscriptMode::resolve(), TranscriptMode::Debug);

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("ANNA_UI_TRANSCRIPT_MODE", val);
        } else {
            std::env::remove_var("ANNA_UI_TRANSCRIPT_MODE");
        }
    }

    /// Test show_internals helper
    #[test]
    fn test_show_internals() {
        assert!(!TranscriptMode::Human.show_internals());
        assert!(TranscriptMode::Debug.show_internals());
        assert!(TranscriptMode::Test.show_internals());
    }
}
