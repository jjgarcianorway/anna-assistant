//! Live request handling with real-time progress display (v0.0.238).
//!
//! Polls for progress events during request processing to show
//! internal IT department chatter (fly-on-wall experience).
//!
//! v0.0.237: Enhanced display format with conversational headers.
//! v0.0.238: Added streaming token support for word-by-word output.

use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::ui::colors;
use anna_shared::user_profile::UserProfile;
use anyhow::Result;
use std::collections::HashSet;
use std::io::{self, Write};
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
}

impl Default for StreamingState {
    fn default() -> Self {
        Self {
            started_streaming: false,
            at_line_start: true,
        }
    }
}

/// Display a progress event in fly-on-wall style
/// v0.0.237: Enhanced conversational format with better styling
/// v0.0.238: Added streaming token support
fn display_progress_event(event: &ProgressEvent, state: &mut StreamingState) {
    // Check user preference for internal comms
    let profile = UserProfile::load();
    let show_internal = profile.preferences.show_internal_comms;

    match &event.event {
        ProgressEventType::InternalComms { from, message } => {
            if !show_internal {
                return;
            }
            // If we were streaming, end the line first
            if state.started_streaming && !state.at_line_start {
                println!();
                state.at_line_start = true;
            }
            // Show internal comms as dialogue
            if from == "Anna" {
                println!(
                    "  {}Anna:{} {}",
                    colors::CYAN,
                    colors::RESET,
                    message.as_str()
                );
            } else {
                println!(
                    "  {}{} ({}team{}):{} {}",
                    colors::HEADER,
                    from,
                    colors::DIM,
                    colors::HEADER,
                    colors::RESET,
                    message.as_str()
                );
            }
            let _ = io::stdout().flush();
        }
        ProgressEventType::Generation { tokens } => {
            // Only show generation progress if not streaming tokens
            if !state.started_streaming {
                let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
                let frame = (*tokens / 5) % spinner.len();
                print!(
                    "\r  {}{}{} thinking... {} tokens",
                    colors::CYAN,
                    spinner[frame],
                    colors::RESET,
                    tokens
                );
                let _ = io::stdout().flush();
            }
        }
        ProgressEventType::StreamingToken { token, is_final } => {
            // Clear spinner line if this is first token
            if !state.started_streaming {
                print!("\r                                        \r");
                state.started_streaming = true;
                state.at_line_start = true;
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
        ProgressEventType::Starting { .. } => {
            // Silently track stage starts
        }
        ProgressEventType::Complete => {
            // Clear generation line if needed (but not if we were streaming)
            if !state.started_streaming {
                print!("\r                                        \r");
                let _ = io::stdout().flush();
            }
        }
        _ => {
            // Other events are handled silently
        }
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
