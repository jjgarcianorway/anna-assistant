//! Command handlers for annactl.

use anna_shared::progress::ProgressEvent;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, symbols};
use anna_shared::VERSION;
use anyhow::Result;
use std::io::{self, Write};

use crate::client::{AnnadClient, StreamingClient};
use crate::display::{print_progress_event, print_repl_header, print_status_display, show_bootstrap_progress};
use crate::transcript_render;

/// Handle status command
pub async fn handle_status() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;
    print_status_display(&status);
    Ok(())
}

/// Core request function with progress streaming
async fn send_request_with_progress(prompt: &str, debug_mode: bool) -> Result<ServiceDeskResult> {
    if debug_mode {
        StreamingClient::request_with_progress(prompt, |event| {
            print_progress_event(event);
        })
        .await
    } else {
        let mut client = AnnadClient::connect().await?;
        client.request(prompt).await
    }
}

/// Handle a single request (one-shot mode)
pub async fn handle_request(prompt: &str) -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;
    let debug_mode = status.debug_mode;

    if status.llm.state != LlmState::Ready {
        drop(client);
        show_bootstrap_progress().await?;
    }

    // Show spinner only in non-debug mode
    if !debug_mode {
        show_spinner_start();
    }

    let result = send_request_with_progress(prompt, debug_mode).await?;

    // Clear spinner if shown
    if !debug_mode {
        clear_spinner();
    }

    transcript_render::render(&result, debug_mode);
    Ok(())
}

/// Handle REPL mode
pub async fn handle_repl() -> Result<()> {
    print_repl_header();

    // Check daemon status and get debug mode
    let debug_mode = {
        let status = match AnnadClient::connect().await {
            Ok(mut client) => client.status().await.ok(),
            Err(_) => None,
        };

        if let Some(status) = &status {
            if status.llm.state != LlmState::Ready {
                show_bootstrap_progress().await?;
            }
        }
        status.map(|s| s.debug_mode).unwrap_or(true)
    };

    loop {
        print!("{}anna> {}", colors::HEADER, colors::RESET);
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

        match input.to_lowercase().as_str() {
            "exit" | "quit" | "bye" | "q" | ":q" | ":wq" => {
                println!("Goodbye! ;)");
                break;
            }
            "status" => {
                handle_status().await?;
            }
            "help" => {
                print_repl_help();
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

                // Show spinner or stage transitions based on mode
                if !debug_mode {
                    show_spinner_start();
                }

                match send_request_with_progress(input, debug_mode).await {
                    Ok(result) => {
                        if !debug_mode {
                            clear_spinner();
                        }
                        transcript_render::render(&result, debug_mode);
                        println!(); // Extra line for REPL readability
                    }
                    Err(e) => {
                        if !debug_mode {
                            clear_spinner();
                        }
                        handle_request_error(&e).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Show spinner for non-debug mode
fn show_spinner_start() {
    print!("{} thinking...", symbols::SPINNER[0]);
    let _ = io::stdout().flush();
}

/// Clear spinner line
fn clear_spinner() {
    print!("\r\x1b[K");
    let _ = io::stdout().flush();
}

/// Print REPL help
fn print_repl_help() {
    println!();
    println!("{}Commands:{}", colors::BOLD, colors::RESET);
    println!("  exit, quit, bye, q  Exit REPL");
    println!("  status              Show Anna status");
    println!("  help                Show this help");
    println!("  <anything>          Send as request to Anna");
    println!();
    println!("{}Examples:{}", colors::BOLD, colors::RESET);
    println!("  what cpu do i have?");
    println!("  show disk usage");
    println!("  top memory processes");
    println!();
}

/// Handle request error with recovery
async fn handle_request_error(e: &anyhow::Error) -> Result<()> {
    let err_str = e.to_string();
    if err_str.contains("LLM") || err_str.contains("connect") {
        println!();
        println!(
            "{}Oops!{} Something went wrong. Let me fix that...",
            colors::WARN,
            colors::RESET
        );
        show_bootstrap_progress().await?;
    } else {
        eprintln!("{}Error:{} {}", colors::ERR, colors::RESET, e);
    }
    Ok(())
}

/// Handle reset command
pub async fn handle_reset() -> Result<()> {
    print!("This will reset all learned data. Continue? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Reset cancelled.");
        return Ok(());
    }

    let mut client = AnnadClient::connect().await?;
    client.reset().await?;
    println!(
        "{}{}{}  Reset complete. Learned data has been cleared.",
        colors::OK,
        symbols::OK,
        colors::RESET
    );
    Ok(())
}

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
    println!("  {} remove: /usr/local/bin/annactl, /usr/local/bin/annad", symbols::ARROW);
    println!("  {} remove: /etc/anna, /var/lib/anna, /var/log/anna", symbols::ARROW);
    println!();

    if !uninstall_info.models.is_empty() {
        println!("{}Helpers installed by Anna:{}", colors::BOLD, colors::RESET);
        if uninstall_info.ollama_installed {
            println!("  {} ollama", symbols::ARROW);
        }
        println!("  {} models: {}", symbols::ARROW, uninstall_info.models.join(", "));
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
                println!("    {}Warning: exited with {}{}", colors::WARN, s, colors::RESET);
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
