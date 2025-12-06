//! Display helpers for annactl UI.
//! v0.0.118: Clean, focused status display.

use anna_shared::rpc::DaemonInfo;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::{colors, HR};
use anna_shared::version::{VersionInfo, VERSION};
use chrono::{DateTime, Local, Utc};

// Re-export from dedicated modules
pub use crate::progress_display::{print_progress_event, show_bootstrap_progress};
pub use crate::stats_display::print_stats_display;

/// Print status display
#[allow(dead_code)]
pub fn print_status_display(status: &DaemonStatus, show_debug: bool) {
    print_status_display_with_daemon_info(status, None, show_debug);
}

/// v0.0.118: Clean, focused status display
pub fn print_status_display_with_daemon_info(
    status: &DaemonStatus,
    daemon_info: Option<&DaemonInfo>,
    show_debug: bool,
) {
    println!();
    println!("{}Anna Service Desk{}", colors::HEADER, colors::RESET);
    println!("{}", HR);

    let client_version = VersionInfo::current();

    // === HEALTH SUMMARY (most important info first) ===
    let version_mismatch = if let Some(info) = daemon_info {
        !client_version.matches(&info.version_info)
    } else {
        status.version != VERSION
    };

    // Overall status line
    print!("\n  Status: ");
    if status.llm.state == LlmState::Ready && !version_mismatch && status.last_error.is_none() {
        println!("{}Ready{}", colors::OK, colors::RESET);
    } else if status.llm.state == LlmState::Bootstrapping {
        println!("{}Starting up{}", colors::WARN, colors::RESET);
    } else if version_mismatch {
        println!("{}Version mismatch{} (restart annad)", colors::WARN, colors::RESET);
    } else if let Some(err) = &status.last_error {
        println!("{}Error:{} {}", colors::ERR, colors::RESET, err);
    } else {
        println!("{}Degraded{}", colors::WARN, colors::RESET);
    }

    // Version
    println!("  Version: {}", VERSION);

    // Uptime
    println!("  Uptime: {}", format_uptime(status.uptime_seconds));

    // LLM status
    let llm_str = match status.llm.state {
        LlmState::Ready => format!("{}Ready{} ({})", colors::OK, colors::RESET, status.llm.provider),
        LlmState::Bootstrapping => {
            if let Some(phase) = &status.llm.phase {
                format!("{}{}{}...", colors::WARN, phase, colors::RESET)
            } else {
                format!("{}Starting{}...", colors::WARN, colors::RESET)
            }
        }
        LlmState::Error => format!("{}Error{}", colors::ERR, colors::RESET),
    };
    println!("  LLM: {}", llm_str);

    // Download progress if bootstrapping
    if let Some(progress) = &status.llm.progress {
        let bar = anna_shared::ui::progress_bar(progress.percent(), 25);
        let current = anna_shared::ui::format_bytes(progress.current_bytes);
        let total = anna_shared::ui::format_bytes(progress.total_bytes);
        println!("       {} {:.0}% ({}/{})", bar, progress.percent() * 100.0, current, total);
    }

    // Update status (only if relevant)
    if status.update.update_available {
        if let Some(ver) = &status.update.latest_version {
            println!("  Update: {}v{} available{}", colors::CYAN, ver, colors::RESET);
        }
    }

    // Open tickets
    if let Ok(open) = TicketTracker::for_user().open_tickets() {
        if !open.is_empty() {
            println!("  Tickets: {} open", open.len());
        }
    }

    // === DEBUG INFO (only with --debug) ===
    if show_debug {
        println!();
        println!("{}[debug]{}", colors::DIM, colors::RESET);

        // Hardware
        let ram_gb = status.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        println!("  CPU: {} ({} cores)", status.hardware.cpu_model, status.hardware.cpu_cores);
        println!("  RAM: {:.1} GB", ram_gb);
        if let Some(gpu) = &status.hardware.gpu {
            let vram_gb = gpu.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            println!("  GPU: {} ({:.1} GB)", gpu.model, vram_gb);
        }

        // Models
        if !status.llm.models.is_empty() {
            for m in &status.llm.models {
                println!("  Model ({}): {}", m.role, m.name);
            }
        }

        // Latency
        if let Some(lat) = &status.latency {
            if lat.sample_count > 0 {
                if let Some(avg) = lat.total_avg_ms {
                    println!("  Avg response: {}ms ({} samples)", avg, lat.sample_count);
                }
            }
        }

        // Active teams
        let active_teams: Vec<_> = status.teams.teams.iter()
            .filter(|t| t.active)
            .map(|t| t.team.to_string())
            .collect();
        if !active_teams.is_empty() {
            println!("  Teams: {}", active_teams.join(", "));
        }

        // PID
        if let Some(pid) = status.pid {
            println!("  PID: {}", pid);
        }
    }

    println!();
    println!("{}", HR);
}

fn format_uptime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}

#[allow(dead_code)]
fn format_local_time(dt: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M").to_string()
}
