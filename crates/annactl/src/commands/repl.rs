//! REPL command handler (v0.0.343).
//!
//! v0.0.237: Added config command handling for natural language settings.
//! v0.0.240: Added idle-time tips during user inactivity.
//! v0.0.343: Use centralized UI helpers for consistency.

use anna_shared::clarification_learning::{
    record_clarification_learning, ClarificationLearningStore,
};
use anna_shared::clarify_v2::{ClarifyRequest, ClarifyResponse};
use anna_shared::config_parser::is_config_request;
use anna_shared::idle_tips::{format_tip, get_contextual_tips, TipColors, TipQueue};
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, print_hint, print_label, print_warn};
use anyhow::Result;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use tokio::io::AsyncBufReadExt;

use crate::client::AnnadClient;
use crate::display::show_bootstrap_progress;
use crate::greeting;
use crate::live_request::send_request_with_progress;
use crate::transcript_render;

use super::config::{show_config_status, try_handle_config, ConfigResult};
use super::feedback::{handle_feedback_request, handle_request_error};

// v0.0.97: Change management
use crate::change_commands::handle_proposed_change;

/// Pending clarification state for REPL mode
struct PendingClarification {
    request: ClarifyRequest,
    started_at: Instant,
}

/// v0.0.240: Idle timeout for showing tips (30 seconds)
const IDLE_TIP_TIMEOUT: Duration = Duration::from_secs(30);

/// v0.0.240: Max tips per session to avoid annoyance
const MAX_TIPS_PER_SESSION: u32 = 3;

/// Handle REPL mode - main interactive interface
pub async fn handle_repl() -> Result<()> {
    // v0.0.818: Get daemon status with short timeout to prevent slow startup
    let status = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        async {
            match AnnadClient::connect().await {
                Ok(mut client) => client.status().await.ok(),
                Err(_) => None,
            }
        }
    ).await.unwrap_or(None);

    // Theatre-style greeting with status awareness
    greeting::print_theatre_greeting(status.as_ref()).await;

    // Check if LLM needs bootstrap
    if let Some(ref st) = status {
        if st.llm.state != LlmState::Ready {
            show_bootstrap_progress().await?;
        }
    }

    // v0.0.168: Get username for personalized prompt
    let username = std::env::var("USER").unwrap_or_else(|_| "you".to_string());

    // Track pending clarification (local state only)
    let mut pending_clarification: Option<PendingClarification> = None;

    // v0.0.240: Initialize tip queue with contextual tips
    let mut tip_queue = TipQueue::new();
    for tip in get_contextual_tips() {
        tip_queue.push(tip);
    }

    // v0.0.240: Use async stdin for timeout-based idle detection
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);

    loop {
        // Show different prompt if clarification pending
        if pending_clarification.is_some() {
            print!("{}[choice]> {}", colors::BOLD, colors::RESET);
        } else {
            // v0.0.168: Show username instead of generic "You"
            print!("{}{}: {}", colors::HEADER, username, colors::RESET);
        }
        io::stdout().flush()?;

        // v0.0.240: Read input with idle timeout for tips
        let mut input = String::new();
        let read_result = read_with_idle_tips(&mut reader, &mut input, &mut tip_queue).await;

        match read_result {
            ReadResult::Input(0) => {
                // EOF (Ctrl-D)
                println!();
                print_hint("Goodbye! ;)");
                break;
            }
            ReadResult::Input(_) => {
                // Normal input, continue processing
            }
            ReadResult::Error(e) => {
                print_label("error", &format!("Input error: {}", e), colors::ERR);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle pending clarification first
        if let Some(ref pending) = pending_clarification {
            let elapsed = pending.started_at.elapsed();
            let ttl = Duration::from_secs(pending.request.ttl_seconds as u64);

            if elapsed > ttl && pending.request.ttl_seconds > 0 {
                print_warn("Clarification timed out");
                pending_clarification = None;
                continue;
            }

            let response = ClarifyResponse::parse(input, &pending.request);

            if response.cancelled {
                print_hint("Cancelled.");
                pending_clarification = None;
                continue;
            }

            let value = if let Some(key) = response.selected {
                pending.request.get_option(key).map(|o| o.value.clone())
            } else {
                response.free_text.clone()
            };

            if let Some(val) = value {
                print_label("selected", &val, colors::OK);
                // v0.0.401: Learn from clarification response
                record_clarification_learning(&pending.request, &val);
                pending_clarification = None;
            } else {
                print_warn("Invalid selection. Try again or type 'cancel'");
            }
            continue;
        }

        // Handle exit commands
        match input.to_lowercase().as_str() {
            "exit" | "quit" | "bye" | "q" | ":q" | ":wq" => {
                print_hint("Goodbye! ;)");
                break;
            }
            // v0.0.237: Show config status
            "config" | "settings" | "preferences" | "my settings" => {
                show_config_status();
                println!();
                continue;
            }
            _ => {
                // v0.0.237: Try config commands first (fast path, no daemon needed)
                if is_config_request(input) {
                    if let ConfigResult::Handled = try_handle_config(input) {
                        println!();
                        continue;
                    }
                }

                // Check LLM ready
                if let Ok(mut client) = AnnadClient::connect().await {
                    if let Ok(status) = client.status().await {
                        if status.llm.state != LlmState::Ready {
                            show_bootstrap_progress().await?;
                        }
                    }
                }

                // v0.0.148: Use live progress display for fly-on-wall experience
                println!();
                match send_request_with_progress(input).await {
                    Ok(result) => {
                        println!();
                        transcript_render::render(&result);

                        // Handle proposed config changes
                        let proposed: Vec<_> = if !result.proposed_changes.is_empty() {
                            result.proposed_changes.clone()
                        } else {
                            result.proposed_change.iter().cloned().collect()
                        };
                        if !proposed.is_empty() {
                            match handle_proposed_change(&proposed).await {
                                Ok(summary) => {
                                    if summary.failed {
                                        print_label(
                                            "config",
                                            "Error applying changes",
                                            colors::ERR,
                                        );
                                    } else if summary.applied > 0 {
                                        print_label(
                                            "config",
                                            &format!(
                                                "Applied {} change{}",
                                                summary.applied,
                                                if summary.applied == 1 { "" } else { "s" }
                                            ),
                                            colors::OK,
                                        );
                                    }
                                }
                                Err(e) => {
                                    print_label("error", &format!("{}", e), colors::ERR);
                                }
                            }
                        }

                        // Handle feedback request
                        if let Some(ref feedback_req) = result.feedback_request {
                            handle_feedback_request(feedback_req, &result.answer).await;
                        }

                        // Handle clarification request
                        if let Some(req) = &result.clarification_request {
                            // v0.0.401: Check if we can auto-answer from learned preferences
                            let learning_store = ClarificationLearningStore::load();
                            if let Some(auto_answer) = learning_store.can_auto_answer(req) {
                                print_hint(&format!("Using learned preference: {}", auto_answer));
                                // Reinforce the learning
                                record_clarification_learning(req, auto_answer);
                            } else {
                                println!();
                                println!("{}", req.format_menu());
                                pending_clarification = Some(PendingClarification {
                                    request: req.clone(),
                                    started_at: Instant::now(),
                                });
                            }
                        }

                        println!();
                    }
                    Err(e) => {
                        handle_request_error(&e).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// v0.0.240: Result of reading input with idle detection
enum ReadResult {
    /// Successfully read input (contains bytes read, 0 = EOF)
    Input(usize),
    /// Error reading input
    Error(std::io::Error),
}

/// v0.0.240: Read a line with idle timeout for tips
async fn read_with_idle_tips(
    reader: &mut tokio::io::BufReader<tokio::io::Stdin>,
    buffer: &mut String,
    tip_queue: &mut TipQueue,
) -> ReadResult {
    let tip_colors = TipColors {
        dim: colors::DIM,
        reset: colors::RESET,
    };

    loop {
        // Check if we should show tips (respect session limit)
        let should_show_tips =
            tip_queue.has_tips() && tip_queue.shown_count() < MAX_TIPS_PER_SESSION;

        if should_show_tips {
            // Use select with timeout to detect idle
            tokio::select! {
                result = reader.read_line(buffer) => {
                    return match result {
                        Ok(n) => ReadResult::Input(n),
                        Err(e) => ReadResult::Error(e),
                    };
                }
                _ = tokio::time::sleep(IDLE_TIP_TIMEOUT) => {
                    // User is idle, show a tip
                    if let Some(tip) = tip_queue.pop() {
                        // Move to new line, show tip, re-show prompt
                        println!();
                        print!("{}", format_tip(&tip, &tip_colors));
                        io::stdout().flush().ok();

                        // Re-display the prompt
                        let username = std::env::var("USER").unwrap_or_else(|_| "you".to_string());
                        print!("{}{}: {}", colors::HEADER, username, colors::RESET);
                        io::stdout().flush().ok();
                    }
                    // Continue waiting for input
                }
            }
        } else {
            // No tips to show, just wait for input normally
            return match reader.read_line(buffer).await {
                Ok(n) => ReadResult::Input(n),
                Err(e) => ReadResult::Error(e),
            };
        }
    }
}
