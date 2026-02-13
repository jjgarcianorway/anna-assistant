//! Anna CLI - simple REPL interface to ask questions about Arch Linux.
//! v0.1.0: Added UI utilities for Hollywood-style experience
//! v0.3.21: Added event renderer for truth-first output
//! v0.3.35: Added daemon_recovery for self-healing connection
//! v0.3.51: Added fake_daemon for golden transcript testing

mod daemon_recovery;
mod dialogue;
mod display;
mod event_renderer;
mod fake_daemon;
#[allow(dead_code)]
mod repair;
#[allow(dead_code)]
mod report; // Internal aggregation logic, not operator-facing
mod rpc;
mod spinner;
mod streaming;
mod telegram;
mod ui;

use anna_shared::declaration::CapabilityDeclaration;
use anyhow::Result;
use display::*;
use std::io::{self, Write};
use streaming::ask_streaming;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Format for capability output
enum CapabilitiesFormat {
    /// Human-readable plain text
    Plain,
    /// Compact onboarding summary
    Onboarding,
    /// Deterministic format for diffing
    Deterministic,
}

/// Show capability declaration
fn show_capabilities(format: CapabilitiesFormat) {
    let decl = CapabilityDeclaration::from_ledger();
    let output = match format {
        CapabilitiesFormat::Plain => decl.render_plain_text(),
        CapabilitiesFormat::Onboarding => decl.render_onboarding(),
        CapabilitiesFormat::Deterministic => decl.render_deterministic(),
    };
    println!("{}", output);
}

/// Run real-time watch mode.
/// v0.3.117: Continuous monitoring display.
async fn run_watch_mode(compact: bool) {
    use tokio::time::{interval, Duration};

    // Set up Ctrl+C handler
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }).ok();

    let mut tick = interval(Duration::from_secs(2));

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        anna_shared::watch::print_watch_frame(compact);
        tick.tick().await;
    }

    // Clear screen on exit
    print!("\x1B[2J\x1B[H");
    println!("Watch mode ended.");
}

/// Show capabilities help
fn show_capabilities_help() {
    println!();
    println_colored("CAPABILITY DECLARATION", BOLD);
    println!();
    println!("Anna declares her capabilities before acting. This command shows");
    println!("what Anna can do, cannot do automatically, and will never do.");
    println!();
    println!("Usage:");
    println!("  annactl capabilities             Human-readable declaration");
    println!("  annactl capabilities --onboarding   Compact summary");
    println!("  annactl capabilities --deterministic   Diffable format");
    println!();
    println!("Why this matters:");
    println!("  Anna's trust is structural, not promised. This declaration is");
    println!("  derived directly from the capability ledger and cannot diverge");
    println!("  from actual behavior. What you see is what Anna can do.");
    println!();
}

/// Handle reset command - clears data based on mode
/// v0.3.20: Added modes per spec (memory, config, models, helpers, everything)
async fn handle_reset(mode: anna_shared::rpc::ResetMode, skip_confirm: bool) {
    println!();
    println_colored("RESET", CYAN);
    println!();

    // Show what will be reset
    print!("  mode:          ");
    println_colored(&format!("{:?}", mode).to_lowercase(), YELLOW);
    print!("  will reset:    ");
    println_colored(mode.description(), DIM);
    println!();

    // Require confirmation for destructive modes
    if !skip_confirm && mode == anna_shared::rpc::ResetMode::Everything {
        print_colored("This will delete all Anna data and cannot be undone.", YELLOW);
        println!();
        print!("  Type 'yes' to confirm: ");
        std::io::stdout().flush().ok();

        let mut response = String::new();
        if std::io::stdin().read_line(&mut response).is_err() {
            println_colored("Cancelled.", DIM);
            return;
        }
        if response.trim().to_lowercase() != "yes" {
            println_colored("Reset cancelled.", DIM);
            println!();
            return;
        }
        println!();
    }

    match rpc::reset(mode).await {
        Ok(result) => {
            println_colored("Reset complete:", GREEN);
            for item in &result.cleared {
                println!("  [OK] {}", item);
            }
            if let Some(backup) = &result.backup_path {
                println!();
                print_colored("  backup saved: ", DIM);
                println_colored(backup, CYAN);
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

/// Show reset help
fn show_reset_help() {
    println!();
    println_colored("RESET MODES", BOLD);
    println!();
    println!("  annactl reset              Reset everything (with confirmation)");
    println!("  annactl reset memory       Reset memory only (experiences, patterns)");
    println!("  annactl reset config       Reset config only (settings to defaults)");
    println!("  annactl reset models       Reset model preferences");
    println!("  annactl reset helpers      Reset helper tracking");
    println!("  annactl reset everything   Full factory reset");
    println!();
    println!("  annactl reset --force      Skip confirmation");
    println!();
}

/// Handle a question with clarification loop
/// v0.3.146: Accept session_id parameter for proper --session flag support
/// v0.3.159: Added direct PDF generation for report requests
async fn handle_question(question: &str, session_id: &str) {
    // Check for direct report/PDF request
    let question_lower = question.to_lowercase();
    // "pdf" anywhere → PDF report. "generate/create report" without "pdf" also triggers it.
    let is_report_request = question_lower.contains("pdf")
        || ((question_lower.contains("report") || question_lower.contains("generate"))
            && (question_lower.contains("generate") || question_lower.contains("create")
                || question_lower.contains("extended")));

    if is_report_request {
        handle_pdf_report_request().await;
        return;
    }

    handle_question_with_clarification(question, false, session_id).await;
}

/// Handle a PDF report request directly
async fn handle_pdf_report_request() {
    println!();
    print_colored("Generating system health report...", CYAN);
    println!();

    // Call daemon to generate PDF
    match rpc::generate_report().await {
        Ok(path) => {
            println!();
            print_colored("✓ Report generated:", GREEN);
            println!(" {}", path.display());
            println!();
            println_colored("The PDF contains:", DIM);
            println!("  • System health overview");
            println!("  • 7-day performance trends");
            println!("  • Predictive alerts and forecasts");
            println!("  • Personalized recommendations");
            println!("  • Automated maintenance summary");
            println!();

            // Check if Telegram is configured
            if std::path::Path::new("/etc/anna/telegram.env").exists() {
                print_colored("📤 Sending to Telegram...", CYAN);
                println!();
                if let Err(e) = rpc::send_report_to_telegram(&path).await {
                    print_colored("Note: ", YELLOW);
                    println!("Could not send to Telegram: {}", e);
                }
            }
        }
        Err(e) => {
            print_colored("Error generating report: ", RED);
            println!("{}", e);
            println!();
            print_colored("Tip: ", YELLOW);
            println!("Make sure the daemon is running and fonts are installed");
        }
    }
    println!();
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
