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
                        // Phase 22: Consistent "Anna:" prefix, no "ANNA needs clarification"
                        print_colored("Anna: ", YELLOW);
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
                handle_reset(anna_shared::rpc::ResetMode::Everything, false).await;
            }
            "reset --help" | "reset -h" => {
                show_reset_help();
            }
            cmd if cmd.starts_with("reset ") => {
                let rest = cmd.strip_prefix("reset ").unwrap().trim();
                let (mode_str, force) = if rest.contains("--force") || rest.contains("-f") {
                    (rest.replace("--force", "").replace("-f", "").trim().to_string(), true)
                } else {
                    (rest.to_string(), false)
                };

                if mode_str.is_empty() {
                    handle_reset(anna_shared::rpc::ResetMode::Everything, force).await;
                } else if let Some(mode) = anna_shared::rpc::ResetMode::from_str(&mode_str) {
                    handle_reset(mode, force).await;
                } else {
                    print_colored("Error: ", RED);
                    println!("Unknown reset mode '{}'. Use 'reset --help' for available modes.", mode_str);
                }
            }
            "repair wifi" => {
                repair::handle_repair_wifi().await;
            }
            "repair" | "repair --help" | "repair -h" => {
                repair::show_repair_help();
            }
            "health" | "health report" => {
                // v0.3.114: Visual health report with charts
                let report = anna_shared::health_report::generate_health_report();
                println!("{}", report);
            }
            "health summary" | "health -s" => {
                // v0.3.114: One-line health summary
                let summary = anna_shared::health_report::health_summary();
                println!("{}", summary);
            }
            "capabilities" | "caps" => {
                show_capabilities(CapabilitiesFormat::Plain);
            }
            "capabilities --onboarding" | "caps --onboarding" => {
                show_capabilities(CapabilitiesFormat::Onboarding);
            }
            "capabilities --deterministic" | "caps --deterministic" => {
                show_capabilities(CapabilitiesFormat::Deterministic);
            }
            "capabilities --help" | "caps --help" | "capabilities -h" | "caps -h" => {
                show_capabilities_help();
            }
            "help" | "--help" | "-h" => {
                println!("Anna - Arch Linux Assistant");
                println!();
                println!("Usage:");
                println!("  annactl                  Start interactive REPL");
                println!("  annactl status           Show daemon status");
                println!("  annactl stats            Show activity statistics");
                println!("  annactl stats -d         Show detailed statistics");
                println!("  annactl capabilities     Show what Anna can and cannot do");
                println!("  annactl health           Show visual system health report");
                println!("  annactl health -s        Show one-line health summary");
                println!("  annactl reset [mode]     Reset data (use 'reset --help' for modes)");
                println!("  annactl repair wifi      Diagnose and repair WiFi issues");
                println!("  annactl <question>       Ask a question");
                println!();
                println!("Reset modes: memory, config, models, helpers, everything");
                println!();
                println!("Examples:");
                println!("  annactl \"what's my disk usage?\"");
                println!("  annactl how do I install neovim");
                println!("  annactl capabilities");
                println!("  annactl reset memory");
                println!("  annactl repair wifi");
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
