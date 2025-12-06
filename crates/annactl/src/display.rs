//! Display helpers for annactl UI.
//! v0.0.67: Service Desk narrative renderer integration.

use anna_shared::render;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::ui::{colors, symbols, HR};
use anna_shared::VERSION;

// Re-export from dedicated modules
pub use crate::progress_display::{print_progress_event, show_bootstrap_progress};
pub use crate::stats_display::print_stats_display;

/// Print status display
pub fn print_status_display(status: &DaemonStatus, show_debug: bool) {
    println!("\n{}annactl v{}{}", colors::HEADER, status.version, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    let kw = 15; // key width

    // Daemon info
    print_kv(
        "daemon",
        &format!("{}   (pid {})", status.state, status.pid.unwrap_or(0)),
        kw,
    );
    print_kv(
        "debug_mode",
        if status.debug_mode { "ON" } else { "OFF" },
        kw,
    );
    println!();

    // Version/Update section
    print_kv("version", &status.version, kw);

    // Show available version from GitHub
    let available = status
        .update
        .available_version
        .as_deref()
        .unwrap_or("checking...");
    if status.update.update_available {
        println!(
            "{:width$} {}{}{} ({}update available{})",
            "available",
            colors::OK,
            available,
            colors::RESET,
            colors::WARN,
            colors::RESET,
            width = kw
        );
    } else {
        print_kv("available", available, kw);
    }

    // Update check pace
    print_kv(
        "check_pace",
        &format!("every {}s", status.update.check_interval_secs),
        kw,
    );

    // Countdown to next check
    if let Some(next) = &status.update.next_check {
        let now = chrono::Utc::now();
        let remaining = next.signed_duration_since(now);
        let secs = remaining.num_seconds().max(0);
        print_kv("next_check", &format!("in {}s", secs), kw);
    }

    // Auto-update status
    let auto_str = if status.update.enabled {
        "ENABLED"
    } else {
        "DISABLED"
    };
    print_kv("auto_update", auto_str, kw);
    println!();

    // Hardware info
    let ram_gb = status.hardware.ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    print_kv(
        "cpu",
        &format!(
            "{} ({} cores)",
            status.hardware.cpu_model, status.hardware.cpu_cores
        ),
        kw,
    );
    print_kv("ram", &format!("{:.1} GB", ram_gb), kw);

    if let Some(gpu) = &status.hardware.gpu {
        let vram_gb = gpu.vram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        print_kv(
            "gpu",
            &format!("{} ({:.1} GB VRAM)", gpu.model, vram_gb),
            kw,
        );
    } else {
        print_kv("gpu", "none", kw);
    }
    println!();

    // LLM info
    let llm_state_color = match status.llm.state {
        LlmState::Ready => colors::OK,
        LlmState::Bootstrapping => colors::WARN,
        LlmState::Error => colors::ERR,
    };
    println!(
        "{:width$} {}{}{}{}",
        "llm",
        llm_state_color,
        status.llm.state,
        if status.llm.state == LlmState::Bootstrapping {
            " (required)"
        } else {
            ""
        },
        colors::RESET,
        width = kw
    );
    print_kv("provider", &status.llm.provider, kw);

    if let Some(phase) = &status.llm.phase {
        print_kv("phase", phase, kw);
    }

    if let Some(progress) = &status.llm.progress {
        let bar = anna_shared::ui::progress_bar(progress.percent(), 30);
        let current = anna_shared::ui::format_bytes(progress.current_bytes);
        let total = anna_shared::ui::format_bytes(progress.total_bytes);
        let speed = anna_shared::ui::format_bytes(progress.speed_bytes_per_sec);
        let eta = anna_shared::ui::format_duration(progress.eta_seconds);
        println!(
            "{:width$} {} {:.0}%",
            "progress",
            bar,
            progress.percent() * 100.0,
            width = kw
        );
        println!(
            "{:width$} {} / {}   {}/s   eta {}",
            "traffic",
            current,
            total,
            speed,
            eta,
            width = kw
        );
    }

    if !status.llm.models.is_empty() {
        let models_str = status
            .llm
            .models
            .iter()
            .map(|m| format!("{}: {}", m.role, m.name))
            .collect::<Vec<_>>()
            .join("\n               ");
        print_kv("models", &models_str, kw);
    }

    if let Some(err) = &status.last_error {
        println!(
            "{:width$} {}{}{}",
            "last_error",
            colors::ERR,
            err,
            colors::RESET,
            width = kw
        );
    } else {
        print_kv("last_error", "none", kw);
    }

    println!();
    if status.llm.state == LlmState::Ready {
        print_kv("health", &format!("{}OK{}", colors::OK, colors::RESET), kw);
    }

    // Debug info - latency stats
    if show_debug {
        println!();
        println!("{}Latency Stats (last 20 requests):{}", colors::BOLD, colors::RESET);
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
            println!("  No latency data yet (run some requests first)");
        }

        // Team roster (v0.0.25)
        println!("\n{}Teams Active:{}", colors::BOLD, colors::RESET);
        for t in &status.teams.teams {
            let (icon, model) = if t.active {
                (format!("{}{}{}", colors::OK, symbols::OK, colors::RESET), format!("[{}]", t.junior_model))
            } else {
                (format!("{}-{}", colors::DIM, colors::RESET), "[inactive]".to_string())
            };
            println!("  {:12} {} {}", t.team, model, icon);
        }
    }

    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}

fn print_kv(key: &str, value: &str, width: usize) {
    println!("{:width$} {}", key, value, width = width);
}

/// Print REPL header with narrative greeting (v0.0.67: Service Desk Theatre)
pub fn print_repl_header() {
    use anna_shared::telemetry::TelemetrySnapshot;

    // Get hostname and username for narrative header
    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
        .unwrap_or_else(|_| "localhost".to_string());
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    // Render header block (v0.0.67)
    render::render_header(&hostname, &username, VERSION, false);

    // Collect system status for greeting
    let telemetry = TelemetrySnapshot::collect();
    let mut critical_issues = 0;

    // Check for failed units (fast, no LLM)
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-pager", "-q"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            critical_issues += stdout.lines()
                .filter(|l| l.contains(".service") || l.contains(".mount"))
                .count();
        }
    }

    // Get boot time delta for narrative
    let boot_time_delta = telemetry.boot_delta_ms.map(|ms| {
        let secs = ms.unsigned_abs() / 1000;
        if secs < 60 {
            format!("{}s", secs)
        } else {
            format!("{}m {}s", secs / 60, secs % 60)
        }
    });

    // Get last interaction time (TODO: integrate with stats store)
    let last_interaction = None; // Will be populated from stats

    // Render narrative greeting (v0.0.67)
    render::render_greeting(
        &username,
        last_interaction,
        boot_time_delta.as_deref(),
        critical_issues,
    );

    // Show help hint
    println!(
        "{}Type a question, or: status, help, exit{}",
        colors::DIM,
        colors::RESET
    );
    println!();
}


