//! Command handlers for annactl.

use anna_shared::status::LlmState;
use anna_shared::ui::{colors, symbols, HR};
use anna_shared::VERSION;
use anyhow::Result;
use std::io::{self, Write};

use crate::client::AnnadClient;
use crate::display::{print_repl_header, print_status_display, show_bootstrap_progress};

/// Handle status command
pub async fn handle_status() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;

    print_status_display(&status);
    Ok(())
}

/// Handle a single request
pub async fn handle_request(prompt: &str) -> Result<()> {
    let mut client = AnnadClient::connect().await?;

    // First check if daemon is ready
    let status = client.status().await?;
    if status.llm.state != LlmState::Ready {
        drop(client);
        show_bootstrap_progress().await?;
        client = AnnadClient::connect().await?;
    }

    println!();
    println!(
        "{}anna (dispatch){} received request",
        colors::HEADER,
        colors::RESET
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!("{}[you]{}     {}", colors::CYAN, colors::RESET, prompt);
    println!();

    print!("{} collecting context", symbols::SPINNER[0]);
    io::stdout().flush()?;

    let response = client.request(prompt).await?;

    // Clear spinner line and print response
    print!("\r\x1b[K");
    println!("{}[anna]{}", colors::OK, colors::RESET);
    println!("{}", response);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();

    Ok(())
}

/// Handle REPL mode
pub async fn handle_repl() -> Result<()> {
    print_repl_header();

    // Check if daemon is ready, show bootstrap if not
    {
        let status = match AnnadClient::connect().await {
            Ok(mut client) => client.status().await.ok(),
            Err(_) => None,
        };

        if let Some(status) = status {
            if status.llm.state != LlmState::Ready {
                show_bootstrap_progress().await?;
            }
        }
    }

    let mut client = AnnadClient::connect().await?;

    loop {
        print!("{}anna>{} ", colors::HEADER, colors::RESET);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            "status" => {
                drop(client);
                handle_status().await?;
                client = AnnadClient::connect().await?;
            }
            "help" => {
                println!("Commands:");
                println!("  exit, quit  - Exit REPL");
                println!("  status      - Show Anna status");
                println!("  help        - Show this help");
                println!("  <anything>  - Send as request to Anna");
            }
            _ => {
                // Check if still ready before request
                let status = client.status().await?;
                if status.llm.state != LlmState::Ready {
                    drop(client);
                    show_bootstrap_progress().await?;
                    client = AnnadClient::connect().await?;
                }

                match client.request(input).await {
                    Ok(response) => {
                        println!();
                        println!("{}[anna]{}", colors::OK, colors::RESET);
                        println!("{}", response);
                        println!();
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
                            drop(client);
                            show_bootstrap_progress().await?;
                            client = AnnadClient::connect().await?;
                        } else {
                            eprintln!("{}Error:{} {}", colors::ERR, colors::RESET, e);
                            if let Ok(new_client) = AnnadClient::connect().await {
                                client = new_client;
                            }
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
    println!("{}{}{}  Reset complete. Learned data has been cleared.", colors::OK, symbols::OK, colors::RESET);
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
    println!("  {} remove: /usr/local/bin/annactl, /usr/local/bin/annad", symbols::ARROW);
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
    println!("Type exactly: {}I UNDERSTAND THIS REMOVES ANNA AND ITS DATA{}", colors::WARN, colors::RESET);
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
                println!("    {}Warning: exited with {}{}", colors::WARN, s, colors::RESET);
            }
            Err(e) => {
                println!("    {}Error: {}{}", colors::ERR, e, colors::RESET);
            }
        }
    }

    println!();
    println!("{}{}{}  Uninstall complete.", colors::OK, symbols::OK, colors::RESET);
    Ok(())
}
