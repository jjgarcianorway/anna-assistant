//! Display helpers for annactl UI.

use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::stats::GlobalStats;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::ui::{colors, symbols, HR};
use anna_shared::VERSION;
use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

use crate::client::AnnadClient;

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

/// Print REPL header with optional telemetry (v0.0.34: relevant-only, no full report)
pub fn print_repl_header() {
    use anna_shared::telemetry::TelemetrySnapshot;

    println!();
    println!("{}annactl v{}{}", colors::HEADER, VERSION, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!("Anna is a local Linux service desk living on your machine.");

    // v0.0.34: Show only relevant deltas - errors, failed units, boot changes
    let telemetry = TelemetrySnapshot::collect();
    let mut alerts = Vec::new();

    // Check for failed units (fast, no LLM)
    if let Ok(output) = std::process::Command::new("systemctl")
        .args(["--failed", "--no-pager", "-q"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let failed_count = stdout.lines()
                .filter(|l| l.contains(".service") || l.contains(".mount"))
                .count();
            if failed_count > 0 {
                alerts.push(format!("{}{}failed units{}", colors::WARN, failed_count, colors::RESET));
            }
        }
    }

    // Check for recent errors this boot (journalctl -p 3 count, fast)
    if let Ok(output) = std::process::Command::new("journalctl")
        .args(["-p", "3", "-b", "--no-pager", "-q"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error_count = stdout.lines().count();
            if error_count > 0 {
                alerts.push(format!("{} journal errors", error_count));
            }
        }
    }

    // Boot time delta from telemetry
    if let Some(delta_ms) = telemetry.boot_delta_ms {
        let delta_secs = delta_ms.abs() as f64 / 1000.0;
        if delta_ms.abs() > 500 { // Only show if > 0.5s change
            if delta_ms < 0 {
                alerts.push(format!("boot {:.1}s faster", delta_secs));
            } else {
                alerts.push(format!("boot {:.1}s slower", delta_secs));
            }
        }
    }

    // Show alerts or "all clear"
    if !alerts.is_empty() {
        println!("{}[status]{} {}", colors::DIM, colors::RESET, alerts.join(" | "));
    } else {
        println!("{}[status]{} {}{} no issues{}", colors::DIM, colors::RESET, colors::OK, symbols::OK, colors::RESET);
    }

    println!(
        "Public commands: {}annactl{} | {}annactl <request>{} | {}annactl status{} | {}annactl -V{} | {}annactl uninstall{}",
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET,
        colors::BOLD, colors::RESET
    );
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}

/// Show bootstrap progress with live updates
pub async fn show_bootstrap_progress() -> Result<()> {
    println!();
    println!("{}anna (bootstrap){}", colors::HEADER, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
    println!(
        "{}Hello!{} I'm setting up my environment. Come back soon! ;)",
        colors::CYAN,
        colors::RESET
    );
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
                    "{}{}{}  All set! Anna is ready.",
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

/// Print a progress event in debug mode
pub fn print_progress_event(event: &ProgressEvent) {
    let elapsed = format!("{:.1}s", event.elapsed_ms as f64 / 1000.0);

    match &event.event {
        ProgressEventType::Starting { timeout_secs } => {
            println!(
                "{}[anna->{}]{} starting (timeout {}s) [{}]",
                colors::DIM,
                event.stage,
                colors::RESET,
                timeout_secs,
                elapsed
            );
        }
        ProgressEventType::Complete => {
            println!(
                "{}[anna]{} {} {}complete{} [{}]",
                colors::DIM,
                colors::RESET,
                event.stage,
                colors::OK,
                colors::RESET,
                elapsed
            );
        }
        ProgressEventType::Timeout => {
            println!(
                "{}[anna]{} {} {}TIMEOUT{} [{}]",
                colors::DIM,
                colors::RESET,
                event.stage,
                colors::ERR,
                colors::RESET,
                elapsed
            );
        }
        ProgressEventType::Error { message } => {
            println!(
                "{}[anna]{} {} {}error:{} {} [{}]",
                colors::DIM,
                colors::RESET,
                event.stage,
                colors::ERR,
                colors::RESET,
                message,
                elapsed
            );
        }
        ProgressEventType::Heartbeat => {
            let detail = event.detail.as_deref().unwrap_or("working");
            println!(
                "{}[anna]{} still working: {} [{}]",
                colors::DIM,
                colors::RESET,
                detail,
                elapsed
            );
        }
        ProgressEventType::ProbeRunning { probe_id } => {
            println!(
                "{}[anna->probe]{} running {} [{}]",
                colors::DIM,
                colors::RESET,
                probe_id,
                elapsed
            );
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
            println!(
                "{}[anna]{} probe {} {} ({}ms) [{}]",
                colors::DIM,
                colors::RESET,
                probe_id,
                status,
                timing_ms,
                elapsed
            );
        }
    }
}

/// Print stats display (v0.0.27)
pub fn print_stats_display(stats: &GlobalStats) {
    println!("\n{}annactl stats v{}{}", colors::HEADER, VERSION, colors::RESET);
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    let kw = 15; // key width

    // Global summary
    print_kv("total_requests", &stats.total_requests.to_string(), kw);
    print_kv(
        "success_rate",
        &format!("{:.1}%", stats.overall_success_rate() * 100.0),
        kw,
    );
    print_kv(
        "avg_reliability",
        &format!("{:.0}", stats.overall_avg_score()),
        kw,
    );

    if let Some(team) = stats.most_consulted_team {
        print_kv("top_team", &team.to_string(), kw);
    }

    println!();
    println!("{}Per-Team Statistics:{}", colors::BOLD, colors::RESET);
    println!(
        "  {:12} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "Team", "Total", "Success", "Failed", "AvgRnd", "AvgScore"
    );
    println!("{}{}{}", colors::DIM, "-".repeat(60), colors::RESET);

    for ts in &stats.by_team {
        if ts.tickets_total > 0 {
            let success_color = if ts.success_rate() >= 0.8 {
                colors::OK
            } else if ts.success_rate() >= 0.5 {
                colors::WARN
            } else {
                colors::ERR
            };
            println!(
                "  {:12} {:>8} {}{:>8}{} {:>8} {:>8.1} {:>8.0}",
                ts.team,
                ts.tickets_total,
                success_color,
                ts.tickets_verified,
                colors::RESET,
                ts.tickets_failed,
                ts.avg_rounds,
                ts.avg_reliability_score,
            );
        }
    }

    // Show teams with no activity
    let inactive: Vec<_> = stats.by_team.iter().filter(|ts| ts.tickets_total == 0).collect();
    if !inactive.is_empty() {
        println!();
        println!(
            "{}Inactive teams:{} {}",
            colors::DIM,
            colors::RESET,
            inactive.iter().map(|t| t.team.to_string()).collect::<Vec<_>>().join(", ")
        );
    }

    println!("{}{}{}", colors::DIM, HR, colors::RESET);
    println!();
}
