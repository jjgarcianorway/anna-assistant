//! Stream buffer for model output.

use super::state::StreamState;
use crate::anna_proto::framing::{has_incomplete_frame, PROTO_END, PROTO_START};

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

/// Get current timestamp in seconds.
fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
