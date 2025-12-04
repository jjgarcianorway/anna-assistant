//! Display helpers for annactl UI.

use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::ui::{colors, symbols, HR};
use anna_shared::VERSION;
use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

use crate::client::AnnadClient;

/// Print status display
pub fn print_status_display(status: &DaemonStatus) {
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

/// Print REPL header
pub fn print_repl_header() {
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

/// Show bootstrap progress with live updates
pub async fn show_bootstrap_progress() -> Result<()> {
    println!();
    println!(
        "{}anna (bootstrap){}",
        colors::HEADER,
        colors::RESET
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
    println!(
        "{}Hello!{} I'm setting up my environment. Come back soon! ;)",
        colors::CYAN,
        colors::RESET
    );
    println!();

    let spinner = &symbols::SPINNER;
    let mut spinner_idx = 0;

    loop {
        // Try to connect and get status
        let status = match AnnadClient::connect().await {
            Ok(mut client) => client.status().await.ok(),
            Err(_) => None,
        };

        // Clear line and show current status
        print!("\r\x1b[K");

        if let Some(status) = &status {
            if status.llm.state == LlmState::Ready {
                println!(
                    "{}{}{}  All set! Anna is ready.",
                    colors::OK,
                    symbols::OK,
                    colors::RESET
                );
                println!();
                println!("{}{}{}", colors::DIM, HR, colors::RESET);
                println!();
                return Ok(());
            }

            let phase = status.llm.phase.as_deref().unwrap_or("initializing");

            if let Some(progress) = &status.llm.progress {
                let bar = anna_shared::ui::progress_bar(progress.percent(), 25);
                let eta = anna_shared::ui::format_duration(progress.eta_seconds);
                print!(
                    "{} {} {} {:.0}%  eta {}",
                    spinner[spinner_idx],
                    phase,
                    bar,
                    progress.percent() * 100.0,
                    eta
                );
            } else {
                print!("{} {}", spinner[spinner_idx], phase);
            }
        } else {
            print!("{} waiting for daemon...", spinner[spinner_idx]);
        }

        io::stdout().flush()?;

        spinner_idx = (spinner_idx + 1) % spinner.len();
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
