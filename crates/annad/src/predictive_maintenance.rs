//! Predictive Maintenance - Anna forecasts future system health issues.
//!
//! Philosophy: Analyze trends, predict future problems, give time-to-failure estimates.
//! NO HARDCODING: LLM decides what warnings matter based on context.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// A predicted future issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// What will happen
    pub prediction: String,
    /// When it will happen
    pub estimated_date: Option<DateTime<Utc>>,
    /// Days until it happens
    pub days_until: Option<f32>,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
    /// Current trend
    pub trend: String,
    /// Severity
    pub severity: PredictionSeverity,
    /// Recommended action
    pub recommendation: String,
    /// Supporting data
    pub evidence: Vec<String>,
}

/// Prediction severity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredictionSeverity {
    Info,       // >30 days away
    Warning,    // 7-30 days
    Urgent,     // <7 days
    Critical,   // <24 hours
}

/// Health forecast result.
#[derive(Debug, Clone)]
pub struct HealthForecast {
    pub predictions: Vec<Prediction>,
    pub overall_health_score: f32, // 0-100
    pub trends_summary: String,
}

/// Predict disk space exhaustion.
pub async fn predict_disk_exhaustion() -> Result<Option<Prediction>> {
    let history = anna_shared::monitor::LongTermHistory::load();

    if history.daily_snapshots.len() < 7 {
        return Ok(None); // Need at least 7 days
    }

    // Get recent disk usage trends (last 14 days)
    let recent: Vec<_> = history.daily_snapshots.iter()
        .rev()
        .take(14)
        .collect();

    if recent.len() < 7 {
        return Ok(None);
    }

    // Calculate daily growth rate
    let disk_values: Vec<f32> = recent.iter().map(|s| s.disk_used_gb).collect();
    let growth_rate = calculate_linear_trend(&disk_values);

    // Only predict if growing
    if growth_rate <= 0.0 {
        return Ok(None);
    }

    let current_disk_gb = recent.first().map(|s| s.disk_used_gb).unwrap_or(0.0);

    // Get current disk total from df command
    let total_disk_gb = if let Ok(output) = crate::core_loop::execute_command("df -BG / | tail -1 | awk '{print $2}'") {
        output.trim().trim_end_matches('G').parse::<f32>().unwrap_or(100.0)
    } else {
        100.0 // Fallback estimate
    };

    let current_pct = (current_disk_gb / total_disk_gb) * 100.0;

    // Calculate days until 95% full
    let remaining_gb = (total_disk_gb * 0.95) - current_disk_gb;
    let days_until_95 = if growth_rate > 0.0 {
        remaining_gb / growth_rate
    } else {
        999.0
    };

    // Only alert if <60 days away
    if days_until_95 > 60.0 {
        return Ok(None);
    }

    info!("Disk exhaustion predicted in {:.1} days (growing {:.2}GB/day)", days_until_95, growth_rate);

    let severity = if days_until_95 < 1.0 {
        PredictionSeverity::Critical
    } else if days_until_95 < 7.0 {
        PredictionSeverity::Urgent
    } else if days_until_95 < 30.0 {
        PredictionSeverity::Warning
    } else {
        PredictionSeverity::Info
    };

    let estimated_date = Utc::now() + Duration::days(days_until_95 as i64);

    Ok(Some(Prediction {
        prediction: format!("Disk will reach 95% full (currently {:.1}%)", current_pct),
        estimated_date: Some(estimated_date),
        days_until: Some(days_until_95),
        confidence: if recent.len() >= 14 { 0.85 } else { 0.70 },
        trend: format!("Growing {:.2}GB/day", growth_rate),
        severity,
        recommendation: format!(
            "Free up space soon. Growing at {:.2}GB/day, need to free ~{:.1}GB or reduce growth rate.",
            growth_rate, remaining_gb
        ),
        evidence: vec![
            format!("Current: {:.1}GB used of {:.1}GB", current_disk_gb, total_disk_gb),
            format!("14-day trend: +{:.2}GB/day", growth_rate),
            format!("At current rate: 95% full in {:.0} days", days_until_95),
        ],
    }))
}

/// Predict memory leak.
pub async fn predict_memory_leak() -> Result<Option<Prediction>> {
    let history = anna_shared::monitor::LongTermHistory::load();

    if history.daily_snapshots.len() < 14 {
        return Ok(None);
    }

    let recent: Vec<_> = history.daily_snapshots.iter()
        .rev()
        .take(14)
        .collect();

    // Check if memory usage is steadily increasing
    let mem_values: Vec<f32> = recent.iter().map(|s| s.avg_memory_pct).collect();
    let growth_rate = calculate_linear_trend(&mem_values);

    // Memory leak: steady growth >0.5% per day
    if growth_rate < 0.5 {
        return Ok(None);
    }

    let current_mem = recent.first().map(|s| s.avg_memory_pct).unwrap_or(0.0);
    let days_until_90 = if growth_rate > 0.0 {
        (90.0 - current_mem) / growth_rate
    } else {
        999.0
    };

    if days_until_90 > 60.0 {
        return Ok(None);
    }

    info!("Memory leak suspected: growing {:.2}%/day, will hit 90% in {:.0} days", growth_rate, days_until_90);

    let severity = if days_until_90 < 7.0 {
        PredictionSeverity::Urgent
    } else if days_until_90 < 30.0 {
        PredictionSeverity::Warning
    } else {
        PredictionSeverity::Info
    };

    Ok(Some(Prediction {
        prediction: "Possible memory leak detected".to_string(),
        estimated_date: if days_until_90 < 90.0 {
            Some(Utc::now() + Duration::days(days_until_90 as i64))
        } else {
            None
        },
        days_until: Some(days_until_90),
        confidence: 0.75,
        trend: format!("Memory usage increasing {:.2}%/day", growth_rate),
        severity,
        recommendation: "Identify which process is leaking memory. Check for long-running processes with growing RSS.".to_string(),
        evidence: vec![
            format!("Current memory: {:.1}%", current_mem),
            format!("14-day trend: +{:.2}%/day", growth_rate),
            format!("Will reach 90% in ~{:.0} days if unchecked", days_until_90),
        ],
    }))
}

/// Predict boot time degradation.
pub async fn predict_boot_degradation() -> Result<Option<Prediction>> {
    let history = anna_shared::monitor::LongTermHistory::load();

    if history.daily_snapshots.len() < 14 {
        return Ok(None);
    }

    let recent: Vec<_> = history.daily_snapshots.iter()
        .rev()
        .take(14)
        .collect();

    let boot_values: Vec<f32> = recent.iter().map(|s| s.avg_boot_time).collect();
    let growth_rate = calculate_linear_trend(&boot_values);

    // Only alert if boot time increasing >0.1s per day
    if growth_rate < 0.1 {
        return Ok(None);
    }

    let current_boot = recent.first().map(|s| s.avg_boot_time).unwrap_or(0.0);
    let days_until_slow = if growth_rate > 0.0 {
        (30.0 - current_boot) / growth_rate // 30s is "slow"
    } else {
        999.0
    };

    if days_until_slow > 90.0 {
        return Ok(None);
    }

    info!("Boot time degrading: growing {:.2}s/day", growth_rate);

    Ok(Some(Prediction {
        prediction: "Boot time is degrading".to_string(),
        estimated_date: if days_until_slow < 90.0 {
            Some(Utc::now() + Duration::days(days_until_slow as i64))
        } else {
            None
        },
        days_until: Some(days_until_slow),
        confidence: 0.70,
        trend: format!("Increasing {:.2}s/day", growth_rate),
        severity: PredictionSeverity::Info,
        recommendation: "Investigate systemd services. New services may be slowing boot.".to_string(),
        evidence: vec![
            format!("Current boot: {:.1}s", current_boot),
            format!("14-day trend: +{:.2}s/day", growth_rate),
        ],
    }))
}

/// Predict SSD wear.
pub async fn predict_ssd_wear() -> Result<Option<Prediction>> {
    // Check for NVMe drives
    let nvme_output = crate::core_loop::execute_command("nvme smart-log /dev/nvme0n1 2>/dev/null");

    if let Ok(output) = nvme_output {
        if output.is_empty() {
            return Ok(None);
        }

        // Parse percentage_used
        for line in output.lines() {
            if line.contains("percentage_used") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pct_str) = parts.last() {
                    if let Ok(pct_used) = pct_str.trim_end_matches('%').parse::<f32>() {
                        // NVMe reports 0-100% of warranty
                        if pct_used < 50.0 {
                            return Ok(None); // Not worth alerting yet
                        }

                        // Estimate years until 100% (assume linear wear)
                        // This is a rough estimate - actual wear depends on write patterns
                        let remaining_pct = 100.0 - pct_used;

                        // Try to get drive age from uptime or installation date
                        // For now, assume 1% = ~1 year for consumer SSDs (rough estimate)
                        let years_until_limit = remaining_pct / 10.0; // Very rough estimate

                        return Ok(Some(Prediction {
                            prediction: "SSD warranty limit approaching".to_string(),
                            estimated_date: None,
                            days_until: Some(years_until_limit * 365.0),
                            confidence: 0.60, // Low confidence - wear is non-linear
                            trend: format!("{}% of warranty consumed", pct_used),
                            severity: if pct_used > 90.0 {
                                PredictionSeverity::Warning
                            } else {
                                PredictionSeverity::Info
                            },
                            recommendation: if pct_used > 80.0 {
                                "Consider backing up important data and monitoring closely.".to_string()
                            } else {
                                "SSD health is acceptable, but monitor wear level.".to_string()
                            },
                            evidence: vec![
                                format!("NVMe percentage_used: {:.1}%", pct_used),
                                "Note: Warranty limit is conservative; drives often outlast it".to_string(),
                            ],
                        }));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Generate health forecast.
pub async fn generate_health_forecast() -> Result<HealthForecast> {
    info!("Generating health forecast...");

    let mut predictions = Vec::new();

    // Gather all predictions
    if let Some(pred) = predict_disk_exhaustion().await? {
        predictions.push(pred);
    }

    if let Some(pred) = predict_memory_leak().await? {
        predictions.push(pred);
    }

    if let Some(pred) = predict_boot_degradation().await? {
        predictions.push(pred);
    }

    if let Some(pred) = predict_ssd_wear().await? {
        predictions.push(pred);
    }

    // Calculate overall health score (0-100)
    let health_score = calculate_health_score(&predictions);

    // Generate trends summary
    let trends_summary = if predictions.is_empty() {
        "All metrics stable, no concerning trends detected.".to_string()
    } else {
        let critical = predictions.iter().filter(|p| p.severity == PredictionSeverity::Critical).count();
        let urgent = predictions.iter().filter(|p| p.severity == PredictionSeverity::Urgent).count();
        let warning = predictions.iter().filter(|p| p.severity == PredictionSeverity::Warning).count();

        if critical > 0 {
            format!("{} critical prediction(s) requiring immediate attention", critical)
        } else if urgent > 0 {
            format!("{} urgent prediction(s) - action needed within 7 days", urgent)
        } else if warning > 0 {
            format!("{} warning(s) - monitor and plan accordingly", warning)
        } else {
            format!("{} informational prediction(s)", predictions.len())
        }
    };

    Ok(HealthForecast {
        predictions,
        overall_health_score: health_score,
        trends_summary,
    })
}

/// Calculate overall health score from predictions.
fn calculate_health_score(predictions: &[Prediction]) -> f32 {
    if predictions.is_empty() {
        return 95.0; // Good health if no predictions
    }

    let mut score = 100.0;

    for pred in predictions {
        let penalty = match pred.severity {
            PredictionSeverity::Critical => 30.0,
            PredictionSeverity::Urgent => 20.0,
            PredictionSeverity::Warning => 10.0,
            PredictionSeverity::Info => 5.0,
        };

        score -= penalty * pred.confidence;
    }

    score.max(0.0)
}

/// Calculate linear trend (simple regression).
fn calculate_linear_trend(values: &[f32]) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }

    let n = values.len() as f32;
    let x_vals: Vec<f32> = (0..values.len()).map(|i| i as f32).collect();

    let sum_x: f32 = x_vals.iter().sum();
    let sum_y: f32 = values.iter().sum();
    let sum_xy: f32 = x_vals.iter().zip(values.iter()).map(|(x, y)| x * y).sum();
    let sum_x2: f32 = x_vals.iter().map(|x| x * x).sum();

    // Slope = (n*sum_xy - sum_x*sum_y) / (n*sum_x2 - sum_x^2)
    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = n * sum_x2 - sum_x * sum_x;

    if denominator.abs() < 0.001 {
        return 0.0;
    }

    numerator / denominator
}

/// Format health forecast for display.
pub fn format_health_forecast(forecast: &HealthForecast) -> String {
    let mut response = format!(
        "System Health Forecast (Score: {:.0}/100)\\n\\n",
        forecast.overall_health_score
    );

    response.push_str(&format!("{}\\n\\n", forecast.trends_summary));

    if forecast.predictions.is_empty() {
        response.push_str("No concerning trends detected. System health looks stable.\\n");
        return response;
    }

    // Group by severity
    let critical: Vec<_> = forecast.predictions.iter()
        .filter(|p| p.severity == PredictionSeverity::Critical)
        .collect();

    let urgent: Vec<_> = forecast.predictions.iter()
        .filter(|p| p.severity == PredictionSeverity::Urgent)
        .collect();

    let warning: Vec<_> = forecast.predictions.iter()
        .filter(|p| p.severity == PredictionSeverity::Warning)
        .collect();

    let info: Vec<_> = forecast.predictions.iter()
        .filter(|p| p.severity == PredictionSeverity::Info)
        .collect();

    if !critical.is_empty() {
        response.push_str("CRITICAL (Immediate Action Required):\\n");
        for pred in critical {
            response.push_str(&format_prediction(pred));
            response.push('\n');
        }
    }

    if !urgent.is_empty() {
        response.push_str("URGENT (Action Needed <7 Days):\\n");
        for pred in urgent {
            response.push_str(&format_prediction(pred));
            response.push('\n');
        }
    }

    if !warning.is_empty() {
        response.push_str("Warnings (Monitor Closely):\\n");
        for pred in warning {
            response.push_str(&format_prediction(pred));
            response.push('\n');
        }
    }

    if !info.is_empty() {
        response.push_str("Informational:\\n");
        for pred in info {
            response.push_str(&format_prediction(pred));
            response.push('\n');
        }
    }

    response
}

/// Format single prediction.
fn format_prediction(pred: &Prediction) -> String {
    let mut s = format!("  - {} ({:.0}% confident)\\n", pred.prediction, pred.confidence * 100.0);

    if let Some(days) = pred.days_until {
        s.push_str(&format!("    Time: {:.0} days\\n", days));
    }

    s.push_str(&format!("    Trend: {}\\n", pred.trend));
    s.push_str(&format!("    Action: {}\\n", pred.recommendation));

    s
}

/// Check if predictive maintenance should run.
pub async fn should_run_prediction() -> bool {
    // Run prediction weekly (Monday morning briefing)
    let history = anna_shared::monitor::LongTermHistory::load();
    history.daily_snapshots.len() >= 14
}
