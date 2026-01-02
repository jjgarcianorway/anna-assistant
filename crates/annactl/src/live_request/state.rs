//! Streaming state management for live request handling.
//!
//! Tracks progress display state including streaming tokens,
//! spinner animation, idle tips, and checkpoint timing.

use anna_shared::health_tips::generate_telemetry_tips;
use anna_shared::idle_tips::{get_contextual_tips, TipQueue};
use anna_shared::system_telemetry::TelemetryStore;
use std::time::Instant;

/// Track streaming state for proper formatting
pub struct StreamingState {
    /// Whether we've started streaming tokens
    pub started_streaming: bool,
    /// Whether we're on a new line (for formatting)
    pub at_line_start: bool,
    /// v0.0.253: Whether we've shown the internal comms header
    pub shown_internal_header: bool,
    /// v0.0.278: Current stage for spinner display
    pub current_stage: Option<String>,
    /// v0.0.278: Whether we're showing a spinner line
    pub spinner_active: bool,
    /// v0.0.284: When request started (for tip timing)
    pub start_time: Instant,
    /// v0.0.284: Tip queue for idle display
    pub tip_queue: TipQueue,
    /// v0.0.284: Whether we've shown a tip this request
    pub shown_tip: bool,
    /// v0.0.304: Last checkpoint shown (for long wait feedback)
    pub last_checkpoint_secs: u64,
}

impl Default for StreamingState {
    fn default() -> Self {
        // Load contextual tips
        let mut tip_queue = TipQueue::new();
        for tip in get_contextual_tips() {
            tip_queue.push(tip);
        }

        // v0.0.285: Also add health tips from telemetry
        if let Some(telemetry) = TelemetryStore::load_if_exists() {
            for tip in generate_telemetry_tips(&telemetry) {
                tip_queue.push(tip);
            }
        }

        Self {
            started_streaming: false,
            at_line_start: true,
            shown_internal_header: false,
            current_stage: None,
            spinner_active: false,
            start_time: Instant::now(),
            tip_queue,
            shown_tip: false,
            last_checkpoint_secs: 0,
        }
    }
}
