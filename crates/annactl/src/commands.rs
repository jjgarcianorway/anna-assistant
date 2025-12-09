//! Command handlers for annactl.
//! v0.0.144: Simplified - removed unnecessary flags, natural language for everything.

use anna_shared::clarify_v2::{ClarifyRequest, ClarifyResponse};
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, symbols};
use anna_shared::version::VERSION;
use anyhow::Result;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::client::AnnadClient;
use crate::display::{print_stats_display, print_status_display, show_bootstrap_progress};
use crate::greeting;
use crate::live_request::send_request_with_progress;
use crate::transcript_render;

/// Pending clarification state for REPL mode
struct PendingClarification {
    request: ClarifyRequest,
    started_at: Instant,
}

/// Handle status command - shows Anna's health, config, and system info
pub async fn handle_status() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;
    let snapshot = client.status_snapshot().await.ok();
    let daemon_info = client.get_daemon_info().await.ok();

    print_status_display(&status, snapshot.as_ref(), daemon_info.as_ref());
    Ok(())
}

/// Handle stats command (v0.0.27)
pub async fn handle_stats() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let stats = client.stats().await?;
    print_stats_display(&stats);
    Ok(())
}

/// Core request function (v0.0.148: kept for fallback, use send_request_with_progress)
#[allow(dead_code)]
async fn send_request(prompt: &str) -> Result<ServiceDeskResult> {
    let mut client = AnnadClient::connect().await?;
    client.request(prompt).await
}

/// Handle a single request (one-shot mode)
pub async fn handle_request(prompt: &str) -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;

    if status.llm.state != LlmState::Ready {
        drop(client);
        show_bootstrap_progress().await?;
    }

    // v0.0.148: Use live progress display for fly-on-wall experience
    println!();
    let result = send_request_with_progress(prompt).await?;
    println!();

    // Render the result
    transcript_render::render(&result);

    // v0.0.96: Handle proposed config changes
    let proposed: Vec<_> = if !result.proposed_changes.is_empty() {
        result.proposed_changes.clone()
    } else {
        result.proposed_change.iter().cloned().collect()
    };
    if !proposed.is_empty() {
        let summary = handle_proposed_change(&proposed).await?;
        if summary.failed {
            println!(
                "{}Anna: config application hit errors; review details above.{}",
                colors::ERR,
                colors::RESET
            );
        } else if summary.applied > 0 {
            println!(
                "{}Anna: config applied ({} step{}, {} noop).{}",
                colors::OK,
                summary.applied,
                if summary.applied == 1 { "" } else { "s" },
                summary.noop,
                colors::RESET
            );
        } else {
            println!(
                "{}Anna: nothing to change; already configured.{}",
                colors::DIM,
                colors::RESET
            );
        }
    }

    // v0.0.103: Handle feedback request from Anna
    if let Some(ref feedback_req) = result.feedback_request {
        handle_feedback_request(feedback_req).await;
    }

    Ok(())
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

// v0.0.97: Change management (handle_proposed_change still needed for config changes)
use crate::change_commands::handle_proposed_change;

/// v0.0.103: Handle feedback request from Anna
/// When Anna is uncertain about a recipe answer, she asks the user for feedback
async fn handle_feedback_request(feedback_req: &anna_shared::recipe_feedback::FeedbackRequest) {
    use anna_shared::recipe_feedback::{
        apply_feedback, log_feedback, FeedbackRating, RecipeFeedback,
    };

    println!();
    println!(
        "{}[feedback]{} {}",
        colors::DIM,
        colors::RESET,
        feedback_req.question
    );
    print!("> ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return;
    }

    let input = input.trim().to_lowercase();
    let rating = match input.as_str() {
        "y" | "yes" | "helpful" | "good" => Some(FeedbackRating::Helpful),
        "n" | "no" | "not helpful" | "bad" => Some(FeedbackRating::NotHelpful),
        "partial" | "meh" | "ok" => Some(FeedbackRating::Partial),
        "" | "skip" => None, // User skipped feedback
        _ => {
            println!(
                "{}Skipping feedback (unrecognized input){}",
                colors::DIM,
                colors::RESET
            );
            None
        }
    };

    if let Some(r) = rating {
        let feedback = RecipeFeedback::new(&feedback_req.recipe_id, r);
        log_feedback(&feedback);

        if let Some(result) = apply_feedback(&feedback) {
            println!(
                "{}Thanks!{} Recipe confidence adjusted ({} → {})",
                colors::OK,
                colors::RESET,
                result.previous_score,
                result.new_score
            );
        } else {
            println!("{}Thanks for the feedback!{}", colors::OK, colors::RESET);
        }
    }
}

/// Handle request error with recovery
async fn handle_request_error(e: &anyhow::Error) -> Result<()> {
    let err_str = e.to_string();
    if err_str.contains("LLM") || err_str.contains("connect") {
        println!();
        println!(
            "{}Connection issue.{} Restarting...",
            colors::WARN,
            colors::RESET
        );
        show_bootstrap_progress().await?;
    } else {
        eprintln!("{}Error:{} {}", colors::ERR, colors::RESET, e);
    }
    Ok(())
}

// v0.0.144: handle_reset removed - use natural language "reset anna" instead

/// Handle uninstall command
pub async fn handle_uninstall() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let uninstall_info = client.uninstall_info().await?;

    println!();
    println!(
        "{}anna uninstall v{}{}",
        colors::HEADER,
        VERSION,
        colors::RESET
    );
    println!();

    println!("This will remove Anna binaries, service, configs, data, logs.");
    println!("It can also remove helpers Anna installed (ollama + models).");
    println!();

    println!("{}Plan:{}", colors::BOLD, colors::RESET);
    println!("  {} stop + disable: annad.service", symbols::ARROW);
    println!(
        "  {} remove: /usr/local/bin/annactl, /usr/local/bin/annad",
        symbols::ARROW
    );
    println!(
        "  {} remove: /etc/anna, /var/lib/anna, /var/log/anna",
        symbols::ARROW
    );
    println!();

    if !uninstall_info.models.is_empty() {
        println!(
            "{}Helpers installed by Anna:{}",
            colors::BOLD,
            colors::RESET
        );
        if uninstall_info.ollama_installed {
            println!("  {} ollama", symbols::ARROW);
        }
        println!(
            "  {} models: {}",
            symbols::ARROW,
            uninstall_info.models.join(", ")
        );
        println!();
    }

    println!("{}Confirmation required{}", colors::BOLD, colors::RESET);
    println!(
        "Type exactly: {}I UNDERSTAND THIS REMOVES ANNA AND ITS DATA{}",
        colors::WARN,
        colors::RESET
    );
    println!();

    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim() != "I UNDERSTAND THIS REMOVES ANNA AND ITS DATA" {
        println!();
        println!("Uninstall cancelled.");
        return Ok(());
    }

    println!();
    println!("Executing uninstall...");

    for cmd in &uninstall_info.commands {
        println!("  {} {}", symbols::ARROW, cmd);
        let status = std::process::Command::new("sudo")
            .args(["sh", "-c", cmd])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("    {}{}{}", colors::OK, symbols::OK, colors::RESET);
            }
            Ok(s) => {
                println!(
                    "    {}Warning: exited with {}{}",
                    colors::WARN,
                    s,
                    colors::RESET
                );
            }
            Err(e) => {
                println!("    {}Error: {}{}", colors::ERR, e, colors::RESET);
            }
        }
    }

    println!();
    println!(
        "{}{}{}  Uninstall complete.",
        colors::OK,
        symbols::OK,
        colors::RESET
    );
    Ok(())
}
