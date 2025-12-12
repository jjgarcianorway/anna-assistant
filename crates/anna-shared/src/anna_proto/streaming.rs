//! Streaming-Safe Behavior (Part C) - v0.0.436.
//!
//! Buffer model output in memory instead of streaming raw tokens.
//! Show spinner + progress while model thinks.
//! Only render after decoding completes.

use super::framing::{has_incomplete_frame, PROTO_END, PROTO_START};
use serde::{Deserialize, Serialize};

/// State of the stream buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Waiting for model to start producing output.
    Waiting,
    /// Model is producing output (show spinner).
    Receiving,
    /// Frame start detected, waiting for end.
    FrameStarted,
    /// Frame complete, ready to decode.
    FrameComplete,
    /// Model finished without frame (will try recovery).
    NoFrame,
    /// Timeout occurred.
    TimedOut,
    /// Error occurred.
    Error,
}

impl StreamState {
    /// Human-readable status.
    pub fn status_text(&self) -> &'static str {
        match self {
            Self::Waiting => "Waiting for specialist...",
            Self::Receiving => "Specialist is thinking...",
            Self::FrameStarted => "Receiving response...",
            Self::FrameComplete => "Processing response...",
            Self::NoFrame => "Processing output...",
            Self::TimedOut => "Timed out",
            Self::Error => "Error occurred",
        }
    }

    /// Whether to show spinner.
    pub fn show_spinner(&self) -> bool {
        matches!(self, Self::Waiting | Self::Receiving | Self::FrameStarted)
    }

    /// Whether streaming is complete.
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            Self::FrameComplete | Self::NoFrame | Self::TimedOut | Self::Error
        )
    }
}

/// Buffer for model output with state tracking.
#[derive(Debug, Clone)]
pub struct StreamBuffer {
    /// Buffered output.
    buffer: String,
    /// Current state.
    state: StreamState,
    /// Bytes received.
    bytes_received: usize,
    /// Start time (for timeout tracking).
    started_at: Option<u64>,
    /// Last activity time.
    last_activity: Option<u64>,
}

impl StreamBuffer {
    /// Create a new stream buffer.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            state: StreamState::Waiting,
            bytes_received: 0,
            started_at: None,
            last_activity: None,
        }
    }

    /// Mark streaming as started.
    pub fn start(&mut self) {
        let now = timestamp_now();
        self.started_at = Some(now);
        self.last_activity = Some(now);
        self.state = StreamState::Waiting;
    }

    /// Append chunk to buffer.
    pub fn append(&mut self, chunk: &str) {
        if self.started_at.is_none() {
            self.start();
        }

        self.buffer.push_str(chunk);
        self.bytes_received += chunk.len();
        self.last_activity = Some(timestamp_now());

        // Update state based on content
        self.update_state();
    }

    /// Update state based on buffer content.
    fn update_state(&mut self) {
        if self.buffer.contains(PROTO_END) && self.buffer.contains(PROTO_START) {
            self.state = StreamState::FrameComplete;
        } else if self.buffer.contains(PROTO_START) {
            self.state = StreamState::FrameStarted;
        } else if !self.buffer.is_empty() {
            self.state = StreamState::Receiving;
        }
    }

    /// Mark as complete (model finished).
    pub fn complete(&mut self) {
        if self.state == StreamState::FrameComplete {
            // Already marked complete
            return;
        }

        // Check if we have a complete frame
        if self.buffer.contains(PROTO_START) && self.buffer.contains(PROTO_END) {
            self.state = StreamState::FrameComplete;
        } else {
            self.state = StreamState::NoFrame;
        }
    }

    /// Mark as timed out.
    pub fn timeout(&mut self) {
        self.state = StreamState::TimedOut;
    }

    /// Mark as error.
    pub fn error(&mut self) {
        self.state = StreamState::Error;
    }

    /// Get current state.
    pub fn state(&self) -> StreamState {
        self.state
    }

    /// Get buffered content.
    pub fn content(&self) -> &str {
        &self.buffer
    }

    /// Get bytes received.
    pub fn bytes_received(&self) -> usize {
        self.bytes_received
    }

    /// Get elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        match self.started_at {
            Some(start) => {
                let now = timestamp_now();
                (now - start) * 1000 // Convert to ms (assuming timestamp is in seconds)
            }
            None => 0,
        }
    }

    /// Get time since last activity in milliseconds.
    pub fn idle_ms(&self) -> u64 {
        match self.last_activity {
            Some(last) => {
                let now = timestamp_now();
                (now - last) * 1000
            }
            None => 0,
        }
    }

    /// Check if buffer has complete frame.
    pub fn has_complete_frame(&self) -> bool {
        self.state == StreamState::FrameComplete
    }

    /// Check if buffer has incomplete frame (started but not finished).
    pub fn has_incomplete_frame(&self) -> bool {
        has_incomplete_frame(&self.buffer)
    }

    /// Clear buffer and reset state.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.state = StreamState::Waiting;
        self.bytes_received = 0;
        self.started_at = None;
        self.last_activity = None;
    }

    /// Take ownership of buffer content.
    pub fn take_content(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }
}

impl Default for StreamBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress frame (optional streaming support).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressFrame {
    /// Frame type.
    #[serde(rename = "type")]
    pub frame_type: ProgressType,
    /// Progress percentage (0-100).
    pub progress: Option<u8>,
    /// Status message.
    pub message: Option<String>,
}

impl ProgressFrame {
    /// Create a progress update.
    pub fn progress(percent: u8, message: &str) -> Self {
        Self {
            frame_type: ProgressType::Progress,
            progress: Some(percent.min(100)),
            message: Some(message.to_string()),
        }
    }

    /// Create a thinking status.
    pub fn thinking(message: &str) -> Self {
        Self {
            frame_type: ProgressType::Thinking,
            progress: None,
            message: Some(message.to_string()),
        }
    }
}

/// Progress frame type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressType {
    /// Progress update.
    Progress,
    /// Model is thinking.
    Thinking,
    /// Result is ready.
    Result,
}

/// Display configuration for streaming.
#[derive(Debug, Clone)]
pub struct StreamDisplay {
    /// Show spinner animation.
    pub show_spinner: bool,
    /// Show bytes received counter.
    pub show_bytes: bool,
    /// Show elapsed time.
    pub show_time: bool,
    /// Custom status message.
    pub status_message: Option<String>,
}

impl Default for StreamDisplay {
    fn default() -> Self {
        Self {
            show_spinner: true,
            show_bytes: false,
            show_time: true,
            status_message: None,
        }
    }
}

impl StreamDisplay {
    /// Format progress line.
    pub fn format_progress(&self, buffer: &StreamBuffer) -> String {
        let mut parts = Vec::new();

        if let Some(msg) = &self.status_message {
            parts.push(msg.clone());
        } else {
            parts.push(buffer.state().status_text().to_string());
        }

        if self.show_bytes && buffer.bytes_received() > 0 {
            parts.push(format!("({} bytes)", buffer.bytes_received()));
        }

        if self.show_time && buffer.elapsed_ms() > 0 {
            let secs = buffer.elapsed_ms() / 1000;
            parts.push(format!(
                "[{:.1}s]",
                secs as f64 + (buffer.elapsed_ms() % 1000) as f64 / 1000.0
            ));
        }

        parts.join(" ")
    }
}

/// Get current timestamp in seconds.
fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

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
