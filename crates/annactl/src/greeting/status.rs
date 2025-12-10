//! Status display sections (v0.0.346).
//!
//! v0.0.275: print_since_last_time now unused (LLM generates greetings), kept for fallback.
//! v0.0.346: Use print_hint() and print_label() for consistency.

#![allow(dead_code)]

use anna_shared::snapshot::{DeltaItem, SystemSnapshot};
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::telemetry::TelemetrySnapshot;
use anna_shared::ui::{colors, print_hint, print_label, symbols};

use super::types::InteractionInfo;

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
                    symbols::BULLET,
                    secs
                );
                // v0.0.142: Add helpful context
                if secs < 10 {
                    print_hint("Don't worry, small variations are normal.");
                }
            } else {
                println!(
                    "  {} {}Your boot time improved by {} seconds!{}",
                    symbols::BULLET,
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
                    symbols::BULLET,
                    colors::WARN,
                    colors::RESET,
                    mount,
                    curr
                );
                print_hint("Consider cleaning up or expanding storage.");
            }
            DeltaItem::DiskCritical { mount, curr, .. } => {
                println!(
                    "  {} {}[critical]{} Disk {} is at {}% - needs attention!",
                    symbols::BULLET,
                    colors::ERR,
                    colors::RESET,
                    mount,
                    curr
                );
            }
            DeltaItem::DiskIncreased { mount, prev, curr } => {
                println!(
                    "  {} Disk {} grew from {}% to {}%.",
                    symbols::BULLET,
                    mount,
                    prev,
                    curr
                );
            }
            DeltaItem::NewFailedService { unit } => {
                println!(
                    "  {} {}[fail]{} Service {} has failed.",
                    symbols::BULLET,
                    colors::ERR,
                    colors::RESET,
                    unit
                );
                print_hint(&format!("Ask me to check it with: \"what's wrong with {}\"", unit));
            }
            DeltaItem::ServiceRecovered { unit } => {
                println!(
                    "  {} {}[recovered]{} Service {} is back up!",
                    symbols::BULLET,
                    colors::OK,
                    colors::RESET,
                    unit
                );
            }
            DeltaItem::MemoryHigh { curr_percent, .. } => {
                println!(
                    "  {} {}[warn]{} Memory usage is high at {}%.",
                    symbols::BULLET,
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
                    symbols::BULLET,
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
                symbols::BULLET,
                failed_services,
                if failed_services == 1 { " is" } else { "s are" }
            );
        } else {
            println!(
                "  {} {}No warnings or errors detected. Looking good!{}",
                symbols::BULLET,
                colors::OK,
                colors::RESET
            );
        }
    } else if failed_services > 0 && items_shown < max_items {
        println!(
            "  {} Also, {} service{} in failed state.",
            symbols::BULLET,
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
                print_hint(&format!("Systems ready. Translator: {}, Specialist: {}", trans, spec));
            } else {
                print_hint("All systems ready.");
            }
        }
        LlmState::Bootstrapping => {
            if let Some(phase) = &status.llm.phase {
                print_label("starting", &format!("{}...", phase), colors::WARN);
            } else {
                print_label("starting", "Preparing AI models...", colors::WARN);
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
                print_hint(&format!("Systems ready (models loading). Translator: {}, Specialist: {}", trans, spec));
            } else {
                print_hint("Systems ready (downloading models in background)...");
            }
        }
        LlmState::Error => {
            print_label("error", "AI models not available. Some features may be limited.", colors::ERR);
            if let Some(err) = &status.last_error {
                print_hint(err);
            }
        }
    }

    // Update notification
    if status.update.update_available {
        if let Some(ver) = &status.update.latest_version {
            println!();
            print_label("update", &format!("Version {} is available. I'll update automatically.", ver), colors::CYAN);
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
