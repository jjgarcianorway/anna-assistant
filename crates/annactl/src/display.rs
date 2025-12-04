//! Display helpers for annactl UI.

use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::rpc::ServiceDeskResult;
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
    print_kv(
        "daemon",
        &format!("{}   (pid {})", status.state, status.pid.unwrap_or(0)),
        kw,
    );
    print_kv(
        "debug_mode",
        if status.debug_mode { "ON" } else { "OFF" },
        kw,
    );
    println!();

    // Version/Update section
    print_kv("version", &status.version, kw);

    // Show available version from GitHub
    let available = status
        .update
        .available_version
        .as_deref()
        .unwrap_or("checking...");
    if status.update.update_available {
        println!(
            "{:width$} {}{}{} ({}update available{})",
            "available",
            colors::OK,
            available,
            colors::RESET,
            colors::WARN,
            colors::RESET,
            width = kw
        );
    } else {
        print_kv("available", available, kw);
    }

    // Update check pace
    print_kv(
        "check_pace",
        &format!("every {}s", status.update.check_interval_secs),
        kw,
    );

    // Countdown to next check
    if let Some(next) = &status.update.next_check {
        let now = chrono::Utc::now();
        let remaining = next.signed_duration_since(now);
        let secs = remaining.num_seconds().max(0);
        print_kv("next_check", &format!("in {}s", secs), kw);
    }

    // Auto-update status
    let auto_str = if status.update.enabled {
        "ENABLED"
    } else {
        "DISABLED"
    };
    print_kv("auto_update", auto_str, kw);
    println!();

    // Hardware info
    let ram_gb = status.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    print_kv(
        "cpu",
        &format!(
            "{} ({} cores)",
            status.hardware.cpu_model, status.hardware.cpu_cores
        ),
        kw,
    );
    print_kv("ram", &format!("{:.1} GB", ram_gb), kw);

    if let Some(gpu) = &status.hardware.gpu {
        let vram_gb = gpu.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        print_kv(
            "gpu",
            &format!("{} ({:.1} GB VRAM)", gpu.model, vram_gb),
            kw,
        );
    } else {
        print_kv("gpu", "none", kw);
    }
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
        if status.llm.state == LlmState::Bootstrapping {
            " (required)"
        } else {
            ""
        },
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
            "traffic",
            current,
            total,
            speed,
            eta,
            width = kw
        );
    }

    if !status.llm.models.is_empty() {
        let models_str = status
            .llm
            .models
            .iter()
            .map(|m| format!("{}: {}", m.role, m.name))
            .collect::<Vec<_>>()
            .join("\n               ");
        print_kv("models", &models_str, kw);
    }

    if let Some(err) = &status.last_error {
        println!(
            "{:width$} {}{}{}",
            "last_error",
            colors::ERR,
            err,
            colors::RESET,
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
    println!("{}annactl v{}{}", colors::HEADER, VERSION, colors::RESET);
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
    println!("{}anna (bootstrap){}", colors::HEADER, colors::RESET);
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

/// Unified display for service desk responses
/// Used by both one-shot and REPL to ensure consistent output
pub fn print_transcript(prompt: &str, result: &ServiceDeskResult) {
    println!();
    println!(
        "{}anna v{} (dispatch){} {}",
        colors::HEADER,
        VERSION,
        colors::RESET,
        colors::DIM
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    // Show user input
    println!("{}[you]{}", colors::CYAN, colors::RESET);
    println!("{}", prompt);
    println!();

    // Check if clarification needed
    if result.needs_clarification {
        if let Some(question) = &result.clarification_question {
            println!(
                "{}[anna]{} needs clarification",
                colors::WARN,
                colors::RESET
            );
            println!("{}", question);
            println!("{}{}{}", colors::DIM, HR, colors::RESET);
            return;
        }
    }

    // Show response with metadata
    println!(
        "{}[anna]{} {} specialist  reliability: {}",
        colors::OK,
        colors::RESET,
        result.domain,
        format_reliability(result.reliability_score)
    );
    println!("{}", result.answer);

    // Show evidence block (probes used)
    let probes_used: Vec<&str> = result
        .evidence
        .probes_executed
        .iter()
        .filter(|p| p.exit_code == 0)
        .map(|p| p.command.as_str())
        .collect();

    if !probes_used.is_empty() {
        println!();
        println!(
            "{}probes:{} {}",
            colors::DIM,
            colors::RESET,
            probes_used.join(", ")
        );
    }

    println!("{}{}{}", colors::DIM, HR, colors::RESET);
}

/// Format reliability score with color
fn format_reliability(score: u8) -> String {
    let color = if score >= 80 {
        colors::OK
    } else if score >= 50 {
        colors::WARN
    } else {
        colors::ERR
    };
    format!("{}{}%{}", color, score, colors::RESET)
}

/// Print a progress event in debug mode
pub fn print_progress_event(event: &ProgressEvent) {
    let elapsed = format!("{:.1}s", event.elapsed_ms as f64 / 1000.0);

    match &event.event {
        ProgressEventType::Starting { timeout_secs } => {
            println!(
                "{}[anna->{}]{} starting (timeout {}s) [{}]",
                colors::DIM,
                event.stage,
                colors::RESET,
                timeout_secs,
                elapsed
            );
        }
        ProgressEventType::Complete => {
            println!(
                "{}[anna]{} {} {}complete{} [{}]",
                colors::DIM,
                colors::RESET,
                event.stage,
                colors::OK,
                colors::RESET,
                elapsed
            );
        }
        ProgressEventType::Timeout => {
            println!(
                "{}[anna]{} {} {}TIMEOUT{} [{}]",
                colors::DIM,
                colors::RESET,
                event.stage,
                colors::ERR,
                colors::RESET,
                elapsed
            );
        }
        ProgressEventType::Error { message } => {
            println!(
                "{}[anna]{} {} {}error:{} {} [{}]",
                colors::DIM,
                colors::RESET,
                event.stage,
                colors::ERR,
                colors::RESET,
                message,
                elapsed
            );
        }
        ProgressEventType::Heartbeat => {
            let detail = event.detail.as_deref().unwrap_or("working");
            println!(
                "{}[anna]{} still working: {} [{}]",
                colors::DIM,
                colors::RESET,
                detail,
                elapsed
            );
        }
        ProgressEventType::ProbeRunning { probe_id } => {
            println!(
                "{}[anna->probe]{} running {} [{}]",
                colors::DIM,
                colors::RESET,
                probe_id,
                elapsed
            );
        }
        ProgressEventType::ProbeComplete {
            probe_id,
            exit_code,
            timing_ms,
        } => {
            let status = if *exit_code == 0 {
                format!("{}ok{}", colors::OK, colors::RESET)
            } else {
                format!("{}exit {}{}", colors::WARN, exit_code, colors::RESET)
            };
            println!(
                "{}[anna]{} probe {} {} ({}ms) [{}]",
                colors::DIM,
                colors::RESET,
                probe_id,
                status,
                timing_ms,
                elapsed
            );
        }
    }
}
