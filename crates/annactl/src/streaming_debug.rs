//! Streaming Debug Client v0.71.0
//!
//! Real-time consumption of debug events from annad.
//! Processes NDJSON stream line by line and displays events as they arrive.
//!
//! Activation:
//! - ANNA_DEBUG=1 environment variable
//! - config debug.live_view = true
//!
//! v0.71.0: Fixed version header, increased question display length

use anna_common::{DebugEvent, DebugStreamConfig, FinalAnswer};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::time::Instant;

const DAEMON_URL: &str = "http://127.0.0.1:7865";

/// Check if debug streaming is enabled
pub fn is_debug_enabled() -> bool {
    DebugStreamConfig::from_env().enabled
}

/// Stream answer with live debug events
///
/// Connects to the streaming endpoint and displays debug events in real-time.
/// Returns the final answer when stream completes.
pub async fn stream_answer_with_debug(question: &str) -> Result<Option<FinalAnswer>> {
    let client = reqwest::Client::new();
    let url = format!("{}/v1/answer/stream", DAEMON_URL);

    // Request payload
    #[derive(serde::Serialize)]
    struct StreamRequest {
        question: String,
        debug: bool,
    }

    let request = StreamRequest {
        question: question.to_string(),
        debug: true,
    };

    // Print header
    print_debug_header(question);

    let start = Instant::now();

    // Send streaming request
    let response = client
        .post(&url)
        .json(&request)
        .timeout(std::time::Duration::from_secs(180)) // 3 min timeout for long LLM ops
        .send()
        .await
        .context("Failed to connect to daemon streaming endpoint")?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Streaming request failed ({}): {}", status, text);
    }

    // Process NDJSON stream
    let mut bytes_stream = response.bytes_stream();
    let mut line_buffer = String::new();
    #[allow(unused_assignments)]
    let final_answer: Option<FinalAnswer> = None;
    let mut event_count = 0;

    while let Some(chunk_result) = bytes_stream.next().await {
        let chunk = chunk_result.context("Error reading stream chunk")?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        // Append to buffer and process complete lines
        line_buffer.push_str(&chunk_str);

        while let Some(newline_pos) = line_buffer.find('\n') {
            let line = line_buffer[..newline_pos].to_string();
            line_buffer = line_buffer[newline_pos + 1..].to_string();

            if line.trim().is_empty() {
                continue;
            }

            // Parse NDJSON line as DebugEvent
            match serde_json::from_str::<DebugEvent>(&line) {
                Ok(event) => {
                    event_count += 1;
                    display_debug_event(&event);

                    // Flush stdout to ensure real-time display
                    let _ = io::stdout().flush();
                }
                Err(e) => {
                    // Try parsing as error or other type
                    eprintln!(
                        "{}  Failed to parse event: {} (line: {})",
                        "[WARN]".yellow(),
                        e,
                        truncate_str(&line, 100)
                    );
                }
            }
        }
    }

    // Print footer with stats
    let elapsed = start.elapsed();
    print_debug_footer(event_count, elapsed.as_secs_f64());

    Ok(final_answer)
}

/// Display header for debug stream
/// v0.71.0: Use package version instead of hardcoded string
fn print_debug_header(question: &str) {
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let dim = "\x1b[2m";
    let cyan = "\x1b[38;2;0;255;255m";
    let magenta = "\x1b[38;2;255;0;255m";

    println!();
    println!(
        "{}{}============================================================================{}",
        bold, magenta, reset
    );
    // v0.71.0: Use CARGO_PKG_VERSION instead of hardcoded version
    println!(
        "{}{}  [DEBUG]  Live Debug Stream v{}{}",
        bold, magenta, env!("CARGO_PKG_VERSION"), reset
    );
    println!(
        "{}{}============================================================================{}",
        bold, magenta, reset
    );
    println!();
    // v0.71.0: Increased truncation from 70 to 512 chars to show full questions
    println!(
        "{}Question:{} {}",
        dim,
        reset,
        truncate_str(question, 512)
    );
    println!();
    println!(
        "{}{}----------------------------------------------------------------------------{}",
        dim, cyan, reset
    );
    println!();
}

/// Display a single debug event
fn display_debug_event(event: &DebugEvent) {
    print!("{}", event.format_terminal());
}

/// Display footer with stats
fn print_debug_footer(event_count: usize, duration_secs: f64) {
    let reset = "\x1b[0m";
    let bold = "\x1b[1m";
    let dim = "\x1b[2m";
    let magenta = "\x1b[38;2;255;0;255m";
    let green = "\x1b[38;2;0;255;0m";

    println!();
    println!(
        "{}{}----------------------------------------------------------------------------{}",
        dim, magenta, reset
    );
    println!(
        "{}{}  [DONE]  {} events in {:.2}s{}",
        bold, green, event_count, duration_secs, reset
    );
    println!(
        "{}{}============================================================================{}",
        bold, magenta, reset
    );
    println!();
}

/// Truncate string for display
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_enabled_check() {
        // Default should be false (no env var set)
        // This test depends on the env var NOT being set
        let config = DebugStreamConfig::default();
        assert!(!config.enabled);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }
}
