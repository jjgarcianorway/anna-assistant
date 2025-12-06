//! Progress display module for annactl (v0.0.67).
//!
//! Handles progress event rendering and bootstrap progress display.

use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, symbols, HR};
use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

use crate::client::AnnadClient;

/// Show bootstrap progress with live updates
pub async fn show_bootstrap_progress() -> Result<()> {
    println!();
    println!("{}anna (bootstrap){}", colors::HEADER, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
    println!(
        "{}Hello!{} I'm setting up my environment. Come back soon.",
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
                    "{}{}{}  All set. Anna is ready.",
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
