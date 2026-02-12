//! Regression Detection - Anna detects performance degradations and explains what changed.
//!
//! Philosophy: Compare current metrics to baseline, identify what changed, explain impact.
//! NO HARDCODING: LLM analyzes changes and proposes fixes.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

/// A detected performance regression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regression {
    /// What metric regressed
    pub metric: String,
    /// Current value
    pub current_value: f32,
    /// Baseline value
    pub baseline_value: f32,
    /// Percentage change
    pub change_pct: f32,
    /// When regression started
    pub started_at: Option<DateTime<Utc>>,
    /// Likely causes
    pub causes: Vec<RegressionCause>,
    /// Severity
    pub severity: RegressionSeverity,
}

/// A potential cause of regression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionCause {
    /// What changed
    pub change: String,
    /// When it changed
    pub when: Option<DateTime<Utc>>,
    /// Impact estimate
    pub impact: String,
    /// Likelihood this is the cause (0.0-1.0)
    pub likelihood: f32,
    /// Suggested fix
    pub fix: Option<String>,
}

/// Regression severity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegressionSeverity {
    Minor,      // <10% regression
    Moderate,   // 10-25% regression
    Significant, // 25-50% regression
    Severe,     // >50% regression
}

/// Analyze boot time regression.
pub async fn analyze_boot_regression(
    current_boot: f32,
    baseline_boot: f32,
) -> Result<Option<Regression>> {
    let change_pct = ((current_boot - baseline_boot) / baseline_boot) * 100.0;

    if change_pct < 10.0 {
        return Ok(None); // No significant regression
    }

    info!("Boot regression detected: {:.1}s -> {:.1}s ({:.0}% slower)", 
        baseline_boot, current_boot, change_pct);

    let mut causes = Vec::new();

    // Analyze systemd-analyze blame
    if let Ok(output) = crate::core_loop::execute_command("systemd-analyze blame | head -10") {
        let lines: Vec<&str> = output.lines().collect();

        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let time_str = parts[0];
                let service = parts.get(1..).map(|s| s.join(" ")).unwrap_or_default();

                // Parse time (e.g., "1.234s" or "123ms")
                if let Some(time_s) = parse_time_to_seconds(time_str) {
                    if time_s > 1.0 {
                        causes.push(RegressionCause {
                            change: format!("{} taking {:.1}s", service, time_s),
                            when: None,
                            impact: format!("{:.1}s added to boot time", time_s),
                            likelihood: 0.8,
                            fix: Some(suggest_service_fix(&service, time_s)),
                        });
                    }
                }
            }
        }
    }

    // Check for new services
    if let Ok(output) = crate::core_loop::execute_command("systemctl list-unit-files --state=enabled --no-pager --no-legend | wc -l") {
        if let Ok(count) = output.trim().parse::<u32>() {
            if count > 30 {
                causes.push(RegressionCause {
                    change: format!("{} services enabled", count),
                    when: None,
                    impact: "Multiple services slowing boot".to_string(),
                    likelihood: 0.6,
                    fix: Some("Review enabled services and disable unnecessary ones".to_string()),
                });
            }
        }
    }

    let severity = if change_pct > 50.0 {
        RegressionSeverity::Severe
    } else if change_pct > 25.0 {
        RegressionSeverity::Significant
    } else if change_pct > 10.0 {
        RegressionSeverity::Moderate
    } else {
        RegressionSeverity::Minor
    };

    Ok(Some(Regression {
        metric: "Boot Time".to_string(),
        current_value: current_boot,
        baseline_value: baseline_boot,
        change_pct,
        started_at: None,
        causes,
        severity,
    }))
}

/// Parse time string to seconds.
fn parse_time_to_seconds(time_str: &str) -> Option<f32> {
    if let Some(s) = time_str.strip_suffix('s') {
        s.parse::<f32>().ok()
    } else if let Some(ms) = time_str.strip_suffix("ms") {
        ms.parse::<f32>().ok().map(|m| m / 1000.0)
    } else {
        None
    }
}

/// Suggest fix for slow service.
fn suggest_service_fix(service: &str, time_s: f32) -> String {
    let service_lower = service.to_lowercase();

    if service_lower.contains("networkmanager-wait-online") {
        "Disable wait-online (safe): systemctl disable NetworkManager-wait-online.service".to_string()
    } else if service_lower.contains("plymouth") {
        "Disable boot splash if not needed: systemctl disable plymouth-start.service".to_string()
    } else if service_lower.contains("snapd") && time_s > 2.0 {
        "Snapd slow start - consider disabling if not using snaps".to_string()
    } else if service_lower.contains("docker") {
        "Docker slow start - check docker logs for issues".to_string()
    } else {
        format!("Investigate why {} is slow", service)
    }
}

/// Analyze memory usage regression.
pub async fn analyze_memory_regression(
    current_mem_pct: f32,
    baseline_mem_pct: f32,
) -> Result<Option<Regression>> {
    let change_pct = ((current_mem_pct - baseline_mem_pct) / baseline_mem_pct) * 100.0;

    if change_pct < 15.0 {
        return Ok(None);
    }

    info!("Memory regression detected: {:.1}% -> {:.1}% ({:.0}% increase)", 
        baseline_mem_pct, current_mem_pct, change_pct);

    let mut causes = Vec::new();

    // Check for memory-heavy processes
    if let Ok(output) = crate::core_loop::execute_command("ps aux --sort=-%mem | head -5") {
        let lines: Vec<&str> = output.lines().skip(1).collect();

        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let mem_pct: f32 = parts[3].parse().unwrap_or(0.0);
                let process = parts[10];

                if mem_pct > 10.0 {
                    causes.push(RegressionCause {
                        change: format!("{} using {:.1}% memory", process, mem_pct),
                        when: None,
                        impact: format!("{:.1}% of total memory", mem_pct),
                        likelihood: 0.7,
                        fix: Some(format!("Investigate or restart {}", process)),
                    });
                }
            }
        }
    }

    let severity = if change_pct > 50.0 {
        RegressionSeverity::Severe
    } else if change_pct > 30.0 {
        RegressionSeverity::Significant
    } else {
        RegressionSeverity::Moderate
    };

    Ok(Some(Regression {
        metric: "Memory Usage".to_string(),
        current_value: current_mem_pct,
        baseline_value: baseline_mem_pct,
        change_pct,
        started_at: None,
        causes,
        severity,
    }))
}

/// Detect all regressions.
pub async fn detect_regressions() -> Result<Vec<Regression>> {
    let mut regressions = Vec::new();

    // Load historical data
    let history = anna_shared::monitor::LongTermHistory::load();

    if history.daily_snapshots.len() < 7 {
        return Ok(regressions); // Need at least 7 days
    }

    // Calculate baselines (7-14 days ago)
    let baseline_snapshots: Vec<_> = history.daily_snapshots.iter()
        .rev()
        .skip(7)
        .take(7)
        .collect();

    let recent_snapshots: Vec<_> = history.daily_snapshots.iter()
        .rev()
        .take(7)
        .collect();

    if baseline_snapshots.is_empty() || recent_snapshots.is_empty() {
        return Ok(regressions);
    }

    // Boot time regression
    let baseline_boot: f32 = baseline_snapshots.iter()
        .map(|s| s.avg_boot_time)
        .sum::<f32>() / baseline_snapshots.len() as f32;

    let current_boot: f32 = recent_snapshots.iter()
        .map(|s| s.avg_boot_time)
        .sum::<f32>() / recent_snapshots.len() as f32;

    if let Some(regression) = analyze_boot_regression(current_boot, baseline_boot).await? {
        regressions.push(regression);
    }

    // Memory regression
    let baseline_mem: f32 = baseline_snapshots.iter()
        .map(|s| s.avg_memory_pct)
        .sum::<f32>() / baseline_snapshots.len() as f32;

    let current_mem: f32 = recent_snapshots.iter()
        .map(|s| s.avg_memory_pct)
        .sum::<f32>() / recent_snapshots.len() as f32;

    if let Some(regression) = analyze_memory_regression(current_mem, baseline_mem).await? {
        regressions.push(regression);
    }

    Ok(regressions)
}

/// Format regression for display.
pub fn format_regression(regression: &Regression) -> String {
    let severity_str = match regression.severity {
        RegressionSeverity::Minor => "Minor",
        RegressionSeverity::Moderate => "Moderate",
        RegressionSeverity::Significant => "Significant",
        RegressionSeverity::Severe => "SEVERE",
    };

    let mut response = format!(
        "{} Regression Detected ({} severity)\n\n",
        regression.metric, severity_str
    );

    response.push_str(&format!(
        "Changed: {:.1} -> {:.1} ({:+.0}% {regression})\n\n",
        regression.baseline_value,
        regression.current_value,
        regression.change_pct,
        regression = if regression.change_pct > 0.0 { "worse" } else { "better" }
    ));

    if !regression.causes.is_empty() {
        response.push_str("Likely Causes:\n");
        for (i, cause) in regression.causes.iter().enumerate().take(3) {
            response.push_str(&format!(
                "{}. {} ({:.0}% likely)\n",
                i + 1,
                cause.change,
                cause.likelihood * 100.0
            ));

            if let Some(fix) = &cause.fix {
                response.push_str(&format!("   Fix: {}\n", fix));
            }

            response.push('\n');
        }
    }

    response.push_str("Would you like me to:\n");
    response.push_str("1. Apply suggested fixes automatically\n");
    response.push_str("2. Show detailed analysis\n");
    response.push_str("3. Monitor for another week\n");
    response.push_str("4. Mark as acceptable (stop alerting)\n");

    response
}
