//! Display helpers for annactl UI.
//! v0.0.67: Service Desk narrative renderer integration.
//! v0.0.69: "Since last time" narrative summary.
//! v0.0.70: Fixed status output contract with proper version display.
//! v0.0.71: Version truth - shows installed (annactl), daemon (annad), available.
//! v0.0.72: Restructured status with factual sections, REPL greeting baseline.
//! v0.0.73: Single source of truth via version module, client/daemon mismatch warning.

use anna_shared::rpc::DaemonInfo;
use anna_shared::status::{DaemonStatus, LlmState, UpdateCheckState};
use anna_shared::ui::{colors, HR};
use anna_shared::version::{VersionInfo, PROTOCOL_VERSION, VERSION, GIT_SHA};
use chrono::{DateTime, Local, Utc};

// Re-export from dedicated modules
pub use crate::progress_display::{print_progress_event, show_bootstrap_progress};
pub use crate::stats_display::print_stats_display;

/// Print status display
/// v0.0.72: Structured sections: daemon, version, update, system, llm, health
/// v0.0.73: Uses daemon_info for accurate version comparison
#[allow(dead_code)]
pub fn print_status_display(status: &DaemonStatus, show_debug: bool) {
    // For backward compatibility, call with None for daemon_info
    print_status_display_with_daemon_info(status, None, show_debug);
}

/// v0.0.73: Print status with daemon info for accurate version comparison
pub fn print_status_display_with_daemon_info(
    status: &DaemonStatus,
    daemon_info: Option<&DaemonInfo>,
    show_debug: bool,
) {
    println!("\n{}annactl status{}", colors::HEADER, colors::RESET);
    println!("{}", HR);

    let kw = 18; // key width
    let client_version = VersionInfo::current();

    // === DAEMON SECTION ===
    print_section("daemon");
    let state_str = match status.state {
        anna_shared::status::DaemonState::Running => format!("{}RUNNING{}", colors::OK, colors::RESET),
        anna_shared::status::DaemonState::Starting => format!("{}STARTING{}", colors::WARN, colors::RESET),
        anna_shared::status::DaemonState::Error => format!("{}ERROR{}", colors::ERR, colors::RESET),
    };
    print_kv("state", &state_str, kw);
    if let Some(pid) = status.pid {
        print_kv("pid", &pid.to_string(), kw);
    }
    print_kv("uptime", &format_uptime(status.uptime_seconds), kw);
    print_kv("debug_mode", if status.debug_mode { "ON" } else { "OFF" }, kw);

    // === VERSION SECTION ===
    // v0.0.73: Show full version info with git SHA
    print_section("version");
    let client_display = if GIT_SHA != "unknown" {
        format!("{} ({})", VERSION, GIT_SHA)
    } else {
        VERSION.to_string()
    };
    print_kv("annactl", &client_display, kw);

    // Show daemon version from DaemonInfo if available, otherwise from status
    let (daemon_ver, daemon_sha, version_mismatch) = if let Some(info) = daemon_info {
        let mismatch = !client_version.matches(&info.version_info);
        let sha_str = if info.version_info.git_sha != "unknown" {
            format!(" ({})", info.version_info.git_sha)
        } else {
            String::new()
        };
        (info.version_info.version.clone(), sha_str, mismatch)
    } else {
        let mismatch = status.version != VERSION;
        (status.version.clone(), String::new(), mismatch)
    };

    if version_mismatch {
        println!(
            "  {:width$} {}{} {}[MISMATCH]{}",
            "annad", daemon_ver, daemon_sha, colors::ERR, colors::RESET, width = kw
        );
    } else {
        print_kv("annad", &format!("{}{}", daemon_ver, daemon_sha), kw);
    }

    print_kv("protocol", &PROTOCOL_VERSION.to_string(), kw);

    // === UPDATE SECTION ===
    print_section("update");
    let auto_str = if status.update.enabled {
        format!("{}ENABLED{}", colors::OK, colors::RESET)
    } else {
        format!("{}DISABLED{}", colors::WARN, colors::RESET)
    };
    print_kv("auto_update", &auto_str, kw);
    print_kv("check_pace", &format!("every {}s", status.update.check_interval_secs), kw);

    // v0.0.72: Show real timestamps in local time
    match &status.update.last_check_at {
        Some(dt) => print_kv("last_check_at", &format_local_time(dt), kw),
        None => print_kv("last_check_at", "never", kw),
    }
    match &status.update.next_check_at {
        Some(dt) => print_kv("next_check_at", &format_local_time(dt), kw),
        None => print_kv("next_check_at", "unknown", kw),
    }

    // v0.0.72: Show available_version with check state
    let available_str = match (&status.update.latest_version, &status.update.check_state) {
        (Some(v), UpdateCheckState::Success) if status.update.update_available => {
            format!("{}{}{} ({}update available{})", colors::OK, v, colors::RESET, colors::WARN, colors::RESET)
        }
        (Some(v), UpdateCheckState::Success) => v.clone(),
        (Some(v), UpdateCheckState::Failed) => {
            format!("{} ({}last check failed{})", v, colors::WARN, colors::RESET)
        }
        (None, UpdateCheckState::NeverChecked) => format!("{}unknown{}", colors::DIM, colors::RESET),
        (None, _) => format!("{}unknown{}", colors::DIM, colors::RESET),
        (Some(v), _) => v.clone(),
    };
    print_kv("available_version", &available_str, kw);

    // v0.0.72: Show when we last successfully got the version
    match &status.update.latest_checked_at {
        Some(dt) => print_kv("available_checked_at", &format_local_time(dt), kw),
        None => print_kv("available_checked_at", "never", kw),
    }

    // === SYSTEM SECTION ===
    print_section("system");
    let ram_gb = status.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    print_kv("cpu", &format!("{} ({} cores)", status.hardware.cpu_model, status.hardware.cpu_cores), kw);
    print_kv("ram", &format!("{:.1} GB", ram_gb), kw);
    if let Some(gpu) = &status.hardware.gpu {
        let vram_gb = gpu.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        print_kv("gpu", &format!("{} ({:.1} GB)", gpu.model, vram_gb), kw);
    } else {
        print_kv("gpu", "none", kw);
    }

    // === LLM SECTION ===
    print_section("llm");
    let llm_state_str = match status.llm.state {
        LlmState::Ready => format!("{}READY{}", colors::OK, colors::RESET),
        LlmState::Bootstrapping => format!("{}BOOTSTRAPPING{}", colors::WARN, colors::RESET),
        LlmState::Error => format!("{}ERROR{}", colors::ERR, colors::RESET),
    };
    print_kv("state", &llm_state_str, kw);
    print_kv("provider", &status.llm.provider, kw);

    if let Some(phase) = &status.llm.phase {
        print_kv("phase", phase, kw);
    }

    if let Some(progress) = &status.llm.progress {
        let bar = anna_shared::ui::progress_bar(progress.percent(), 30);
        let current = anna_shared::ui::format_bytes(progress.current_bytes);
        let total = anna_shared::ui::format_bytes(progress.total_bytes);
        println!("{:width$} {} {:.0}%", "progress", bar, progress.percent() * 100.0, width = kw);
        println!("{:width$} {} / {}", "downloaded", current, total, width = kw);
    }

    if !status.llm.models.is_empty() {
        for m in &status.llm.models {
            print_kv(&format!("model.{}", m.role), &m.name, kw);
        }
    }

    // === HEALTH SECTION ===
    // v0.0.73: Include version mismatch warning in health
    print_section("health");

    // Determine health status considering version mismatch
    let has_version_mismatch = if let Some(info) = daemon_info {
        !client_version.matches(&info.version_info)
    } else {
        status.version != VERSION
    };

    if has_version_mismatch {
        println!(
            "{:width$} {}WARN{}: client/daemon version mismatch",
            "status", colors::WARN, colors::RESET, width = kw
        );
    } else if status.llm.state == LlmState::Ready && status.last_error.is_none() {
        println!("{:width$} {}OK{}", "status", colors::OK, colors::RESET, width = kw);
    } else if let Some(err) = &status.last_error {
        println!("{:width$} {}ERROR{}: {}", "status", colors::ERR, colors::RESET, err, width = kw);
    } else {
        println!("{:width$} {}DEGRADED{}", "status", colors::WARN, colors::RESET, width = kw);
    }

    // Debug info - latency stats (only in debug mode)
    if show_debug {
        print_section("latency");
        if let Some(lat) = &status.latency {
            let fmt = |avg: Option<u64>, p95: Option<u64>| {
                match (avg, p95) {
                    (Some(a), Some(p)) => format!("avg {}ms, p95 {}ms", a, p),
                    _ => "no data".to_string(),
                }
            };
            print_kv("translator", &fmt(lat.translator_avg_ms, lat.translator_p95_ms), kw);
            print_kv("probes", &fmt(lat.probes_avg_ms, lat.probes_p95_ms), kw);
            print_kv("specialist", &fmt(lat.specialist_avg_ms, lat.specialist_p95_ms), kw);
            print_kv("total", &fmt(lat.total_avg_ms, lat.total_p95_ms), kw);
            print_kv("samples", &lat.sample_count.to_string(), kw);
        } else {
            println!("  No latency data yet");
        }

        print_section("teams");
        for t in &status.teams.teams {
            let state = if t.active {
                format!("{}active{}", colors::OK, colors::RESET)
            } else {
                format!("{}inactive{}", colors::DIM, colors::RESET)
            };
            print_kv(&format!("{}", t.team), &state, kw);
        }
    }

    println!("{}", HR);
}

fn print_section(name: &str) {
    println!("\n{}[{}]{}", colors::BOLD, name, colors::RESET);
}

fn print_kv(key: &str, value: &str, width: usize) {
    println!("  {:width$} {}", key, value, width = width);
}

fn format_uptime(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
}

fn format_local_time(dt: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}
