//! Display formatting for streaming progress.

use super::buffer::StreamBuffer;

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
