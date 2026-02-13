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

    // 10. Predictive Alerts (disk full prediction, memory leaks, boot degradation)
    telemetry.push_str("## Predictive Alerts:\n");
    let history = anna_shared::monitor::LongTermHistory::load();
    if !history.daily_snapshots.is_empty() {
        use anna_shared::prediction::{generate_predictive_alerts, AlertInput, AlertSeverity};

        // Build historical data for prediction
        let alerts_input = AlertInput {
            disk_usage: history.daily_snapshots.iter()
                .rev()
                .take(30)
                .rev()
                .map(|s| {
                    // Convert GB used to percentage if we have total disk size
                    // For now, use a rough estimate - actual percentage would need total disk size
                    let total_gb = 200.0f64; // Rough estimate, could be improved
                    (s.disk_used_gb as f64 / total_gb) * 100.0
                })
                .collect(),
            memory_usage: history.daily_snapshots.iter()
                .rev()
                .take(30)
                .rev()
                .map(|s| s.avg_memory_pct as f64)
                .collect(),
            boot_times: history.daily_snapshots.iter()
                .rev()
                .take(30)
                .rev()
                .map(|s| s.avg_boot_time as f64)
                .collect(),
        };

        let predictive_alerts = generate_predictive_alerts(&alerts_input);

        if predictive_alerts.is_empty() {
            telemetry.push_str("No predictive alerts - all metrics stable.\n");
        } else {
            for alert in predictive_alerts {
                let severity_tag = match alert.severity {
                    AlertSeverity::Critical => "[CRITICAL]",
                    AlertSeverity::Warning => "[WARNING]",
                    AlertSeverity::Info => "[INFO]",
                };
                telemetry.push_str(&format!("{} {}: {}\n",
                    severity_tag, alert.title, alert.description));
                telemetry.push_str(&format!("   → Recommendation: {}\n", alert.recommendation));
            }
        }
    } else {
        telemetry.push_str("Not enough historical data for predictions (need 7+ days).\n");
    }
    telemetry.push_str("\n");

    // 11. Historical Comparison (vs 30-day average)
    if !history.daily_snapshots.is_empty() && history.daily_snapshots.len() >= 7 {
        telemetry.push_str("## Historical Comparison (vs 30-day average):\n");

        let averages = history.calculate_averages();
        if let Some(current) = history.daily_snapshots.back() {
            let mut comparisons = Vec::new();

            // Disk comparison
            let disk_diff_pct = ((current.disk_used_gb - averages.disk_gb) / averages.disk_gb) * 100.0;
            if disk_diff_pct.abs() > 10.0 {
                let direction = if disk_diff_pct > 0.0 { "above" } else { "below" };
                comparisons.push(format!(
                    "Disk: {:.1}GB ({:.0}% {} average {:.1}GB)",
                    current.disk_used_gb, disk_diff_pct.abs(), direction, averages.disk_gb
                ));
            }

            // Memory comparison
            let mem_diff_pct = ((current.avg_memory_pct - averages.memory_pct) / averages.memory_pct) * 100.0;
            if mem_diff_pct.abs() > 15.0 {
                let direction = if mem_diff_pct > 0.0 { "above" } else { "below" };
                comparisons.push(format!(
                    "Memory: {:.1}% ({:.0}% {} average {:.1}%)",
                    current.avg_memory_pct, mem_diff_pct.abs(), direction, averages.memory_pct
                ));
            }

            // Boot time comparison
            let boot_diff_pct = ((current.avg_boot_time - averages.boot_time_sec) / averages.boot_time_sec) * 100.0;
            if boot_diff_pct.abs() > 20.0 {
                let direction = if boot_diff_pct > 0.0 { "slower than" } else { "faster than" };
                comparisons.push(format!(
                    "Boot: {:.1}s ({:.0}% {} average {:.1}s)",
                    current.avg_boot_time, boot_diff_pct.abs(), direction, averages.boot_time_sec
                ));
            }

            // Load comparison
            let load_diff_pct = if averages.load_avg > 0.0 {
                ((current.avg_load - averages.load_avg) / averages.load_avg) * 100.0
            } else {
                0.0
            };
            if load_diff_pct.abs() > 30.0 {
                let direction = if load_diff_pct > 0.0 { "above" } else { "below" };
                comparisons.push(format!(
                    "Load: {:.2} ({:.0}% {} average {:.2})",
                    current.avg_load, load_diff_pct.abs(), direction, averages.load_avg
                ));
            }

            if comparisons.is_empty() {
                telemetry.push_str("All metrics within normal range (±10-30% of average).\n");
            } else {
                for comparison in comparisons {
                    telemetry.push_str(&format!("• {}\n", comparison));
                }
            }
        }
        telemetry.push_str("\n");
    }

    // 12. Visual trends (ASCII charts for last 7 days)
    if prefs.briefing.charts {
        telemetry.push_str("## Trend Charts (7 days):\n");

    // Already loaded history above
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

    // v0.3.167: Regression Detection (performance degradations)
    telemetry.push_str("## Regression Detection:\n");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let regressions = rt.block_on(async {
        crate::regression_detector::detect_regressions().await.unwrap_or_default()
    });

    if regressions.is_empty() {
        telemetry.push_str("No performance regressions detected.\n");
    } else {
        for regression in &regressions {
            telemetry.push_str(&crate::regression_detector::format_regression(regression));
            telemetry.push('\n');
        }
    }
    telemetry.push_str("\n");

    // v0.3.167: Enhanced Predictive Maintenance (health forecast)
    telemetry.push_str("## Health Forecast:\n");
    let health_forecast = rt.block_on(async {
        crate::predictive_maintenance::generate_health_forecast().await.unwrap_or_else(|_| {
            crate::predictive_maintenance::HealthForecast {
                predictions: vec![],
                overall_health_score: 95.0,
                trends_summary: "Unable to generate forecast.".to_string(),
            }
        })
    });

    telemetry.push_str(&crate::predictive_maintenance::format_health_forecast(&health_forecast));
    telemetry.push_str("\n");

    // v0.3.167: Cleanup Opportunities (when disk >75%)
    let disk_pct = get_disk_usage_percentage();
    if disk_pct > 75.0 {
        telemetry.push_str("## Cleanup Opportunities:\n");
        let cleanup_analysis = rt.block_on(async {
            crate::cleanup_detector::scan_for_cleanable_space().await.unwrap_or_else(|_| {
                crate::cleanup_detector::CleanupAnalysis {
                    total_cleanable_mb: 0.0,
                    items: vec![],
                    recommendations: vec![],
                }
            })
        });

        if cleanup_analysis.total_cleanable_mb > 100.0 {
            telemetry.push_str(&crate::cleanup_detector::format_cleanup_analysis(&cleanup_analysis));
        } else {
            telemetry.push_str("No significant cleanup opportunities found.\n");
        }
        telemetry.push_str("\n");
    }

    // v0.3.168: Cross-Module Intelligence (connecting the dots)
    telemetry.push_str("## Cross-Module Intelligence:\n");
    let insights = rt.block_on(async {
        crate::cross_module_intelligence::synthesize_insights(None).await.unwrap_or_default()
    });

    if insights.is_empty() {
        telemetry.push_str("No cross-module insights at this time.\n");
    } else {
        telemetry.push_str(&crate::cross_module_intelligence::format_insights(&insights));
    }
    telemetry.push_str("\n");

    // v0.3.186: Power, Battery, GPU, XWayland, and Learned Insights
    let power_section = crate::power_profile::power_telemetry();
    if !power_section.is_empty() {
        telemetry.push_str(&power_section);
        telemetry.push_str("\n");
    }

    let battery_section = crate::battery::battery_telemetry();
    if !battery_section.is_empty() {
        telemetry.push_str(&battery_section);
        telemetry.push_str("\n");
    }

    let gpu_section = crate::gpu_monitor::gpu_telemetry();
    if !gpu_section.is_empty() {
        telemetry.push_str(&gpu_section);
        telemetry.push_str("\n");
    }

    let xwayland_section = crate::xwayland::xwayland_telemetry();
    if !xwayland_section.is_empty() {
        telemetry.push_str(&xwayland_section);
        telemetry.push_str("\n");
    }

    let insights_section = crate::system_learner::insights_for_briefing();
    if !insights_section.is_empty() {
        telemetry.push_str(&insights_section);
        telemetry.push_str("\n");
    }

    let pkg_section = crate::pkg_suggestions::pending_suggestions_briefing();
    if !pkg_section.is_empty() {
        telemetry.push_str(&pkg_section);
        telemetry.push_str("\n");
    }

    // Anna's autonomous activities in past 24h
    let personality = crate::personality::PersonalityState::load();
    if !personality.learned_lessons.is_empty() {
        telemetry.push_str("\n## Anna's Recent Activities:\n");

        // Show most recent lessons (last 5)
        let recent_lessons = personality.learned_lessons.iter().rev().take(5);
        for (i, lesson) in recent_lessons.enumerate() {
            if i < 3 { // Only show last 3 in briefing to keep it concise
                telemetry.push_str(&format!("• {}\n", lesson));
            }
        }

        // Add auto-healing summary
        let healing_summary = crate::autohealing::get_healing_summary();
        if !healing_summary.is_empty() && !healing_summary.contains("No auto-healing") {
            telemetry.push_str("\nAuto-healing actions:\n");
            telemetry.push_str(&healing_summary);
        }
    }

    telemetry
}

/// Quick disk usage check for mood determination
pub fn get_disk_usage_percentage() -> f32 {
    std::process::Command::new("df")
        .args(["--output=pcent", "/"])
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.lines()
                .nth(1)
                .and_then(|l| l.trim().trim_end_matches('%').parse::<f32>().ok())
        })
        .unwrap_or(0.0)
}

/// Quick memory usage check for mood determination
fn get_memory_usage_percentage() -> f32 {
    std::process::Command::new("free")
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // Parse "Mem:" line: total, used, free
            for line in out.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let total: f32 = parts[1].parse().ok()?;
                        let used: f32 = parts[2].parse().ok()?;
                        return Some((used / total) * 100.0);
                    }
                }
            }
            None
        })
        .unwrap_or(0.0)
}

/// Quick failed services count for mood determination
fn count_failed_services() -> usize {
    std::process::Command::new("systemctl")
        .args(["list-units", "--state=failed", "--no-legend"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0)
}

/// Generate morning briefing using LLM analysis.
/// v0.3.156: No hardcoding - LLM analyzes raw telemetry.
/// v0.3.157: Added personality - dynamic greetings based on mood and time.
pub async fn generate_morning_briefing_llm(username: Option<&str>) -> Result<String> {
    use anna_shared::preferences::{UserPreferences, BriefingVerbosity};

    info!("Generating morning briefing with LLM analysis...");

    // Load preferences for verbosity
    let prefs = UserPreferences::load();

    // Collect all raw telemetry
    let telemetry = collect_system_telemetry();

    // v0.3.157: Load personality and determine mood from system state
    let mut personality = crate::personality::PersonalityState::load();

    // Quick health check for mood
    let disk_pct = get_disk_usage_percentage();
    let mem_pct = get_memory_usage_percentage();
    let failed_services = count_failed_services();

    personality.update_mood(disk_pct, mem_pct, failed_services, 0);
    personality.record_interaction();
    let _ = personality.save();

    // Build prompt for LLM with personality-driven greeting
    let user_greeting = personality.personalized_greeting(username);

    let (sentence_count, detail_level) = match prefs.briefing.verbosity {
        BriefingVerbosity::Brief => ("5-8", "brief"),
        BriefingVerbosity::Detailed => ("10-15", "detailed with technical specifics"),
    };

    let prompt = format!(
        r#"You are Anna, an AI system administrator. Generate a {} morning briefing for the user.

{}

Analyze the system telemetry below and create a natural, conversational briefing.

INTELLIGENCE PRIORITIES (CRITICAL - READ FIRST):
1. PRIORITIZE CRITICAL ISSUES FIRST:
   - Failed or degraded services
   - Disk usage >90% or predictive alerts showing disk full <14 days
   - Memory >90% or memory leak detected
   - Security breaches or failed login attempts
   - Boot degradation >30% from baseline

2. HIGHLIGHT PREDICTIVE ALERTS:
   - If telemetry shows "Predictive Alerts" section, these are forecasts based on trends
   - Example: "Disk will reach 95% in 8 days" - mention this prominently with timeframe
   - Focus on alerts marked [CRITICAL] first, then [WARNING]
   - Explain the trend and recommended action

3. USE HISTORICAL CONTEXT:
   - If metrics show significant deviation from 30-day average (>20%), mention it
   - Example: "Memory usage 40% above normal" is more meaningful than "Memory at 60%"
   - Explain WHY the change matters (performance impact, capacity planning)

4. BE ACTIONABLE:
   - Skip routine/stable metrics unless user asks for detailed mode
   - Focus on what needs attention or what changed significantly
   - For problems: suggest concrete next steps (even if just "investigate")
   - Don't just list numbers - explain implications

5. PROVIDE CONTEXT:
   - Explain WHY a metric matters (e.g., "high disk usage means updates may fail")
   - Connect trends to user impact (e.g., "slower boot means longer startup time")
   - For good news: acknowledge improvements (e.g., "boot time improved 12% this week")

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
10. PERSONALITY TONE: {}

STYLE:
- Conversational and friendly
- No bullet points or markdown formatting
- Natural paragraphs
- Use "your system" not "the system"
- Sparklines are visual: ▁▂▃▄▅▆▇█ (don't reproduce them, just describe trend if notable)
- Match the personality tone specified above

SYSTEM TELEMETRY:
{}

Generate the briefing now:"#,
        detail_level,
        user_greeting,
        sentence_count,
        personality.tone_instruction(),
        telemetry
    );

    // Call LLM
    let model = std::env::var("ANNA_OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:14b".to_string());
    match crate::ollama::chat_with_timeout(&model, &prompt, 60).await {
        Ok(response) => {
            info!("Morning briefing generated successfully");
            // Add personality-driven closing
            let closing = personality.closing_message();
            Ok(format!("{}\n\n{}", response.trim(), closing))
        }
        Err(e) => {
            warn!("LLM failed to generate briefing: {}", e);
            // Fallback to simple summary
            let closing = personality.closing_message();
            Ok(format!(
                "{}  Your system is running. Check `annactl status` for details.\n\n{}",
                user_greeting, closing
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
