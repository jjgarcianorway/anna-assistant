//! Display helpers for annactl UI.
//! v0.0.67: Service Desk narrative renderer integration.
//! v0.0.69: "Since last time" narrative summary.
//! v0.0.70: Fixed status output contract with proper version display.
//! v0.0.71: Version truth - shows installed (annactl), daemon (annad), available.
//! v0.0.72: Restructured status with factual sections, REPL greeting baseline.

use anna_shared::snapshot::{self, DeltaItem, SystemSnapshot};
use anna_shared::status::{DaemonStatus, LlmState, UpdateCheckState};
use anna_shared::ui::{colors, HR};
use anna_shared::VERSION;
use chrono::{DateTime, Local, Utc};

// Re-export from dedicated modules
pub use crate::progress_display::{print_progress_event, show_bootstrap_progress};
pub use crate::stats_display::print_stats_display;

/// RPC protocol version (for status display)
const RPC_PROTOCOL_VERSION: &str = "1";

/// Print status display
/// v0.0.72: Structured sections: daemon, version, update, system, llm, health
pub fn print_status_display(status: &DaemonStatus, show_debug: bool) {
    println!("\n{}annactl status{}", colors::HEADER, colors::RESET);
    println!("{}", HR);

    let kw = 18; // key width

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
    print_section("version");
    print_kv("annactl", VERSION, kw);
    let daemon_ver = &status.version;
    if daemon_ver == VERSION {
        print_kv("annad", daemon_ver, kw);
    } else {
        println!("{:width$} {} {}[mismatch]{}", "annad", daemon_ver, colors::ERR, colors::RESET, width = kw);
    }
    print_kv("protocol", RPC_PROTOCOL_VERSION, kw);

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
    print_section("health");
    if status.llm.state == LlmState::Ready && status.last_error.is_none() {
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

/// v0.0.72: Print REPL greeting baseline (movie-IT style, debug OFF only)
/// Uses only local facts: last interaction, boot delta, service warnings, health
pub fn print_repl_greeting() {
    use anna_shared::telemetry::TelemetrySnapshot;

    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    // Load last snapshot for interaction time
    let last_snapshot = snapshot::load_last_snapshot();
    let last_interaction_hours = last_snapshot.as_ref().and_then(|s| {
        if s.captured_at > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let hours = (now.saturating_sub(s.captured_at)) / 3600;
            Some(hours)
        } else {
            None
        }
    });

    // Collect current state
    let telemetry = TelemetrySnapshot::collect();
    let mut current_snapshot = SystemSnapshot::now();
    let mut failed_services = 0;

    // Check for failed units
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-pager", "-q"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(".service") || line.contains(".mount") {
                    failed_services += 1;
                    if let Some(unit) = line.split_whitespace().find(|p|
                        p.ends_with(".service") || p.ends_with(".mount")
                    ) {
                        current_snapshot.add_failed_service(unit);
                    }
                }
            }
        }
    }

    // Build greeting
    println!();
    println!("{}Anna Service Desk{}", colors::HEADER, colors::RESET);
    println!("{}", HR);

    // Greeting line with last interaction
    let greeting = match last_interaction_hours {
        Some(h) if h > 12 => format!("Hello {}. It's been about {} hours since we last spoke.", username, h),
        Some(_) => format!("Hello {}, welcome back.", username),
        None => format!("Hello {}, welcome to Anna.", username),
    };
    println!("{}", greeting);

    // Notable deltas (0-2 max)
    let mut deltas_shown = 0;

    // Boot time delta
    if let Some(boot_ms) = telemetry.boot_delta_ms {
        if boot_ms.abs() > 5000 {
            let secs = boot_ms.unsigned_abs() / 1000;
            if secs < 3600 {
                println!("System booted {} minutes ago.", secs / 60);
                deltas_shown += 1;
            }
        }
    }

    // Service warnings
    if deltas_shown < 2 && failed_services > 0 {
        println!("{} service{} currently in failed state.", failed_services, if failed_services == 1 { "" } else { "s" });
        deltas_shown += 1;
    }

    // Health status
    if deltas_shown < 2 && failed_services == 0 {
        println!("All services running normally.");
    }

    println!();
    println!("What can I do for you today?");
    println!();

    // Save snapshot for next time
    let _ = snapshot::save_snapshot(&current_snapshot);
}

/// Print REPL header (legacy, calls new greeting)
pub fn print_repl_header() {
    print_repl_greeting();
}

/// Format delta item without emojis (v0.0.69)
#[allow(dead_code)]
fn format_delta_plain(delta: &DeltaItem) -> String {
    match delta {
        DeltaItem::DiskWarning { mount, prev, curr } => {
            format!("{}[warn]{} Disk {} at {}% (was {}%)", colors::WARN, colors::RESET, mount, curr, prev)
        }
        DeltaItem::DiskCritical { mount, prev, curr } => {
            format!("{}[crit]{} Disk {} at {}% (was {}%)", colors::ERR, colors::RESET, mount, curr, prev)
        }
        DeltaItem::DiskIncreased { mount, prev, curr } => {
            format!("Disk {} increased to {}% (was {}%)", mount, curr, prev)
        }
        DeltaItem::NewFailedService { unit } => {
            format!("{}[fail]{} Service {} failed", colors::ERR, colors::RESET, unit)
        }
        DeltaItem::ServiceRecovered { unit } => {
            format!("{}[ok]{} Service {} recovered", colors::OK, colors::RESET, unit)
        }
        DeltaItem::MemoryHigh { prev_percent, curr_percent } => {
            format!("{}[warn]{} Memory at {}% (was {}%)", colors::WARN, colors::RESET, curr_percent, prev_percent)
        }
        DeltaItem::MemoryIncreased { prev_percent, curr_percent } => {
            format!("Memory increased to {}% (was {}%)", curr_percent, prev_percent)
        }
    }
}
