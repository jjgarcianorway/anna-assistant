//! Helper functions for live request handling.
//!
//! Provides utilities for event deduplication, idle tips,
//! and checkpoint messages during long-running requests.

use anna_shared::idle_tips::TipColors;
use anna_shared::progress::ProgressEvent;
use anna_shared::ui::colors;
use std::io::{self, Write};

use super::state::StreamingState;

/// Create a unique key for an event to avoid duplicates
pub fn format_event_key(event: &ProgressEvent) -> String {
    format!("{:?}-{}", event.stage, event.elapsed_ms)
}

/// v0.0.284: Maybe show an idle tip if we've been waiting long enough
/// Shows max one tip per request, after 3+ seconds of waiting
pub fn maybe_show_idle_tip(state: &mut StreamingState) {
    // Only show tips if:
    // 1. We haven't shown one yet this request
    // 2. We're not actively streaming
    // 3. We've been waiting at least 3 seconds
    // 4. We have tips available
    if state.shown_tip || state.started_streaming {
        return;
    }

    let elapsed = state.start_time.elapsed();
    if elapsed.as_secs() < 3 {
        return;
    }

    if !state.tip_queue.has_tips() {
        return;
    }

    // Clear spinner if active
    if state.spinner_active {
        print!("\r{}\r", " ".repeat(60));
        state.spinner_active = false;
    }

    // Get and display a tip
    if let Some(tip) = state.tip_queue.pop() {
        let tip_colors = TipColors {
            dim: colors::DIM,
            reset: colors::RESET,
        };
        let formatted = anna_shared::idle_tips::format_tip(&tip, &tip_colors);
        print!("{}", formatted);
        let _ = io::stdout().flush();
        state.shown_tip = true;
    }
}

/// v0.0.304: Show progress checkpoint for long-running requests
/// Provides reassurance that Anna is still working
pub fn maybe_show_checkpoint(state: &mut StreamingState) {
    let elapsed = state.start_time.elapsed().as_secs();

    // Show checkpoint every 15 seconds (at 15s, 30s, 45s, etc.)
    let checkpoint_interval = 15;
    let expected_checkpoint = (elapsed / checkpoint_interval) * checkpoint_interval;

    // Only show if we've crossed a new checkpoint boundary and it's > 0
    if expected_checkpoint > state.last_checkpoint_secs && expected_checkpoint > 0 {
        // Clear spinner if active
        if state.spinner_active {
            print!("\r{}\r", " ".repeat(60));
            state.spinner_active = false;
        }

        let message = match expected_checkpoint {
            15 => "Still working on your request...",
            30 => "This is taking longer than usual. Anna is still analyzing...",
            45 => "Almost there... complex queries take more time.",
            _ => "Still processing... please wait.",
        };

        println!("{}[{}s]{} {}", colors::DIM, elapsed, colors::RESET, message);
        state.last_checkpoint_secs = expected_checkpoint;
    }
}
