//! Live request handling with real-time progress display (v0.0.253).
//!
//! Polls for progress events during request processing to show
//! internal IT department chatter (fly-on-wall experience).
//!
//! v0.0.237: Enhanced display format with conversational headers.
//! v0.0.238: Added streaming token support for word-by-word output.
//! v0.0.253: Enhanced specialist dialogue with role titles and visual polish.
//! v0.0.278: Enhanced Hollywood-style stage indicators and spinners.
//! v0.0.284: Added idle tips during wait times.
//! v0.0.285: Integrated telemetry-based health tips.

use anna_shared::health_tips::generate_telemetry_tips;
use anna_shared::idle_tips::{get_contextual_tips, TipColors, TipQueue};
use anna_shared::system_telemetry::TelemetryStore;
use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::roster;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::ui::colors;
use anna_shared::user_profile::UserProfile;
use anyhow::Result;
use std::collections::HashSet;
use std::io::{self, Write};
use std::time::Instant;
use tokio::time::{sleep, Duration};

use crate::client::AnnadClient;

/// Send request with live progress polling
/// Shows internal comms as they happen for fly-on-wall experience
/// v0.0.238: Added streaming token support for real-time output
pub async fn send_request_with_progress(prompt: &str) -> Result<ServiceDeskResult> {
    let mut client = AnnadClient::connect().await?;

    // Start the request in a background task
    let prompt_owned = prompt.to_string();
    let request_handle = tokio::spawn(async move {
        let mut c = AnnadClient::connect().await?;
        c.request(&prompt_owned).await
    });

    // Track which events we've already displayed
    let mut seen_events: HashSet<String> = HashSet::new();
    let mut last_event_count = 0;
    let mut streaming_state = StreamingState::default();

    // Poll for progress events while request is running
    // Use faster polling (50ms) for smoother streaming experience
    loop {
        // Check if request completed
        if request_handle.is_finished() {
            break;
        }

        // Try to get progress events
        if let Ok(events) = client.progress().await {
            // Only process new events
            if events.len() > last_event_count {
                for event in events.iter().skip(last_event_count) {
                    let event_key = format_event_key(event);
                    if !seen_events.contains(&event_key) {
                        display_progress_event(event, &mut streaming_state);
                        seen_events.insert(event_key);
                    }
                }
                last_event_count = events.len();
            }
        }

        // v0.0.284: Maybe show an idle tip if waiting long enough
        maybe_show_idle_tip(&mut streaming_state);

        // v0.0.304: Show checkpoint messages for long waits
        maybe_show_checkpoint(&mut streaming_state);

        // Use faster polling when streaming tokens for smoother output
        let poll_delay = if streaming_state.started_streaming {
            50 // 50ms for streaming
        } else {
            200 // 200ms when waiting
        };
        sleep(Duration::from_millis(poll_delay)).await;
    }

    // Get the final result
    request_handle.await?
}

/// Create a unique key for an event to avoid duplicates
fn format_event_key(event: &ProgressEvent) -> String {
    format!("{:?}-{}", event.stage, event.elapsed_ms)
}

/// Track streaming state for proper formatting
struct StreamingState {
    /// Whether we've started streaming tokens
    started_streaming: bool,
    /// Whether we're on a new line (for formatting)
    at_line_start: bool,
    /// v0.0.253: Whether we've shown the internal comms header
    shown_internal_header: bool,
    /// v0.0.278: Current stage for spinner display
    current_stage: Option<String>,
    /// v0.0.278: Whether we're showing a spinner line
    spinner_active: bool,
    /// v0.0.284: When request started (for tip timing)
    start_time: Instant,
    /// v0.0.284: Tip queue for idle display
    tip_queue: TipQueue,
    /// v0.0.284: Whether we've shown a tip this request
    shown_tip: bool,
    /// v0.0.304: Last checkpoint shown (for long wait feedback)
    last_checkpoint_secs: u64,
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

/// Display a progress event in fly-on-wall style
/// v0.0.237: Enhanced conversational format with better styling
/// v0.0.238: Added streaming token support
/// v0.0.253: Enhanced with role titles and internal comms header
/// v0.0.278: Hollywood-style stage transitions with animated spinners
fn display_progress_event(event: &ProgressEvent, state: &mut StreamingState) {
    // Check user preference for internal comms
    let profile = UserProfile::load();
    let show_internal = profile.preferences.show_internal_comms;

    match &event.event {
        ProgressEventType::Starting { timeout_secs: _ } => {
            // v0.0.278: Show stage transition with Hollywood flair
            let stage_name = match event.stage {
                anna_shared::progress::RequestStage::Translator => "classifying query",
                anna_shared::progress::RequestStage::Probes => "gathering system data",
                anna_shared::progress::RequestStage::Specialist => "consulting specialist",
                anna_shared::progress::RequestStage::Supervisor => "verifying answer",
            };
            // Clear any previous spinner
            if state.spinner_active {
                print!("\r{}\r", " ".repeat(60));
            }
            state.current_stage = Some(stage_name.to_string());
            state.spinner_active = true;
        }
        ProgressEventType::InternalComms { from, message } => {
            if !show_internal {
                return;
            }
            // Clear spinner if active
            if state.spinner_active {
                print!("\r{}\r", " ".repeat(60));
                state.spinner_active = false;
            }
            // If we were streaming, end the line first
            if state.started_streaming && !state.at_line_start {
                println!();
                state.at_line_start = true;
            }
            // v0.0.253: Show internal comms header on first message
            if !state.shown_internal_header {
                println!("{}--- internal comms ---{}", colors::DIM, colors::RESET);
                state.shown_internal_header = true;
            }
            // v0.0.253: Show internal comms with role titles from roster
            display_internal_comms(from, message);
            let _ = io::stdout().flush();
        }
        ProgressEventType::Generation { tokens } => {
            // Only show generation progress if not streaming tokens
            if !state.started_streaming {
                let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
                let frame = (*tokens / 5) % spinner.len();
                let stage_desc = state.current_stage.as_deref().unwrap_or("thinking");
                print!(
                    "\r  {}{}{} {}... {} tokens",
                    colors::CYAN,
                    spinner[frame],
                    colors::RESET,
                    stage_desc,
                    tokens
                );
                state.spinner_active = true;
                let _ = io::stdout().flush();
            }
        }
        ProgressEventType::StreamingToken { token, is_final } => {
            // Clear spinner line if this is first token
            if !state.started_streaming {
                print!("\r{}\r", " ".repeat(60));
                state.started_streaming = true;
                state.at_line_start = true;
                state.spinner_active = false;
            }
            // Print the token (word-by-word output)
            print!("{}", token);
            state.at_line_start = token.ends_with('\n');
            let _ = io::stdout().flush();

            // If final, ensure we end on a newline
            if *is_final && !state.at_line_start {
                println!();
                state.at_line_start = true;
            }
        }
        ProgressEventType::Complete => {
            // Clear generation line if needed (but not if we were streaming)
            if state.spinner_active && !state.started_streaming {
                print!("\r{}\r", " ".repeat(60));
                state.spinner_active = false;
                let _ = io::stdout().flush();
            }
        }
        _ => {
            // Other events are handled silently
        }
    }
}

/// v0.0.253: Display internal comms with role lookups
fn display_internal_comms(from: &str, message: &str) {
    if from == "Anna" {
        println!(
            "  {}Anna:{} {}",
            colors::OK,
            colors::RESET,
            message
        );
    } else {
        // Try to look up the person by display name
        if let Some(person) = roster::person_by_display_name(from) {
            println!(
                "  {}{} ({}):{} {}",
                colors::WARN,
                person.display_name,
                person.role_title,
                colors::RESET,
                message
            );
        } else {
            // Fallback if name not in roster
            println!(
                "  {}{}:{} {}",
                colors::WARN,
                from,
                colors::RESET,
                message
            );
        }
    }
}

/// v0.0.284: Maybe show an idle tip if we've been waiting long enough
/// Shows max one tip per request, after 3+ seconds of waiting
fn maybe_show_idle_tip(state: &mut StreamingState) {
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
fn maybe_show_checkpoint(state: &mut StreamingState) {
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

        println!(
            "{}[{}s]{} {}",
            colors::DIM,
            elapsed,
            colors::RESET,
            message
        );
        state.last_checkpoint_secs = expected_checkpoint;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::progress::RequestStage;

    #[test]
    fn test_event_key_uniqueness() {
        let event1 =
            ProgressEvent::internal_comms(RequestStage::Translator, "Anna", "Test message", 100);
        let event2 =
            ProgressEvent::internal_comms(RequestStage::Translator, "Anna", "Test message", 200);

        let key1 = format_event_key(&event1);
        let key2 = format_event_key(&event2);

        assert_ne!(
            key1, key2,
            "Different timestamps should produce different keys"
        );
    }
}
