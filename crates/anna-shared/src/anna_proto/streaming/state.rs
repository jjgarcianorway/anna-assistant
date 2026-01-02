//! Stream state tracking.

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
