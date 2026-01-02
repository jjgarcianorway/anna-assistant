//! Tests for streaming functionality.

#[cfg(test)]
mod tests {
    use super::super::buffer::StreamBuffer;
    use super::super::display::StreamDisplay;
    use super::super::progress::ProgressFrame;
    use super::super::state::StreamState;

    #[test]
    fn test_stream_buffer_new() {
        let buffer = StreamBuffer::new();
        assert_eq!(buffer.state(), StreamState::Waiting);
        assert!(buffer.content().is_empty());
        assert_eq!(buffer.bytes_received(), 0);
    }

    #[test]
    fn test_stream_buffer_append() {
        let mut buffer = StreamBuffer::new();
        buffer.append("Hello ");
        buffer.append("World");

        assert_eq!(buffer.content(), "Hello World");
        assert_eq!(buffer.bytes_received(), 11);
        assert_eq!(buffer.state(), StreamState::Receiving);
    }

    #[test]
    fn test_stream_buffer_frame_detection() {
        let mut buffer = StreamBuffer::new();

        // Append start marker
        buffer.append("<<<ANNA_PROTO_V1>>>");
        assert_eq!(buffer.state(), StreamState::FrameStarted);

        // Append content
        buffer.append(r#"{"ok": true}"#);
        assert_eq!(buffer.state(), StreamState::FrameStarted);

        // Append end marker
        buffer.append("<<<END_ANNA_PROTO_V1>>>");
        assert_eq!(buffer.state(), StreamState::FrameComplete);
        assert!(buffer.has_complete_frame());
    }

    #[test]
    fn test_stream_buffer_complete_no_frame() {
        let mut buffer = StreamBuffer::new();
        buffer.append("Some raw output without markers");
        buffer.complete();

        assert_eq!(buffer.state(), StreamState::NoFrame);
    }

    #[test]
    fn test_stream_buffer_timeout() {
        let mut buffer = StreamBuffer::new();
        buffer.start();
        buffer.timeout();

        assert_eq!(buffer.state(), StreamState::TimedOut);
        assert!(buffer.state().is_complete());
    }

    #[test]
    fn test_stream_buffer_reset() {
        let mut buffer = StreamBuffer::new();
        buffer.append("content");
        buffer.reset();

        assert!(buffer.content().is_empty());
        assert_eq!(buffer.state(), StreamState::Waiting);
    }

    #[test]
    fn test_stream_state_status() {
        assert_eq!(
            StreamState::Waiting.status_text(),
            "Waiting for specialist..."
        );
        assert_eq!(
            StreamState::Receiving.status_text(),
            "Specialist is thinking..."
        );
        assert!(StreamState::Waiting.show_spinner());
        assert!(!StreamState::FrameComplete.show_spinner());
    }

    #[test]
    fn test_progress_frame() {
        let progress = ProgressFrame::progress(50, "Analyzing boot data");
        assert_eq!(progress.progress, Some(50));
        assert!(progress.message.as_ref().unwrap().contains("boot"));

        // Test max clamping
        let over = ProgressFrame::progress(150, "Test");
        assert_eq!(over.progress, Some(100));
    }

    #[test]
    fn test_stream_display() {
        let display = StreamDisplay::default();
        let mut buffer = StreamBuffer::new();
        buffer.start();
        buffer.append("test");

        let formatted = display.format_progress(&buffer);
        assert!(formatted.contains("thinking"));
    }
}
