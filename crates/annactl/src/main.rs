//! Anna CLI - simple REPL interface to ask questions about Arch Linux.
//! v0.1.0: Added UI utilities for Hollywood-style experience
//! v0.3.21: Added event renderer for truth-first output
//! v0.3.35: Added daemon_recovery for self-healing connection
//! v0.3.51: Added fake_daemon for golden transcript testing

mod commands;
mod daemon_recovery;
mod service_state;
mod dialogue;
mod display;
mod event_renderer;
mod fake_daemon;
mod init_wait;
#[allow(dead_code)]
mod repair;
#[allow(dead_code)]
mod report; // Internal aggregation logic, not operator-facing
mod rpc;
mod spinner;
mod streaming;
mod telegram;
mod ui;

use anyhow::Result;
use display::*;
use std::io::{self, Write};
use streaming::ask_streaming;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Handle a question with clarification loop
/// v0.3.146: Accept session_id parameter for proper --session flag support
/// Route a question through the daemon. No client-side intent parsing — the LLM classifies.
async fn handle_question(question: &str, session_id: &str) {
    handle_question_with_clarification(question, false, session_id).await;
}

/// Handle a question with clarification support.
/// v0.3.120: One-shot mode now loops for clarification just like REPL.
async fn handle_question_with_clarification(question: &str, _in_repl: bool, session_id: &str) {
    // Clear line and start streaming
    println!();

    let mut current_question = question.to_string();
    let max_clarifications = 3; // Prevent infinite loops
    let mut clarification_count = 0;

    loop {
        match ask_streaming(&current_question, session_id).await {
            Ok(result) => {
                // v0.3.120: Always handle clarification, even in one-shot mode
                if result.needs_clarification && clarification_count < max_clarifications {
                    // Display clarification question and prompt user
                    println!();
                    if let Some(ref clarification_q) = result.clarification_question {
                        print_colored("Anna: ", YELLOW);
                        println!("{}", clarification_q);
                    } else {
                        print_colored("Anna: ", YELLOW);
                        println!("Could you provide more details?");
                    }
                    print_colored("> ", CYAN);
                    io::stdout().flush().ok();

                    // Read user's clarification response
                    let mut response = String::new();
                    if io::stdin().read_line(&mut response).is_ok() {
                        let response = response.trim();
                        if !response.is_empty()
                            && response.to_lowercase() != "cancel"
                            && response.to_lowercase() != "quit"
                            && response.to_lowercase() != "exit"
                        {
                            // v0.3.133: Detect if this is a confirmation prompt (yes/no, proceed, etc.)
                            // If so, send just the response - daemon is waiting for simple yes/no
                            let is_confirmation = result.clarification_question
                                .as_ref()
                                .map(|q| {
                                    let q_lower = q.to_lowercase();
                                    q_lower.contains("yes") && q_lower.contains("no")
                                        || q_lower.contains("proceed")
                                        || q_lower.contains("confirm")
                                })
                                .unwrap_or(false);

                            if is_confirmation {
                                // Send just the user's response for confirmations
                                current_question = response.to_string();
                            } else {
                                // For clarifications, append context to original question
                                current_question = format!("{} (Context: {})", question, response);
                            }
                            clarification_count += 1;
                            println!();
                            continue; // Re-submit with clarification
                        }
                    }
                    // User cancelled or empty response - show what we have
                    if !result.answer.is_empty() {
                        println!();
                    }
                }
                // Done
                break;
            }
            Err(e) => {
                print_colored("Error: ", RED);
                println!("{}", e);
                break;
            }
        }
    }
}

/// Run the REPL
async fn run_repl() -> Result<()> {
    print_greeting();
    print_status().await;

    // v0.0.992: Show proactive alerts from monitoring
    if show_proactive_alerts() {
        mark_alerts_shown();
    }

    println!();

    // Generate a session_id that persists for this REPL session
    // This enables context tracking across questions ("it", "that service", etc.)
    let session_id = uuid::Uuid::new_v4().to_string();

    let username = std::env::var("USER").unwrap_or_else(|_| "you".to_string());
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        print_colored(&format!("{}: ", username), CYAN);
        io::stdout().flush()?;

        let mut input = String::new();
        match reader.read_line(&mut input).await {
            Ok(0) => {
                // EOF (Ctrl-D)
                println!();
                println_colored("Goodbye!", DIM);
                break;
            }
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                match input.to_lowercase().as_str() {
                    "quit" | "exit" | "q" | ":q" => {
                        println_colored("Goodbye!", DIM);
                        break;
                    }
                    "status" => {
                        print_status().await;
                    }
                    "stats" => {
                        print_stats(false);
                    }
                    "stats --detailed" | "stats -d" => {
                        print_stats(true);
                    }
                    "help" => {
                        println!("Just ask questions about your Arch Linux system!");
                        println!("Examples:");
                        println!("  What's my disk usage?");
                        println!("  Setup telegram bot");
                        println!("  Any suggestions for me?");
                        println!("  Show failed services");
                        println!();
                        println!("Commands: status, stats, help, quit");
                    }
                    _ => {
                        handle_question(input, &session_id).await;
                    }
                }
                println!();
            }
            Err(e) => {
                print_colored("Input error: ", RED);
                println!("{}", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Skip version/help commands — they don't need daemon
    let first_cmd = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let skip_wait = matches!(first_cmd, "--version" | "-v" | "--help" | "-h" | "help");

    if !skip_wait {
        init_wait::wait_for_ready().await;
        init_wait::show_morning_report_if_new();
    }

    if args.len() > 1 {
        // v0.3.146: Parse --session flag properly instead of joining everything
        let mut session_id = "cli".to_string();
        let mut question_parts = Vec::new();

        let mut i = 1;
        while i < args.len() {
            if args[i] == "--session" && i + 1 < args.len() {
                session_id = args[i + 1].clone();
                i += 2;
            } else {
                question_parts.push(args[i].clone());
                i += 1;
            }
        }

        let cmd = question_parts.join(" ");

        // Handle only essential commands - everything else is natural language
        match cmd.to_lowercase().as_str() {
            "status" => {
                print_status().await;
            }
            "help" | "--help" | "-h" => {
                println!("Anna - Arch Linux Assistant");
                println!();
                println!("Usage:");
                println!("  annactl                  Start interactive session");
                println!("  annactl status           Show daemon status");
                println!("  annactl <question>       Ask anything in plain English");
                println!();
                println!("Examples:");
                println!("  annactl \"what's my disk usage?\"");
                println!("  annactl \"show me system health\"");
                println!("  annactl \"setup telegram bot\"");
                println!("  annactl \"any suggestions for me?\"");
                println!("  annactl \"install neovim\"");
                println!("  annactl \"replace grub with limine\"");
                println!();
                println!("Everything is natural language - no special commands needed.");
            }
            "--version" | "-v" => {
                println!("annactl {}", anna_shared::VERSION);
            }
            _ => {
                // Everything else is a question - handle it naturally
                handle_question(&cmd, &session_id).await;
            }
        }
    } else {
        // REPL mode
        run_repl().await?;
    }

    Ok(())
}
