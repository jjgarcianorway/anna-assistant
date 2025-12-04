//! Command handlers for annactl.

#[allow(unused_imports)]
use anna_shared::progress::ProgressEvent;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, symbols, HR};
use anna_shared::VERSION;
use anyhow::Result;
use std::io::{self, Write};

use crate::client::{AnnadClient, StreamingClient};
use crate::display::{
    print_progress_event, print_repl_header, print_status_display, show_bootstrap_progress,
};
use crate::transcript_render;

/// Handle status command
pub async fn handle_status() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;

    print_status_display(&status);
    Ok(())
}

/// Core request function with progress streaming
/// Used by both one-shot and REPL for consistent behavior
async fn send_request_with_progress(prompt: &str, debug_mode: bool) -> Result<ServiceDeskResult> {
    if debug_mode {
        // Use streaming client with progress display
        StreamingClient::request_with_progress(prompt, |event| {
            print_progress_event(event);
        })
        .await
    } else {
        // Simple request without progress polling
        let mut client = AnnadClient::connect().await?;
        client.request(prompt).await
    }
}

/// Handle a single request (one-shot mode)
pub async fn handle_request(prompt: &str) -> Result<()> {
    let mut client = AnnadClient::connect().await?;

    // First check if daemon is ready and get debug mode
    let status = client.status().await?;
    let debug_mode = status.debug_mode;

    if status.llm.state != LlmState::Ready {
        drop(client);
        show_bootstrap_progress().await?;
    }

    // Show initial status
    if debug_mode {
        println!();
        println!(
            "{}[anna->pipeline]{} starting request",
            colors::DIM,
            colors::RESET
        );
    } else {
        print!("{} collecting context", symbols::SPINNER[0]);
        io::stdout().flush()?;
    }

    let result = send_request_with_progress(prompt, debug_mode).await?;

    // Clear spinner line if not debug mode
    if !debug_mode {
        print!("\r\x1b[K");
    }

    // Use unified transcript display
    transcript_render::render(&result, debug_mode);

    Ok(())
}

/// Handle REPL mode
pub async fn handle_repl() -> Result<()> {
    print_repl_header();

    // Check if daemon is ready, show bootstrap if not, and get debug mode
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
                println!("Commands:");
                println!("  exit, quit, bye, q  - Exit REPL");
                println!("  status              - Show Anna status");
                println!("  help                - Show this help");
                println!("  <anything>          - Send as request to Anna");
            }
            _ => {
                // Check if still ready before request
                if let Ok(mut client) = AnnadClient::connect().await {
                    if let Ok(status) = client.status().await {
                        if status.llm.state != LlmState::Ready {
                            show_bootstrap_progress().await?;
                        }
                    }
                }

                if debug_mode {
                    println!(
                        "{}[anna->pipeline]{} starting request",
                        colors::DIM,
                        colors::RESET
                    );
                }

                match send_request_with_progress(input, debug_mode).await {
                    Ok(result) => {
                        // Use unified transcript display - same as one-shot
                        transcript_render::render(&result, debug_mode);
                    }
                    Err(e) => {
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
                    }
                }
            }
        }
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
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!("This will remove Anna binaries, service, configs, data, logs.");
    println!("It can also remove helpers Anna installed (ollama + models).");
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();

    println!("{}Plan{}", colors::BOLD, colors::RESET);
    println!("  {} stop + disable: annad.service", symbols::ARROW);
    println!(
        "  {} remove: /usr/local/bin/annactl, /usr/local/bin/annad",
        symbols::ARROW
    );
    println!("  {} remove: /etc/anna", symbols::ARROW);
    println!("  {} remove: /var/lib/anna", symbols::ARROW);
    println!("  {} remove: /var/log/anna", symbols::ARROW);
    println!();

    if !uninstall_info.models.is_empty() {
        println!("{}Helpers installed by Anna{}", colors::BOLD, colors::RESET);
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
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

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
