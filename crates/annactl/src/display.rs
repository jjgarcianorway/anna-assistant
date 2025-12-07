//! Display helpers for annactl UI.
//! v0.0.119: Clean, focused status display.

use anna_shared::event_log::EventLog;
use anna_shared::rpc::DaemonInfo;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::{colors, HR};
use anna_shared::version::{VersionInfo, VERSION};
use chrono::{DateTime, Local, TimeZone, Utc};

// Re-export from dedicated modules
pub use crate::progress_display::{print_progress_event, show_bootstrap_progress};
pub use crate::stats_display::print_stats_display;

/// Print status display
#[allow(dead_code)]
pub fn print_status_display(
    status: &DaemonStatus,
    snapshot: Option<&StatusSnapshot>,
    show_debug: bool,
) {
    print_status_display_with_daemon_info(status, snapshot, None, show_debug);
}

/// v0.0.118: Clean, focused status display
pub fn print_status_display_with_daemon_info(
    status: &DaemonStatus,
    snapshot: Option<&StatusSnapshot>,
    daemon_info: Option<&DaemonInfo>,
    show_debug: bool,
) {
    println!();
    println!("{}Anna Service Desk{}", colors::HEADER, colors::RESET);
    println!("{}", HR);

    let client_version = VersionInfo::current();
    let daemon_version_matches = daemon_info
        .map(|info| client_version.matches(&info.version_info))
        .unwrap_or_else(|| status.version == VERSION);
    let update_available = snapshot
        .map(|s| s.update_available())
        .unwrap_or(status.update.update_available);
    let latest_version = snapshot
        .and_then(|s| s.versions.git_tag_remote.clone())
        .or_else(|| status.update.latest_version.clone());

    // === HEALTH SUMMARY ===
    print!("\n  Status: ");
    if status.llm.state == LlmState::Bootstrapping {
        println!("{}Starting up{}", colors::WARN, colors::RESET);
    } else if let Some(err) = &status.last_error {
        println!("{}Error:{} {}", colors::ERR, colors::RESET, err);
    } else if !daemon_version_matches {
        println!(
            "{}Version mismatch{} (restart annad)",
            colors::WARN,
            colors::RESET
        );
    } else if update_available {
        println!("{}Ready (update available){}", colors::WARN, colors::RESET);
    } else {
        println!("{}Ready{}", colors::OK, colors::RESET);
    }

    // === VERSIONS & UPDATE CADENCE ===
    println!();
    println!("{}Versions & Updates{}", colors::BOLD, colors::RESET);
    println!("  annactl: {}", client_version.display_string());
    if let Some(info) = daemon_info {
        println!(
            "  annad:   {} (pid {}, up {})",
            info.version_info.display_string(),
            info.pid,
            format_uptime(info.uptime_secs)
        );
    } else {
        println!("  annad:   {}", status.version);
    }
    if let Some(latest) = latest_version.as_deref() {
        println!(
            "  latest:  {}{}{}",
            if update_available {
                colors::CYAN
            } else {
                colors::DIM
            },
            latest,
            colors::RESET
        );
    }
    println!(
        "  auto-update: {} (every {}s)",
        fmt_on_off(status.update.enabled),
        status.update.check_interval_secs
    );
    println!(
        "  last check: {}",
        format_optional_dt(status.update.last_check_at.as_ref())
    );
    if let Some(snap) = snapshot {
        println!(
            "  next check: {}",
            format_optional_ts(snap.update.next_check_ts)
        );
    }

    // === DAEMON & LLM ===
    println!();
    println!("{}Daemon & LLM{}", colors::BOLD, colors::RESET);
    let uptime_str = daemon_info
        .map(|d| format_uptime(d.uptime_secs))
        .unwrap_or_else(|| format_uptime(status.uptime_seconds));
    println!("  Daemon: {} (uptime {})", status.state, uptime_str);

    let llm_str = match status.llm.state {
        LlmState::Ready => format!(
            "{}Ready{} ({})",
            colors::OK,
            colors::RESET,
            status.llm.provider
        ),
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

    if let Some(progress) = &status.llm.progress {
        let bar = anna_shared::ui::progress_bar(progress.percent(), 20);
        println!(
            "  Download: {} {:.0}% ({}/{})",
            bar,
            progress.percent() * 100.0,
            anna_shared::ui::format_bytes(progress.current_bytes),
            anna_shared::ui::format_bytes(progress.total_bytes)
        );
    }

    if !status.llm.models.is_empty() {
        let models: Vec<String> = status
            .llm
            .models
            .iter()
            .map(|m| format!("{}={}", m.role, m.name))
            .collect();
        println!("  Models: {}", models.join(", "));
    }

    // === ACCESS & DEPENDENCIES ===
    if let Some(snap) = snapshot {
        println!();
        println!("{}Access & Dependencies{}", colors::BOLD, colors::RESET);
        let groups = if snap.perms.groups.is_empty() {
            "-".to_string()
        } else {
            snap.perms.groups.join(", ")
        };
        println!("  User: {} [{}]", snap.perms.user, groups);
        println!(
            "  Daemon socket: {}",
            fmt_ok_warn(snap.perms.can_talk_to_daemon)
        );
        println!("  Data dir: {}", fmt_ok_warn(snap.perms.data_dir_ok));

        let ollama_state = match (snap.models.ollama_present, snap.models.ollama_running) {
            (true, true) => format!("{}present{}, running", colors::OK, colors::RESET),
            (true, false) => format!("{}present{}, stopped", colors::WARN, colors::RESET),
            _ => format!("{}missing{}", colors::ERR, colors::RESET),
        };
        println!("  Ollama: {}", ollama_state);

        if snap.helpers.total > 0 {
            let samples: Vec<String> = snap
                .helpers
                .list
                .iter()
                .take(4)
                .map(|h| format!("{} [{}]", h.name, h.source))
                .collect();
            println!(
                "  Helpers: {} total (anna {}, user {}, bundled {})",
                snap.helpers.total,
                snap.helpers.anna_installed,
                snap.helpers.user_installed,
                snap.helpers.bundled
            );
            if !samples.is_empty() {
                println!("           {}", samples.join(", "));
            }
        }

        if !snap.models.roles.is_empty() {
            let bindings: Vec<String> = snap
                .models
                .roles
                .iter()
                .map(|r| format!("{} {} → {}", r.team, r.role, r.model_name))
                .collect();
            println!("  LLM roles: {}", bindings.join(", "));
        }
    }

    // === CONFIG ===
    println!();
    println!("{}Config{}", colors::BOLD, colors::RESET);
    let debug_mode = snapshot
        .map(|s| s.config.debug_mode)
        .unwrap_or(status.debug_mode);
    println!("  Debug mode: {}", fmt_on_off(debug_mode));
    if let Some(snap) = snapshot {
        println!("  REPL clean: {}", fmt_on_off(snap.config.repl_clean_mode));
        println!("  Autonomy: {}%", snap.config.autonomy_level);
    }

    // === RPG / XP ===
    if let Some((xp, title)) = load_profile_xp() {
        println!();
        println!("{}RPG{}", colors::BOLD, colors::RESET);
        println!("  XP: {}", xp);
        println!("  Title: {}", title);
    }

    // Open tickets
    if let Ok(open) = TicketTracker::for_user().open_tickets() {
        if !open.is_empty() {
            println!();
            println!("{}Tickets{}", colors::BOLD, colors::RESET);
            println!("  {} open", open.len());
        }
    }

    // === DEBUG DETAIL ===
    if show_debug {
        println!();
        println!("{}[debug]{}", colors::DIM, colors::RESET);

        let ram_gb = status.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        println!(
            "  CPU: {} ({} cores)",
            status.hardware.cpu_model, status.hardware.cpu_cores
        );
        println!("  RAM: {:.1} GB", ram_gb);
        if let Some(gpu) = &status.hardware.gpu {
            let vram_gb = gpu.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            println!("  GPU: {} ({:.1} GB)", gpu.model, vram_gb);
        }

        if let Some(lat) = &status.latency {
            if lat.sample_count > 0 {
                if let Some(avg) = lat.total_avg_ms {
                    println!("  Avg response: {}ms ({} samples)", avg, lat.sample_count);
                }
                if let Some(p95) = lat.total_p95_ms {
                    println!("  P95 response: {}ms", p95);
                }
            }
        }

        let active_teams: Vec<_> = status
            .teams
            .teams
            .iter()
            .filter(|t| t.active)
            .map(|t| t.team.to_string())
            .collect();
        if !active_teams.is_empty() {
            println!("  Teams: {}", active_teams.join(", "));
        }

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

fn format_optional_dt(dt: Option<&DateTime<Utc>>) -> String {
    dt.map(|d| format_local_time(d))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_ts(ts: Option<u64>) -> String {
    ts.and_then(|t| Utc.timestamp_opt(t as i64, 0).single())
        .map(|d| format_local_time(&d))
        .unwrap_or_else(|| "-".to_string())
}

#[allow(dead_code)]
fn format_local_time(dt: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M").to_string()
}

fn fmt_on_off(val: bool) -> String {
    if val {
        format!("{}on{}", colors::OK, colors::RESET)
    } else {
        format!("{}off{}", colors::DIM, colors::RESET)
    }
}

fn fmt_ok_warn(val: bool) -> String {
    if val {
        format!("{}ok{}", colors::OK, colors::RESET)
    } else {
        format!("{}check{}", colors::WARN, colors::RESET)
    }
}

fn load_profile_xp() -> Option<(u64, String)> {
    let log = EventLog::new(EventLog::default_path(), 5000);
    let agg = log.aggregate().ok()?;
    if agg.total_requests == 0 {
        return None;
    }
    Some((agg.xp, agg.title))
}
