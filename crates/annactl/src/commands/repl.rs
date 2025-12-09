//! REPL command handler (v0.0.205).

use anna_shared::clarify_v2::{ClarifyRequest, ClarifyResponse};
use anna_shared::status::LlmState;
use anna_shared::ui::colors;
use anyhow::Result;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::client::AnnadClient;
use crate::display::show_bootstrap_progress;
use crate::greeting;
use crate::live_request::send_request_with_progress;
use crate::transcript_render;

use super::feedback::{handle_feedback_request, handle_request_error};

// v0.0.97: Change management
use crate::change_commands::handle_proposed_change;

/// Pending clarification state for REPL mode
struct PendingClarification {
    request: ClarifyRequest,
    started_at: Instant,
}

/// Handle REPL mode - main interactive interface
pub async fn handle_repl() -> Result<()> {
    // Get daemon status for greeting
    let status = match AnnadClient::connect().await {
        Ok(mut client) => client.status().await.ok(),
        Err(_) => None,
    };

    // Theatre-style greeting with status awareness
    greeting::print_theatre_greeting(status.as_ref());

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

    loop {
        // Show different prompt if clarification pending
        if pending_clarification.is_some() {
            print!("{}[choice]> {}", colors::BOLD, colors::RESET);
        } else {
            // v0.0.168: Show username instead of generic "You"
            print!("{}{}: {}", colors::HEADER, username, colors::RESET);
        }
        io::stdout().flush()?;

        let mut input = String::new();
        let bytes_read = io::stdin().read_line(&mut input)?;

        // Handle Ctrl-D (EOF)
        if bytes_read == 0 {
            println!();
            println!("Goodbye! ;)");
            break;
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
                println!("{}Clarification timed out.{}", colors::WARN, colors::RESET);
                pending_clarification = None;
                continue;
            }

            let response = ClarifyResponse::parse(input, &pending.request);

            if response.cancelled {
                println!("Cancelled.");
                pending_clarification = None;
                continue;
            }

            let value = if let Some(key) = response.selected {
                pending.request.get_option(key).map(|o| o.value.clone())
            } else {
                response.free_text.clone()
            };

            if let Some(val) = value {
                println!("Selected: {}{}{}", colors::OK, val, colors::RESET);
                pending_clarification = None;
            } else {
                println!(
                    "{}Invalid selection. Try again or type 'cancel'.{}",
                    colors::WARN,
                    colors::RESET
                );
            }
            continue;
        }

        // Handle exit commands
        match input.to_lowercase().as_str() {
            "exit" | "quit" | "bye" | "q" | ":q" | ":wq" => {
                println!("Goodbye! ;)");
                break;
            }
            _ => {
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
                                        println!(
                                            "{}Error applying config.{}",
                                            colors::ERR,
                                            colors::RESET
                                        );
                                    } else if summary.applied > 0 {
                                        println!(
                                            "{}Done! Applied {} change{}.{}",
                                            colors::OK,
                                            summary.applied,
                                            if summary.applied == 1 { "" } else { "s" },
                                            colors::RESET
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{}Error:{} {}", colors::ERR, colors::RESET, e);
                                }
                            }
                        }

                        // Handle feedback request
                        if let Some(ref feedback_req) = result.feedback_request {
                            handle_feedback_request(feedback_req).await;
                        }

                        // Handle clarification request
                        if let Some(req) = &result.clarification_request {
                            println!();
                            println!("{}", req.format_menu());
                            pending_clarification = Some(PendingClarification {
                                request: req.clone(),
                                started_at: Instant::now(),
                            });
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
