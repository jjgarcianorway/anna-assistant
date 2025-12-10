//! Status display sections (v0.0.186).
//!
//! v0.0.275: print_since_last_time now unused (LLM generates greetings), kept for fallback.

#![allow(dead_code)]

use anna_shared::snapshot::{DeltaItem, SystemSnapshot};
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::telemetry::TelemetrySnapshot;
use anna_shared::ui::colors;

use super::types::{bullet, InteractionInfo};

pub fn print_since_last_time(
    telemetry: &TelemetrySnapshot,
    health_deltas: &[DeltaItem],
    failed_services: usize,
    info: &InteractionInfo,
) {
    // Only show "since last time" if it's been a while
    let show_section = info.hours_since_last.map(|h| h > 1).unwrap_or(false);
    if !show_section {
        return;
    }

    // v0.0.142: More conversational header
    println!();
    println!("Since the last time, a few things happened:");
    println!();

    let mut items_shown = 0;
    let max_items = 4;

    // Boot time changes - with conversational explanation
    if let Some(boot_ms) = telemetry.boot_delta_ms {
        if boot_ms.abs() > 2000 && items_shown < max_items {
            let secs = boot_ms.unsigned_abs() / 1000;
            if boot_ms > 0 {
                println!(
                    "  {} Your boot time increased by {} seconds.",
                    bullet(),
                    secs
                );
                // v0.0.142: Add helpful context
                if secs < 10 {
                    println!(
                        "    {}Don't worry, small variations are normal.{}",
                        colors::DIM,
                        colors::RESET
                    );
                }
            } else {
                println!(
                    "  {} {}Your boot time improved by {} seconds!{}",
                    bullet(),
                    colors::OK,
                    secs,
                    colors::RESET
                );
            }
            items_shown += 1;
        }
    }

    // Health deltas with conversational messages
    for delta in health_deltas.iter().take(max_items - items_shown) {
        match delta {
            DeltaItem::DiskWarning { mount, curr, .. } => {
                println!(
                    "  {} {}[warn]{} Disk {} is at {}% - getting full.",
                    bullet(),
                    colors::WARN,
                    colors::RESET,
                    mount,
                    curr
                );
                println!(
                    "    {}Consider cleaning up or expanding storage.{}",
                    colors::DIM,
                    colors::RESET
                );
            }
            DeltaItem::DiskCritical { mount, curr, .. } => {
                println!(
                    "  {} {}[critical]{} Disk {} is at {}% - needs attention!",
                    bullet(),
                    colors::ERR,
                    colors::RESET,
                    mount,
                    curr
                );
            }
            DeltaItem::DiskIncreased { mount, prev, curr } => {
                println!(
                    "  {} Disk {} grew from {}% to {}%.",
                    bullet(),
                    mount,
                    prev,
                    curr
                );
            }
            DeltaItem::NewFailedService { unit } => {
                println!(
                    "  {} {}[fail]{} Service {} has failed.",
                    bullet(),
                    colors::ERR,
                    colors::RESET,
                    unit
                );
                println!(
                    "    {}Ask me to check it with: \"what's wrong with {}\"{}",
                    colors::DIM,
                    unit,
                    colors::RESET
                );
            }
            DeltaItem::ServiceRecovered { unit } => {
                println!(
                    "  {} {}[recovered]{} Service {} is back up!",
                    bullet(),
                    colors::OK,
                    colors::RESET,
                    unit
                );
            }
            DeltaItem::MemoryHigh { curr_percent, .. } => {
                println!(
                    "  {} {}[warn]{} Memory usage is high at {}%.",
                    bullet(),
                    colors::WARN,
                    colors::RESET,
                    curr_percent
                );
            }
            DeltaItem::MemoryIncreased {
                prev_percent,
                curr_percent,
            } => {
                println!(
                    "  {} Memory usage went from {}% to {}%.",
                    bullet(),
                    prev_percent,
                    curr_percent
                );
            }
        }
        items_shown += 1;
    }

    // Service status summary - conversational
    if items_shown == 0 {
        if failed_services > 0 {
            println!(
                "  {} {} service{} in failed state.",
                bullet(),
                failed_services,
                if failed_services == 1 { " is" } else { "s are" }
            );
        } else {
            println!(
                "  {} {}No warnings or errors detected. Looking good!{}",
                bullet(),
                colors::OK,
                colors::RESET
            );
        }
    } else if failed_services > 0 && items_shown < max_items {
        println!(
            "  {} Also, {} service{} in failed state.",
            bullet(),
            failed_services,
            if failed_services == 1 { " is" } else { "s are" }
        );
    }
}

pub fn print_system_readiness(status: &DaemonStatus) {
    println!();

    match status.llm.state {
        LlmState::Ready => {
            // Show which models are ready
            if let (Some(trans), Some(spec)) =
                (&status.llm.translator_model, &status.llm.specialist_model)
            {
                println!(
                    "{}Systems ready. Translator: {}, Specialist: {}{}",
                    colors::DIM,
                    trans,
                    spec,
                    colors::RESET
                );
            } else {
                println!("{}All systems ready.{}", colors::DIM, colors::RESET);
            }
        }
        LlmState::Bootstrapping => {
            if let Some(phase) = &status.llm.phase {
                println!("{}[starting]{} {}...", colors::WARN, colors::RESET, phase);
            } else {
                println!(
                    "{}[starting]{} Preparing AI models...",
                    colors::WARN,
                    colors::RESET
                );
            }
            // Show progress if available
            if let Some(progress) = &status.llm.progress {
                let bar = anna_shared::ui::progress_bar(progress.percent(), 30);
                println!("  {} {:.0}%", bar, progress.percent() * 100.0);
            }
        }
        LlmState::PullingModels => {
            // v0.0.310: Daemon is ready but models are loading in background
            if let (Some(trans), Some(spec)) =
                (&status.llm.translator_model, &status.llm.specialist_model)
            {
                println!(
                    "{}Systems ready (models loading). Translator: {}, Specialist: {}{}",
                    colors::DIM,
                    trans,
                    spec,
                    colors::RESET
                );
            } else {
                println!(
                    "{}Systems ready (downloading models in background)...{}",
                    colors::DIM,
                    colors::RESET
                );
            }
        }
        LlmState::Error => {
            println!(
                "{}[error]{} AI models not available. Some features may be limited.",
                colors::ERR,
                colors::RESET
            );
            if let Some(err) = &status.last_error {
                println!("  {}{}{}", colors::DIM, err, colors::RESET);
            }
        }
    }

    // Update notification
    if status.update.update_available {
        if let Some(ver) = &status.update.latest_version {
            println!();
            println!(
                "{}[update]{} Version {} is available. I'll update automatically.",
                colors::CYAN,
                colors::RESET,
                ver
            );
        }
    }
}

pub fn collect_failed_services(snapshot: &mut SystemSnapshot) -> usize {
    let mut count = 0;

    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-pager", "-q"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(".service") || line.contains(".mount") {
                    count += 1;
                    if let Some(unit) = line
                        .split_whitespace()
                        .find(|p| p.ends_with(".service") || p.ends_with(".mount"))
                    {
                        snapshot.add_failed_service(unit);
                    }
                }
            }
        }
    }

    count
}
