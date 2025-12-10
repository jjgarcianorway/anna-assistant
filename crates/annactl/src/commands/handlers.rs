//! Command handlers (v0.0.356).
//! v0.0.330: Initial version.
//! v0.0.339: Use centralized UI printing for consistency.
//! v0.0.344: Use print_title() for header display.
//! v0.0.349: Use print_step() for action steps.
//! v0.0.356: Uninstall uses centralized UI helpers.

use anna_shared::probe_learning::ProbeLearningStore;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::status::LlmState;
use anna_shared::ui::{colors, print_label, print_ok, print_section_header, print_step, print_title, print_warn};
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
    print_step("stop + disable: annad.service");
    print_step("remove: /usr/local/bin/annactl, /usr/local/bin/annad");
    print_step("remove: /etc/anna, /var/lib/anna, /var/log/anna");
    println!();

    if !uninstall_info.models.is_empty() {
        print_section_header("helpers installed by anna");
        if uninstall_info.ollama_installed {
            print_step("ollama");
        }
        print_step(&format!("models: {}", uninstall_info.models.join(", ")));
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
        print_step(cmd);
        let status = std::process::Command::new("sudo")
            .args(["sh", "-c", cmd])
            .status();

        match status {
            Ok(s) if s.success() => {
                print_ok("done");
            }
            Ok(s) => {
                print_warn(&format!("exited with {}", s));
            }
            Err(e) => {
                print_label("error", &e.to_string(), colors::ERR);
            }
        }
    }

    println!();
    print_ok("Uninstall complete.");
    Ok(())
}

/// Handle reset command (v0.0.329)
pub async fn handle_reset() -> Result<()> {
    let mut client = AnnadClient::connect().await?;

    println!();
    print_title("anna reset");
    println!();

    print_section_header("plan");
    print_step("Clear learned recipes");
    print_step("Clear knowledge base");
    print_step("Clear event log (stats)");
    print_step("Clear probe learning");
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
