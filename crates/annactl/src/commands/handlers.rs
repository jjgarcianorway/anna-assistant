//! Command handlers (v0.0.344).
//! v0.0.330: Initial version.
//! v0.0.339: Use centralized UI printing for consistency.
//! v0.0.344: Use print_title() for header display.

use anna_shared::probe_learning::ProbeLearningStore;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, print_label, print_ok, print_section_header, print_title, print_warn, symbols};
use anna_shared::version::VERSION;
use anyhow::Result;
use std::io::{self, Write};

use crate::client::AnnadClient;
use crate::display::{print_stats_display, print_status_display, show_bootstrap_progress};
use crate::live_request::send_request_with_progress;
use crate::transcript_render;

use super::feedback::handle_feedback_request;

// v0.0.97: Change management (handle_proposed_change still needed for config changes)
use crate::change_commands::handle_proposed_change;

/// Handle status command - shows Anna's health, config, and system info
pub async fn handle_status() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;
    let snapshot = client.status_snapshot().await.ok();
    let daemon_info = client.get_daemon_info().await.ok();

    print_status_display(&status, snapshot.as_ref(), daemon_info.as_ref());
    Ok(())
}

/// Handle stats command (v0.0.27)
pub async fn handle_stats() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let stats = client.stats().await?;
    print_stats_display(&stats);
    Ok(())
}

/// Core request function (v0.0.148: kept for fallback, use send_request_with_progress)
#[allow(dead_code)]
pub async fn send_request(prompt: &str) -> Result<ServiceDeskResult> {
    let mut client = AnnadClient::connect().await?;
    client.request(prompt).await
}

/// Handle a single request (one-shot mode)
pub async fn handle_request(prompt: &str) -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let status = client.status().await?;

    if status.llm.state != LlmState::Ready {
        drop(client);
        show_bootstrap_progress().await?;
    }

    // v0.0.148: Use live progress display for fly-on-wall experience
    println!();
    let result = send_request_with_progress(prompt).await?;
    println!();

    // Render the result
    transcript_render::render(&result);

    // v0.0.96: Handle proposed config changes
    let proposed: Vec<_> = if !result.proposed_changes.is_empty() {
        result.proposed_changes.clone()
    } else {
        result.proposed_change.iter().cloned().collect()
    };
    if !proposed.is_empty() {
        let summary = handle_proposed_change(&proposed).await?;
        if summary.failed {
            print_label("config", "Application hit errors; review details above", colors::ERR);
        } else if summary.applied > 0 {
            let msg = format!(
                "Applied ({} step{}, {} noop)",
                summary.applied,
                if summary.applied == 1 { "" } else { "s" },
                summary.noop
            );
            print_label("config", &msg, colors::OK);
        } else {
            print_label("config", "Nothing to change; already configured", colors::DIM);
        }
    }

    // v0.0.103: Handle feedback request from Anna
    if let Some(ref feedback_req) = result.feedback_request {
        handle_feedback_request(feedback_req).await;
    }

    Ok(())
}

/// Handle uninstall command
pub async fn handle_uninstall() -> Result<()> {
    let mut client = AnnadClient::connect().await?;
    let uninstall_info = client.uninstall_info().await?;

    println!();
    print_title(&format!("anna uninstall v{}", VERSION));
    println!();

    println!("This will remove Anna binaries, service, configs, data, logs.");
    println!("It can also remove helpers Anna installed (ollama + models).");
    println!();

    print_section_header("plan");
    println!("  {} stop + disable: annad.service", symbols::ARROW);
    println!("  {} remove: /usr/local/bin/annactl, /usr/local/bin/annad", symbols::ARROW);
    println!("  {} remove: /etc/anna, /var/lib/anna, /var/log/anna", symbols::ARROW);
    println!();

    if !uninstall_info.models.is_empty() {
        print_section_header("helpers installed by anna");
        if uninstall_info.ollama_installed {
            println!("  {} ollama", symbols::ARROW);
        }
        println!("  {} models: {}", symbols::ARROW, uninstall_info.models.join(", "));
        println!();
    }

    print_section_header("confirmation required");
    println!(
        "  Type exactly: {}I UNDERSTAND THIS REMOVES ANNA AND ITS DATA{}",
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

/// Handle reset command (v0.0.329)
pub async fn handle_reset() -> Result<()> {
    let mut client = AnnadClient::connect().await?;

    println!();
    print_title("anna reset");
    println!();

    print_section_header("plan");
    println!("  {} Clear learned recipes", symbols::ARROW);
    println!("  {} Clear knowledge base", symbols::ARROW);
    println!("  {} Clear event log (stats)", symbols::ARROW);
    println!("  {} Clear probe learning", symbols::ARROW);
    println!();

    print_section_header("confirmation required");
    print!("  {}Confirm reset?{} [y/N]: ", colors::WARN, colors::RESET);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!();
        print_label("cancelled", "Reset aborted by user", colors::DIM);
        return Ok(());
    }

    println!();

    // v0.0.329: Also reset probe learning
    if let Err(e) = ProbeLearningStore::reset() {
        print_warn(&format!("Failed to reset probe learning: {}", e));
    }

    client.reset().await?;

    print_ok("Reset complete. Anna will start fresh.");
    Ok(())
}
