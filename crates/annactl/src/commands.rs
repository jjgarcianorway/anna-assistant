//! Command handlers for annactl.

use anyhow::Result;
use std::io::{self, Write};

use crate::client::AnnadClient;

/// Handle status command
pub async fn handle_status() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;

    println!("Anna Status");
    println!("===========");
    println!("Version:     {}", status.version);
    println!("State:       {}", status.state);
    println!("Uptime:      {}s", status.uptime_seconds);
    println!();

    println!("Ollama");
    println!("------");
    println!("Installed:   {}", if status.ollama.installed { "yes" } else { "no" });
    println!("Running:     {}", if status.ollama.running { "yes" } else { "no" });
    if let Some(version) = &status.ollama.version {
        println!("Version:     {}", version);
    }
    println!();

    if let Some(model) = &status.model {
        println!("Model");
        println!("-----");
        println!("Name:        {}", model.name);
        println!("Pulled:      {}", if model.pulled { "yes" } else { "no" });
        println!();
    }

    println!("Hardware");
    println!("--------");
    println!("CPU:         {} ({} cores)", status.hardware.cpu_model, status.hardware.cpu_cores);
    println!("RAM:         {} GB", status.hardware.ram_bytes / (1024 * 1024 * 1024));
    if let Some(gpu) = &status.hardware.gpu {
        println!("GPU:         {} {} ({} GB VRAM)",
            gpu.vendor, gpu.model, gpu.vram_bytes / (1024 * 1024 * 1024));
    }
    println!();

    println!("Update Check");
    println!("------------");
    if let Some(last) = &status.last_update_check {
        println!("Last:        {}", last);
    } else {
        println!("Last:        never");
    }
    if let Some(next) = &status.next_update_check {
        println!("Next:        {}", next);
    }
    println!();

    println!("Ledger Summary");
    println!("--------------");
    println!("Total entries:  {}", status.ledger.total);
    println!("Packages:       {}", status.ledger.packages);
    println!("Models:         {}", status.ledger.models);
    println!("Files:          {}", status.ledger.files);

    Ok(())
}

/// Handle a single request
pub async fn handle_request(prompt: &str) -> Result<()> {
    let mut client = AnnadClient::connect().await?;

    // First check if daemon is ready
    let status = client.status().await?;
    if status.state != anna_shared::status::DaemonState::Ready {
        eprintln!("Anna is not ready: {}", status.state);
        eprintln!("Please wait for initialization to complete.");
        return Ok(());
    }

    let response = client.request(prompt).await?;
    println!("{}", response);
    Ok(())
}

/// Handle REPL mode
pub async fn handle_repl() -> Result<()> {
    println!("Anna v{} - Interactive Mode", anna_shared::VERSION);
    println!("Type 'exit' or 'quit' to leave, 'status' to check status");
    println!();

    let mut client = AnnadClient::connect().await?;

    loop {
        print!("anna> ");
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
                // Reconnect for fresh status
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
                match client.request(input).await {
                    Ok(response) => println!("{}\n", response),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        // Try to reconnect
                        if let Ok(new_client) = AnnadClient::connect().await {
                            client = new_client;
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
    println!("Reset complete. Learned data has been cleared.");
    Ok(())
}

/// Handle uninstall command
pub async fn handle_uninstall() -> Result<()> {
    println!("Anna Uninstall");
    println!("==============");
    println!();
    println!("This will remove Anna and all its data.");
    print!("Continue? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    let mut client = AnnadClient::connect().await?;
    let commands = client.uninstall().await?;

    println!();
    println!("The following commands will be executed with sudo:");
    println!();
    for cmd in &commands {
        println!("  {}", cmd);
    }
    println!();
    print!("Execute? [y/N] ");
    io::stdout().flush()?;

    input.clear();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    // Execute commands
    for cmd in commands {
        println!("Running: {}", cmd);
        let status = std::process::Command::new("sudo")
            .args(["sh", "-c", &cmd])
            .status();

        match status {
            Ok(s) if s.success() => {}
            Ok(s) => eprintln!("Warning: command exited with {}", s),
            Err(e) => eprintln!("Warning: failed to execute: {}", e),
        }
    }

    println!();
    println!("Uninstall complete.");
    Ok(())
}
