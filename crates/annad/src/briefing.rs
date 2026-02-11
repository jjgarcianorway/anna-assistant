//! Morning briefing generation - LLM analyzes telemetry data.
//! v0.3.156: Replaced hardcoded parsing with LLM analysis.

use anyhow::Result;
use std::process::Command;
use tracing::{info, warn};

/// Collect all system telemetry for LLM analysis.
/// Returns raw command outputs - NO parsing, NO hardcoding.
/// v0.3.156: Respects user preferences for which sections to include.
fn collect_system_telemetry() -> String {
    use anna_shared::preferences::UserPreferences;

    let prefs = UserPreferences::load();
    let mut telemetry = String::new();

    telemetry.push_str("=== SYSTEM TELEMETRY (24h) ===\n\n");

    // 1. Package updates available
    if prefs.briefing.updates {
        telemetry.push_str("## Updates Available:\n");
    if let Ok(output) = Command::new("checkupdates").output() {
        let updates = String::from_utf8_lossy(&output.stdout);
        if updates.is_empty() {
            telemetry.push_str("No updates available.\n");
        } else {
            telemetry.push_str(&updates);
        }
    } else {
        telemetry.push_str("Could not check for updates.\n");
    }
        telemetry.push_str("\n");
    }

    // 2. System errors (last 24h)
    if prefs.briefing.errors {
        telemetry.push_str("## System Errors (priority: err):\n");
    if let Ok(output) = Command::new("journalctl")
        .args(["--since", "24 hours ago", "-p", "err", "--no-pager", "-q", "-n", "30"])
        .output()
    {
        let errors = String::from_utf8_lossy(&output.stdout);
        if errors.trim().is_empty() {
            telemetry.push_str("No errors logged.\n");
        } else {
            telemetry.push_str(&errors);
        }
    } else {
        telemetry.push_str("Could not read error logs.\n");
    }
        telemetry.push_str("\n");
    }

    // 3. Failed services
    if prefs.briefing.services {
        telemetry.push_str("## Failed Services:\n");
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        let failed = String::from_utf8_lossy(&output.stdout);
        if failed.trim().is_empty() {
            telemetry.push_str("No failed services.\n");
        } else {
            telemetry.push_str(&failed);
        }
    } else {
        telemetry.push_str("Could not check services.\n");
    }
        telemetry.push_str("\n");
    }

    // 4. Disk usage
    if prefs.briefing.disk {
        telemetry.push_str("## Disk Usage:\n");
    if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
        telemetry.push_str(&String::from_utf8_lossy(&output.stdout));
    } else {
        telemetry.push_str("Could not check disk.\n");
    }
        telemetry.push_str("\n");
    }

    // 5. Memory usage
    if prefs.briefing.memory {
        telemetry.push_str("## Memory Usage:\n");
    if let Ok(output) = Command::new("free").args(["-h"]).output() {
        telemetry.push_str(&String::from_utf8_lossy(&output.stdout));
    } else {
        telemetry.push_str("Could not check memory.\n");
    }
        telemetry.push_str("\n");
    }

    // 6. System load
    if prefs.briefing.load {
        telemetry.push_str("## Load Average:\n");
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        telemetry.push_str(&load);
    } else {
        telemetry.push_str("Could not read load.\n");
    }
        telemetry.push_str("\n");
    }

    // 7. Security events (failed auth)
    if prefs.briefing.security {
        telemetry.push_str("## Security Events (failed auth):\n");
    if let Ok(output) = Command::new("journalctl")
        .args(["--since", "24 hours ago", "-p", "warning", "-u", "sshd", "-u", "sudo", "--no-pager", "-q", "-n", "20"])
        .output()
    {
        let logs = String::from_utf8_lossy(&output.stdout);
        if logs.trim().is_empty() {
            telemetry.push_str("No failed authentication attempts.\n");
        } else {
            telemetry.push_str(&logs);
        }
    } else {
        telemetry.push_str("Could not check security logs.\n");
    }
        telemetry.push_str("\n");
    }

    // 8. Recent package installations (last 7 days)
    if prefs.briefing.package_changes {
        telemetry.push_str("## Recent Package Changes (7 days):\n");
    if let Ok(output) = Command::new("grep")
        .args(["-E", "installed|upgraded|removed", "/var/log/pacman.log"])
        .output()
    {
        let log = String::from_utf8_lossy(&output.stdout);
        let recent: Vec<&str> = log.lines()
            .filter(|l| {
                // Filter to last 7 days
                if let Some(date_str) = l.split('[').nth(1) {
                    if let Some(date) = date_str.split(']').next() {
                        if let Ok(timestamp) = chrono::NaiveDateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%z") {
                            let now = chrono::Utc::now().naive_utc();
                            let diff = now.signed_duration_since(timestamp);
                            return diff.num_days() <= 7;
                        }
                    }
                }
                false
            })
            .take(15)
            .collect();

        if recent.is_empty() {
            telemetry.push_str("No package changes in the last 7 days.\n");
        } else {
            for line in recent {
                telemetry.push_str(line);
                telemetry.push('\n');
            }
        }
    } else {
        telemetry.push_str("Could not read pacman log.\n");
    }
        telemetry.push_str("\n");
    }

    // 9. Anomaly data (if available)
    if prefs.briefing.anomalies {
        telemetry.push_str("## Anomaly Detection:\n");
    let store = crate::anomaly::AnomalyStore::load();
    let anomalies: Vec<_> = store.metrics.values()
        .filter_map(|h| {
            if let Some(ref baseline) = h.baseline {
                if let Some(sample) = h.samples.last() {
                    if baseline.is_anomaly(sample.value) {
                        return Some(format!("{}: {:.1}{} (normal: {:.1} ± {:.1})",
                            h.name, sample.value, h.unit, baseline.mean, baseline.std_dev));
                    }
                }
            }
            None
        })
        .collect();

    if anomalies.is_empty() {
        telemetry.push_str("All metrics within normal ranges.\n");
    } else {
        for anomaly in anomalies {
            telemetry.push_str(&anomaly);
            telemetry.push('\n');
        }
    }
        telemetry.push_str("\n");
    }

    // 10. Visual trends (ASCII charts for last 7 days)
    if prefs.briefing.charts {
        telemetry.push_str("## Trend Charts (7 days):\n");

    // Disk usage trend (from daily snapshots)
    let history = anna_shared::monitor::LongTermHistory::load();
    if !history.daily_snapshots.is_empty() {
        use anna_shared::charts::Sparkline;

        // Last 7 days of disk usage
        let disk_trend: Vec<f64> = history.daily_snapshots.iter()
            .rev()
            .take(7)
            .rev()
            .map(|s| s.disk_used_gb as f64)
            .collect();

        if !disk_trend.is_empty() {
            let sparkline = Sparkline::new(&disk_trend);
            telemetry.push_str(&format!("Disk (GB):    {} ({:.1} → {:.1} GB)\n",
                sparkline.render(),
                disk_trend.first().unwrap_or(&0.0),
                disk_trend.last().unwrap_or(&0.0)));
        }

        // Memory usage trend
        let mem_trend: Vec<f64> = history.daily_snapshots.iter()
            .rev()
            .take(7)
            .rev()
            .map(|s| s.avg_memory_pct as f64)
            .collect();

        if !mem_trend.is_empty() {
            let sparkline = Sparkline::new(&mem_trend);
            telemetry.push_str(&format!("Memory (%):   {} ({:.1}% → {:.1}%)\n",
                sparkline.render(),
                mem_trend.first().unwrap_or(&0.0),
                mem_trend.last().unwrap_or(&0.0)));
        }

        // Boot time trend
        let boot_trend: Vec<f64> = history.daily_snapshots.iter()
            .rev()
            .take(7)
            .rev()
            .map(|s| s.avg_boot_time as f64)
            .collect();

        if !boot_trend.is_empty() {
            let sparkline = Sparkline::new(&boot_trend);
            telemetry.push_str(&format!("Boot (sec):   {} ({:.1}s → {:.1}s)\n",
                sparkline.render(),
                boot_trend.first().unwrap_or(&0.0),
                boot_trend.last().unwrap_or(&0.0)));
        }
        } else {
            telemetry.push_str("Not enough historical data yet (need 7 days).\n");
        }
    }

    telemetry
}

/// Generate morning briefing using LLM analysis.
/// v0.3.156: No hardcoding - LLM analyzes raw telemetry.
pub async fn generate_morning_briefing_llm(username: Option<&str>) -> Result<String> {
    use anna_shared::preferences::{UserPreferences, BriefingVerbosity};

    info!("Generating morning briefing with LLM analysis...");

    // Load preferences for verbosity
    let prefs = UserPreferences::load();

    // Collect all raw telemetry
    let telemetry = collect_system_telemetry();

    // Build prompt for LLM
    let user_greeting = if let Some(name) = username {
        format!("Good morning, {}!", name)
    } else {
        "Good morning!".to_string()
    };

    let (sentence_count, detail_level) = match prefs.briefing.verbosity {
        BriefingVerbosity::Brief => ("5-8", "brief"),
        BriefingVerbosity::Detailed => ("10-15", "detailed with technical specifics"),
    };

    let prompt = format!(
        r#"You are Anna, an AI system administrator. Generate a {} morning briefing for the user.

{}

Analyze the system telemetry below and create a natural, conversational briefing.

REQUIREMENTS:
1. Start with a personalized greeting (use the one provided above)
2. Summarize key points in plain language (not technical jargon unless necessary)
3. For errors: Show WHAT is failing (service names, error types), not just counts
4. For updates: Mention if there are security updates
5. For resources: Only mention if concerning (>85% disk, >90% memory, high load)
6. For trends: If sparkline charts show worrying patterns (▇▇▇ = rising), mention them
7. Be honest: If something needs attention, say so clearly
8. Length: {} sentences
9. End with a closing that reflects system health

STYLE:
- Conversational and friendly
- No bullet points or markdown formatting
- Natural paragraphs
- Use "your system" not "the system"
- Sparklines are visual: ▁▂▃▄▅▆▇█ (don't reproduce them, just describe trend if notable)

SYSTEM TELEMETRY:
{}

Generate the briefing now:"#,
        detail_level,
        user_greeting,
        sentence_count,
        telemetry
    );

    // Call LLM
    let model = std::env::var("ANNA_OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:14b".to_string());
    match crate::ollama::chat_with_timeout(&model, &prompt, 60).await {
        Ok(response) => {
            info!("Morning briefing generated successfully");
            Ok(response.trim().to_string())
        }
        Err(e) => {
            warn!("LLM failed to generate briefing: {}", e);
            // Fallback to simple summary
            Ok(format!(
                "{}  Your system is running. Check `annactl status` for details.",
                user_greeting
            ))
        }
    }
}

/// Legacy synchronous version (fallback).
pub fn generate_morning_briefing() -> String {
    // Run async version in blocking context
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(generate_morning_briefing_llm(None))
        .unwrap_or_else(|_| "Good morning! System status check failed.".to_string())
}
