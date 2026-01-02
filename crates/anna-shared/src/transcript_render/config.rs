//! Render configuration for transcript rendering.

use crate::transcript_segment::TranscriptMode;

/// Render configuration
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Display mode
    pub mode: TranscriptMode,
    /// Show internal comms section
    pub show_internal_comms: bool,
    /// Show tips
    pub show_tips: bool,
    /// Show probe details
    pub show_probes: bool,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Terminal width (for wrapping)
    pub width: usize,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            mode: TranscriptMode::Cinematic,
            show_internal_comms: true,
            show_tips: true,
            show_probes: true,
            show_timestamps: true,
            width: 80,
        }
    }
}

impl RenderConfig {
    pub fn cinematic() -> Self {
        Self::default()
    }

    pub fn debug() -> Self {
        Self {
            mode: TranscriptMode::Debug,
            ..Self::default()
        }
    }

    pub fn minimal() -> Self {
        Self {
            show_internal_comms: false,
            show_probes: false,
            show_tips: false,
            show_timestamps: false,
            ..Self::default()
        }
    }
}
