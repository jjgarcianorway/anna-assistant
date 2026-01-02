//! Main request handling with live progress polling.
//!
//! Sends requests to annad while polling for progress events
//! to display real-time internal IT department chatter.

use anna_shared::rpc::ServiceDeskResult;
use anyhow::Result;
use std::collections::HashSet;
use tokio::time::{sleep, Duration};

use crate::client::AnnadClient;

use super::display::display_progress_event;
use super::helpers::{format_event_key, maybe_show_checkpoint, maybe_show_idle_tip};
use super::state::StreamingState;

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
