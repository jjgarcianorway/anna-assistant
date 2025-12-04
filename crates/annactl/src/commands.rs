//! Command handlers for annactl.

use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::ui::{colors, symbols, HR};
use anna_shared::VERSION;
use anyhow::Result;
use std::io::{self, Write};

use crate::client::AnnadClient;

/// Handle status command
pub async fn handle_status() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;

    print_status_display(&status);
    Ok(())
}

fn print_status_display(status: &DaemonStatus) {
    println!();
    println!(
        "{}annactl v{}{}",
        colors::HEADER,
        status.version,
        colors::RESET
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    let kw = 15; // key width

    // Daemon info
    print_kv("daemon", &format!("{}   (pid {})", status.state, status.pid.unwrap_or(0)), kw);
    print_kv("debug_mode", if status.debug_mode { "ON" } else { "OFF" }, kw);

    let update_str = if status.auto_update {
        match &status.last_update_check {
            Some(t) => {
                let ago = chrono::Utc::now().signed_duration_since(*t);
                format!("ENABLED   last check {:02}:{:02}:{:02} ago",
                    ago.num_hours(), ago.num_minutes() % 60, ago.num_seconds() % 60)
            }
            None => "ENABLED   ( every 600s )".to_string()
        }
    } else {
        "DISABLED".to_string()
    };
    print_kv("auto_update", &update_str, kw);
    println!();

    // LLM info
    let llm_state_color = match status.llm.state {
        LlmState::Ready => colors::OK,
        LlmState::Bootstrapping => colors::WARN,
        LlmState::Error => colors::ERR,
    };
    println!(
        "{:width$} {}{}{}{}",
        "llm",
        llm_state_color,
        status.llm.state,
        if status.llm.state == LlmState::Bootstrapping { " (required)" } else { "" },
        colors::RESET,
        width = kw
    );
    print_kv("provider", &status.llm.provider, kw);

    if let Some(phase) = &status.llm.phase {
        print_kv("phase", phase, kw);
    }

    if let Some(progress) = &status.llm.progress {
        let bar = anna_shared::ui::progress_bar(progress.percent(), 30);
        let current = anna_shared::ui::format_bytes(progress.current_bytes);
        let total = anna_shared::ui::format_bytes(progress.total_bytes);
        let speed = anna_shared::ui::format_bytes(progress.speed_bytes_per_sec);
        let eta = anna_shared::ui::format_duration(progress.eta_seconds);
        println!(
            "{:width$} {} {:.0}%",
            "progress",
            bar,
            progress.percent() * 100.0,
            width = kw
        );
        println!(
            "{:width$} {} / {}   {}/s   eta {}",
            "traffic", current, total, speed, eta,
            width = kw
        );
    }

    if let Some(bench) = &status.llm.benchmark {
        print_kv("benchmark", &format!("DONE   cpu {}, ram {}, gpu {}", bench.cpu, bench.ram, bench.gpu), kw);
    }

    if !status.llm.models.is_empty() {
        let models_str = status.llm.models.iter()
            .map(|m| format!("{}: {}", m.role, m.name))
            .collect::<Vec<_>>()
            .join("\n               ");
        print_kv("models", &models_str, kw);
    }

    if let Some(err) = &status.last_error {
        println!(
            "{:width$} {}{}{}",
            "last_error", colors::ERR, err, colors::RESET,
            width = kw
        );
    } else {
        print_kv("last_error", "none", kw);
    }

    println!();
    if status.llm.state == LlmState::Ready {
        print_kv("health", &format!("{}OK{}", colors::OK, colors::RESET), kw);
    }

    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}

fn print_kv(key: &str, value: &str, width: usize) {
    println!("{:width$} {}", key, value, width = width);
}

/// Handle a single request
pub async fn handle_request(prompt: &str) -> Result<()> {
    let mut client = AnnadClient::connect().await?;

    // First check if daemon is ready
    let status = client.status().await?;
    if status.llm.state != LlmState::Ready {
        println!();
        println!(
            "{}anna (status){}",
            colors::HEADER,
            colors::RESET
        );
        println!("{}{}{}", colors::DIM, HR, colors::RESET);
        println!(
            "LLM is {}{}{}, please wait for bootstrap to complete.",
            colors::WARN,
            status.llm.state,
            colors::RESET
        );
        if let Some(phase) = &status.llm.phase {
            println!("Current phase: {}", phase);
        }
        println!("{}{}{}", colors::DIM, HR, colors::RESET);
        println!();
        return Ok(());
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
                match client.request(input).await {
                    Ok(response) => {
                        println!();
                        println!("{}[anna]{}", colors::OK, colors::RESET);
                        println!("{}", response);
                        println!();
                    }
                    Err(e) => {
                        eprintln!("{}Error:{} {}", colors::ERR, colors::RESET, e);
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

fn print_repl_header() {
    println!();
    println!(
        "{}annactl v{}{}",
        colors::HEADER,
        VERSION,
        colors::RESET
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!("Anna is a local Linux service desk living on your machine.");
    println!(
        "Public commands: {}annactl{} | {}annactl <request>{} | {}annactl status{} | {}annactl -V{} | {}annactl uninstall{}",
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
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

    // Execute commands
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
