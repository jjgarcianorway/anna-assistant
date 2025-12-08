//! Live request handling with real-time progress display (v0.0.148).
//!
//! Polls for progress events during request processing to show
//! internal IT department chatter (fly-on-wall experience).

use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::ui::colors;
use anyhow::Result;
use std::collections::HashSet;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};

use crate::client::AnnadClient;

/// Send request with live progress polling
/// Shows internal comms as they happen for fly-on-wall experience
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

    // Poll for progress events while request is running
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
                        display_progress_event(event);
                        seen_events.insert(event_key);
                    }
                }
                last_event_count = events.len();
            }
        }

        // Small delay before next poll
        sleep(Duration::from_millis(200)).await;
    }

    // Get the final result
    request_handle.await?
}

/// Create a unique key for an event to avoid duplicates
fn format_event_key(event: &ProgressEvent) -> String {
    format!("{:?}-{}", event.stage, event.elapsed_ms)
}

/// Display a progress event in fly-on-wall style
fn display_progress_event(event: &ProgressEvent) {
    match &event.event {
        ProgressEventType::InternalComms { from, message } => {
            // Show internal comms in cyan with staff name
            println!(
                "  {}[{}]{} {}",
                colors::CYAN,
                from,
                colors::RESET,
                message
            );
            let _ = io::stdout().flush();
        }
        ProgressEventType::Generation { tokens } => {
            // Show generation progress on same line
            print!(
                "\r  {}generating...{} {} tokens",
                colors::DIM,
                colors::RESET,
                tokens
            );
            let _ = io::stdout().flush();
        }
        ProgressEventType::Starting { .. } => {
            // Silently track stage starts
        }
        ProgressEventType::Complete => {
            // Clear generation line if needed
            print!("\r                                    \r");
            let _ = io::stdout().flush();
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
        let event1 = ProgressEvent::internal_comms(
            RequestStage::Translator,
            "Anna",
            "Test message",
            100,
        );
        let event2 = ProgressEvent::internal_comms(
            RequestStage::Translator,
            "Anna",
            "Test message",
            200,
        );

        let key1 = format_event_key(&event1);
        let key2 = format_event_key(&event2);

        assert_ne!(key1, key2, "Different timestamps should produce different keys");
    }
}
