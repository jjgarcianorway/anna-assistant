//! Anna CLI - simple REPL interface to ask questions about Arch Linux.
//! v0.1.0: Added UI utilities for Hollywood-style experience

mod display;
mod rpc;
mod spinner;
mod streaming;
mod ui;

use anyhow::Result;
use display::*;
use std::io::{self, Write};
use streaming::ask_streaming;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Handle reset command - clears all statistics and learning data
async fn handle_reset() {
    println!();
    println_colored("RESET", CYAN);
    println!();

    match rpc::reset().await {
        Ok(result) => {
            println_colored("Reset complete:", GREEN);
            for item in &result.cleared {
                println!("  ✓ {}", item);
            }
            println!();
            println_colored("Anna is ready to start fresh.", DIM);
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
        }
    }
    println!();
}

/// Handle a question with clarification loop
async fn handle_question(question: &str) {
    // v0.0.994: Use stable session ID for non-interactive mode
    // This allows pending autofixes to persist between CLI calls
    let session_id = "cli".to_string();
    handle_question_with_clarification(question, false, &session_id).await;
}

/// Handle a question, with optional clarification support
/// When in_repl is true, can prompt user for clarification
async fn handle_question_with_clarification(question: &str, in_repl: bool, session_id: &str) {
    // Clear line and start streaming
    println!();

    let mut current_question = question.to_string();
    let max_clarifications = 3; // Prevent infinite loops
    let mut clarification_count = 0;

    loop {
        match ask_streaming(&current_question, session_id).await {
            Ok(result) => {
                if result.needs_clarification && in_repl && clarification_count < max_clarifications
                {
                    // Display clarification question and prompt user
                    println!();
                    if let Some(ref clarification_q) = result.clarification_question {
                        print_colored("ANNA needs clarification: ", YELLOW);
                        println!("{}", clarification_q);
                    }
                    print_colored("> ", CYAN);
                    io::stdout().flush().ok();

                    // Read user's clarification response
                    let mut response = String::new();
                    if io::stdin().read_line(&mut response).is_ok() {
                        let response = response.trim();
                        if !response.is_empty() && response.to_lowercase() != "cancel" {
                            // Append clarification to original question
                            current_question = format!("{} (Context: {})", question, response);
                            clarification_count += 1;
                            println!();
                            continue; // Re-submit with clarification
                        }
                    }
                    // User cancelled or empty response
                    println_colored("Clarification cancelled.", DIM);
                } else if result.needs_clarification && !in_repl {
                    // Non-REPL mode: just show the clarification question
                    // v0.1.2: Don't show Note for confirmation requests (autofix yes/no prompts)
                    let is_confirmation = result.clarification_question.as_ref()
                        .map(|q| q.contains("yes") || q.contains("no") || q.contains("confirm"))
                        .unwrap_or(false);
                    if !is_confirmation {
                        println!();
                        print_colored("Note: ", YELLOW);
                        println!("This question may need more context. Try running in interactive mode (annactl without arguments).");
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
                        println!("  How do I install neovim?");
                        println!("  Show failed services");
                        println!();
                        println!("Commands: status, stats, help, quit");
                    }
                    _ => {
                        handle_question_with_clarification(input, true, &session_id).await;
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

    if args.len() > 1 {
        let cmd = args[1..].join(" ");

        // Handle built-in commands
        match cmd.to_lowercase().as_str() {
            "status" => {
                print_status().await;
            }
            "stats" => {
                print_stats(false);
            }
            "stats --detailed" | "stats -d" => {
                print_stats(true);
            }
            "reset" => {
                handle_reset().await;
            }
            "help" | "--help" | "-h" => {
                println!("Anna - Arch Linux Assistant");
                println!();
                println!("Usage:");
                println!("  annactl                  Start interactive REPL");
                println!("  annactl status           Show daemon status");
                println!("  annactl stats            Show activity statistics");
                println!("  annactl stats -d         Show detailed statistics");
                println!("  annactl reset            Reset all statistics and learning data");
                println!("  annactl <question>       Ask a question");
                println!();
                println!("Examples:");
                println!("  annactl \"what's my disk usage?\"");
                println!("  annactl how do I install neovim");
            }
            "--version" | "-v" => {
                println!("annactl {}", anna_shared::VERSION);
            }
            _ => {
                // It's a question
                handle_question(&cmd).await;
            }
        }
    } else {
        // REPL mode
        run_repl().await?;
    }

    Ok(())
}
