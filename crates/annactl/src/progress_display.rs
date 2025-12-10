//! Progress display module for annactl (v0.0.338).
//! v0.0.119: Clean progress messages.
//! v0.0.338: Use centralized UI printing for consistency.

use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, print_label, print_ok, symbols, HR};
use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

use crate::client::AnnadClient;

/// Show bootstrap progress with live updates
pub async fn show_bootstrap_progress() -> Result<()> {
    println!();
    println!("{}anna (bootstrap){}", colors::HEADER, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!("{}Setting up...{}", colors::DIM, colors::RESET);
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

/// Print a progress event (v0.0.144: kept for future streaming UI)
/// v0.0.338: Use centralized print_label for consistency
#[allow(dead_code)]
pub fn print_progress_event(event: &ProgressEvent) {
    let elapsed = format!("{:.1}s", event.elapsed_ms as f64 / 1000.0);

    match &event.event {
        ProgressEventType::Starting { timeout_secs } => {
            print_label(
                &format!("anna->{}", event.stage),
                &format!("starting (timeout {}s) [{}]", timeout_secs, elapsed),
                colors::DIM,
            );
        }
        ProgressEventType::Complete => {
            print_label(
                "anna",
                &format!("{} {}complete{} [{}]", event.stage, colors::OK, colors::RESET, elapsed),
                colors::DIM,
            );
        }
        ProgressEventType::Timeout => {
            print_label(
                "anna",
                &format!("{} {}TIMEOUT{} [{}]", event.stage, colors::ERR, colors::RESET, elapsed),
                colors::DIM,
            );
        }
        ProgressEventType::Error { message } => {
            print_label(
                "anna",
                &format!("{} {}error:{} {} [{}]", event.stage, colors::ERR, colors::RESET, message, elapsed),
                colors::DIM,
            );
        }
        ProgressEventType::Heartbeat => {
            let detail = event.detail.as_deref().unwrap_or("working");
            print_label("anna", &format!("still working: {} [{}]", detail, elapsed), colors::DIM);
        }
        ProgressEventType::ProbeRunning { probe_id } => {
            print_label("anna->probe", &format!("running {} [{}]", probe_id, elapsed), colors::DIM);
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
            print_label(
                "anna",
                &format!("probe {} {} ({}ms) [{}]", probe_id, status, timing_ms, elapsed),
                colors::DIM,
            );
        }
        // v0.0.145: LLM generation progress
        ProgressEventType::Generation { tokens } => {
            // Inline print for live update (no newline)
            print!(
                "\r{}[anna]{} generating... {} tokens [{}]",
                colors::DIM,
                colors::RESET,
                tokens,
                elapsed
            );
            let _ = io::stdout().flush();
        }
        // v0.0.145: Internal comms (fly on wall view)
        ProgressEventType::InternalComms { from, message } => {
            print_label(from, message, colors::CYAN);
        }
        // v0.0.238: Streaming tokens (handled in live_request.rs)
        ProgressEventType::StreamingToken { .. } => {
            // Streaming tokens are handled separately in live_request.rs
        }
    }
}
