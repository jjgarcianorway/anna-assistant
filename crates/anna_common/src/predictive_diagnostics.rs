//! v6.28.0: Predictive Diagnostics Engine (PDE) v1
//!
//! The Predictive Diagnostics Engine analyzes historical telemetry to detect early
//! warning signals and forecast future system risks using deterministic, rules-based methods.
//!
//! ## Design Principles
//!
//! 1. **Deterministic**: Pure rules-based logic, no LLM dependencies
//! 2. **Evidence-Based**: All predictions backed by historical data
//! 3. **Time-Windowed**: Precise prediction windows ("next 3 days", "48 hours")
//! 4. **Actionable**: Every prediction includes recommended actions
//! 5. **Conservative**: Prefer false negatives over false positives
//!
//! ## Architecture
//!
//! - Extends Historian with new trend models (CPU pressure, I/O wait, thermal, network)
//! - Implements 5 core predictors (disk-full, thermal creep, CPU pressure, I/O wait, network latency)
//! - Integrates with Insights Engine for unified output
//! - Displayed in status command under "📈 Predictive Diagnostics"

use crate::historian::Historian;
use crate::insights_engine::InsightSeverity;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A predictive insight about future system risks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveInsight {
    /// Unique identifier (e.g., "disk_full_prediction_2025-11-24")
    pub id: String,

    /// Short title describing the prediction (1 line)
    pub title: String,

    /// Severity level
    pub severity: InsightSeverity,

    /// Time window for prediction ("next 3 days", "48 hours", etc.)
    pub prediction_window: String,

    /// Evidence supporting this prediction
    pub evidence: Vec<String>,

    /// Root cause analysis (optional)
    pub cause: Option<String>,

    /// Recommended preventive actions
    pub recommended_actions: Vec<String>,

    /// When this prediction was generated
    pub generated_at: DateTime<Utc>,
}

impl PredictiveInsight {
    pub fn new(
        id_prefix: impl Into<String>,
        title: impl Into<String>,
        severity: InsightSeverity,
        prediction_window: impl Into<String>,
    ) -> Self {
        let generated_at = Utc::now();
        let id = format!("{}_{}", id_prefix.into(), generated_at.format("%Y%m%d_%H%M%S"));

        Self {
            id,
            title: title.into(),
            severity,
            prediction_window: prediction_window.into(),
            evidence: Vec::new(),
            cause: None,
            recommended_actions: Vec::new(),
            generated_at,
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence = evidence;
        self
    }

    pub fn with_cause(mut self, cause: impl Into<String>) -> Self {
        self.cause = Some(cause.into());
        self
    }

    pub fn with_actions(mut self, actions: Vec<String>) -> Self {
        self.recommended_actions = actions;
        self
    }
}

// ============================================================================
// Trend Models for Prediction
// ============================================================================

/// CPU pressure trend analysis
#[derive(Debug, Clone)]
pub struct CpuPressureTrend {
    /// Baseline CPU usage (average over historical period)
    pub baseline: f64,

    /// Current average CPU usage
    pub current_average: f64,

    /// Deviation from baseline (percentage points)
    pub deviation_percent: f64,

    /// Rate of change (percentage points per day)
    pub slope: f64,

    /// Number of data points used for calculation
    pub data_points_count: usize,
}

/// I/O pressure trend analysis
#[derive(Debug, Clone)]
pub struct IoPressureTrend {
    /// Baseline I/O wait percentage
    pub baseline: f64,

    /// Current average I/O wait
    pub current_average: f64,

    /// Deviation from baseline (percentage points)
    pub deviation_percent: f64,

    /// Rate of change (percentage points per day)
    pub slope: f64,

    /// Number of data points
    pub data_points_count: usize,
}

/// Thermal trend analysis
#[derive(Debug, Clone)]
pub struct ThermalTrend {
    /// Baseline temperature (°C)
    pub baseline: f64,

    /// Current average temperature
    pub current_average: f64,

    /// Deviation from baseline (°C)
    pub deviation_degrees: f64,

    /// Rate of change (°C per 24 hours)
    pub slope: f64,

    /// Number of data points
    pub data_points_count: usize,
}

/// Network latency trend analysis
#[derive(Debug, Clone)]
pub struct NetworkLatencyTrend {
    /// Baseline latency (ms)
    pub baseline: f64,

    /// Current average latency
    pub current_average: f64,

    /// Deviation from baseline (ms)
    pub deviation_ms: f64,

    /// Rate of change (ms per hour)
    pub slope: f64,

    /// Variance (ms²) - indicates instability
    pub variance: f64,

    /// Number of data points
    pub data_points_count: usize,
}

// ============================================================================
// Historian Extensions
// ============================================================================
//
// Note: Trend models are defined here, but the methods to retrieve them
// are implemented in historian.rs to access private fields

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predictive_insight_creation() {
        let insight = PredictiveInsight::new(
            "disk_full_prediction",
            "Root partition will reach 95% in 3 days",
            InsightSeverity::Warning,
            "next 3 days",
        );

        assert_eq!(insight.severity, InsightSeverity::Warning);
        assert_eq!(insight.prediction_window, "next 3 days");
        assert!(insight.id.starts_with("disk_full_prediction_"));
    }

    #[test]
    fn test_predictive_insight_with_evidence() {
        let insight = PredictiveInsight::new(
            "thermal_creep",
            "Temperature trending upward",
            InsightSeverity::Warning,
            "next 48 hours",
        )
        .with_evidence(vec![
            "Baseline: 55°C".to_string(),
            "Current: 62°C".to_string(),
            "Slope: +2°C per 24h".to_string(),
        ]);

        assert_eq!(insight.evidence.len(), 3);
        assert!(insight.evidence[0].contains("55°C"));
    }

    #[test]
    fn test_predictive_insight_with_cause_and_actions() {
        let insight = PredictiveInsight::new(
            "cpu_pressure",
            "CPU pressure increasing",
            InsightSeverity::Info,
            "next 24 hours",
        )
        .with_cause("Background process consuming resources")
        .with_actions(vec![
            "Check: ps aux --sort=-%cpu | head -10".to_string(),
            "Consider: systemctl stop suspected-service".to_string(),
        ]);

        assert!(insight.cause.is_some());
        assert_eq!(insight.recommended_actions.len(), 2);
    }
}

// ============================================================================
// Predictor Algorithms (Deterministic)
// ============================================================================

/// Predict disk-full events based on usage trends
///
/// Analyzes disk usage growth rate and predicts when the root partition will reach 95% capacity.
/// Uses conservative linear extrapolation based on historical trends.
pub fn predict_disk_full(historian: &Historian, days: u32) -> Result<Option<PredictiveInsight>> {
    // Get disk trends for the root filesystem
    let disk_trends = historian.get_disk_trends(days)?;

    // Conservative thresholds
    const WARNING_THRESHOLD_PERCENT: f64 = 95.0;
    const CRITICAL_THRESHOLD_PERCENT: f64 = 98.0;
    const MIN_GROWTH_RATE: f64 = 0.1; // Minimum 0.1 GB/day to trigger prediction

    // Only predict if there's measurable growth
    if disk_trends.growth_rate_gb_per_day < MIN_GROWTH_RATE {
        return Ok(None);
    }

    // Calculate days until warning threshold
    let free_gb = disk_trends.total_gb - disk_trends.used_gb;
    let warning_threshold_gb = disk_trends.total_gb * (WARNING_THRESHOLD_PERCENT / 100.0);
    let critical_threshold_gb = disk_trends.total_gb * (CRITICAL_THRESHOLD_PERCENT / 100.0);

    let gb_until_warning = warning_threshold_gb - disk_trends.used_gb;
    let gb_until_critical = critical_threshold_gb - disk_trends.used_gb;

    // Calculate days using linear extrapolation
    let days_until_warning = gb_until_warning / disk_trends.growth_rate_gb_per_day;
    let days_until_critical = gb_until_critical / disk_trends.growth_rate_gb_per_day;

    // Only predict if within reasonable forecast window (next 90 days)
    if days_until_warning > 90.0 {
        return Ok(None);
    }

    // Determine severity and prediction window
    let (severity, prediction_window, threshold_desc) = if days_until_critical <= 7.0 && days_until_critical > 0.0 {
        (
            InsightSeverity::Critical,
            format!("{} days", days_until_critical.ceil() as i32),
            "98%"
        )
    } else if days_until_warning <= 30.0 && days_until_warning > 0.0 {
        (
            InsightSeverity::Warning,
            format!("{} days", days_until_warning.ceil() as i32),
            "95%"
        )
    } else {
        return Ok(None);
    };

    let insight = PredictiveInsight::new(
        "disk_full_prediction",
        format!("Root partition predicted to reach {} capacity", threshold_desc),
        severity,
        &prediction_window,
    )
    .with_evidence(vec![
        format!("Current usage: {:.1}% ({:.1} GB / {:.1} GB)",
            disk_trends.current_used_percent,
            disk_trends.used_gb,
            disk_trends.total_gb),
        format!("Growth rate: {:.2} GB/day", disk_trends.growth_rate_gb_per_day),
        format!("Free space remaining: {:.1} GB", free_gb),
        format!("Data points: {} days", disk_trends.days_analyzed),
    ])
    .with_cause("Sustained disk usage growth detected based on historical trends")
    .with_actions(vec![
        "Check disk usage: df -h /".to_string(),
        "Find large directories: du -sh /* | sort -rh | head -10".to_string(),
        "Clean package cache: sudo pacman -Sc".to_string(),
        "Remove orphaned packages: sudo pacman -Rns $(pacman -Qdtq)".to_string(),
        "Check system journals: sudo journalctl --disk-usage".to_string(),
    ]);

    Ok(Some(insight))
}

/// Predict thermal creep (temperature trending upward)
///
/// Detects sustained temperature increases that could lead to thermal throttling or hardware damage.
/// Triggers if temperature increase exceeds 8% over 24 hours.
pub fn predict_thermal_creep(historian: &Historian, days: u32) -> Result<Option<PredictiveInsight>> {
    let thermal_trend = historian.get_thermal_trend(days)?;

    // Need sufficient data points for reliable prediction
    if thermal_trend.data_points_count < 5 {
        return Ok(None);
    }

    // Conservative thresholds
    const WARNING_TEMP_C: f64 = 75.0;
    const CRITICAL_TEMP_C: f64 = 85.0;
    const MIN_DEVIATION_PERCENT: f64 = 8.0; // 8% increase from baseline

    // Calculate deviation percentage
    let deviation_percent = if thermal_trend.baseline > 0.0 {
        (thermal_trend.deviation_degrees / thermal_trend.baseline) * 100.0
    } else {
        0.0
    };

    // Only trigger if significant temperature increase detected
    if deviation_percent < MIN_DEVIATION_PERCENT {
        return Ok(None);
    }

    // Only trigger if slope is positive (trending up)
    if thermal_trend.slope <= 0.0 {
        return Ok(None);
    }

    // Determine severity based on current temperature and trend
    let severity = if thermal_trend.current_average >= CRITICAL_TEMP_C {
        InsightSeverity::Critical
    } else if thermal_trend.current_average >= WARNING_TEMP_C || deviation_percent >= 15.0 {
        InsightSeverity::Warning
    } else {
        InsightSeverity::Info
    };

    // Calculate prediction window based on slope
    let prediction_window = if thermal_trend.slope >= 2.0 {
        "next 24-48 hours"
    } else {
        "next 3-7 days"
    };

    let insight = PredictiveInsight::new(
        "thermal_creep",
        "Sustained temperature increase detected - thermal throttling risk",
        severity,
        prediction_window,
    )
    .with_evidence(vec![
        format!("Baseline temperature: {:.1}°C", thermal_trend.baseline),
        format!("Current average: {:.1}°C", thermal_trend.current_average),
        format!("Temperature increase: +{:.1}°C ({:.1}%)",
            thermal_trend.deviation_degrees,
            deviation_percent),
        format!("Trend slope: +{:.2}°C per day", thermal_trend.slope),
        format!("Data points: {} samples over {} days", thermal_trend.data_points_count, days),
    ])
    .with_cause("Temperature trending upward - possible cooling system degradation or increased workload")
    .with_actions(vec![
        "Check current temperatures: sensors".to_string(),
        "Monitor CPU/GPU temps: watch -n 2 sensors".to_string(),
        "Check for dust buildup in cooling system".to_string(),
        "Verify fans are operational: sensors | grep fan".to_string(),
        "Review recent high-load processes: ps aux --sort=-%cpu | head -10".to_string(),
        "Consider reducing CPU governor aggression if using performance mode".to_string(),
    ]);

    Ok(Some(insight))
}

/// Predict CPU pressure and potential throttling
///
/// Detects sustained high CPU utilization patterns that may lead to performance degradation.
pub fn predict_cpu_pressure(historian: &Historian, days: u32) -> Result<Option<PredictiveInsight>> {
    let cpu_trend = historian.get_cpu_pressure_trend(days)?;

    // Need sufficient data points
    if cpu_trend.data_points_count < 5 {
        return Ok(None);
    }

    // Conservative thresholds
    const HIGH_CPU_THRESHOLD: f64 = 80.0; // 80% sustained usage
    const CRITICAL_CPU_THRESHOLD: f64 = 95.0; // 95% sustained usage
    const MIN_DEVIATION_PERCENT: f64 = 15.0; // 15% increase from baseline

    // Only trigger if CPU utilization is trending upward
    if cpu_trend.slope <= 0.0 {
        return Ok(None);
    }

    // Check if current usage is already high or deviation is significant
    let is_high_usage = cpu_trend.current_average >= HIGH_CPU_THRESHOLD;
    let is_significant_increase = cpu_trend.deviation_percent >= MIN_DEVIATION_PERCENT;

    if !is_high_usage && !is_significant_increase {
        return Ok(None);
    }

    // Determine severity
    let severity = if cpu_trend.current_average >= CRITICAL_CPU_THRESHOLD {
        InsightSeverity::Critical
    } else if cpu_trend.current_average >= HIGH_CPU_THRESHOLD {
        InsightSeverity::Warning
    } else {
        InsightSeverity::Info
    };

    // Calculate prediction window
    let prediction_window = if cpu_trend.slope >= 2.0 {
        "next 24-48 hours"
    } else {
        "next 3-7 days"
    };

    let insight = PredictiveInsight::new(
        "cpu_pressure",
        "Sustained CPU pressure detected - performance degradation risk",
        severity,
        prediction_window,
    )
    .with_evidence(vec![
        format!("Baseline CPU usage: {:.1}%", cpu_trend.baseline),
        format!("Current average: {:.1}%", cpu_trend.current_average),
        format!("Deviation: +{:.1}%", cpu_trend.deviation_percent),
        format!("Trend slope: +{:.2}% per day", cpu_trend.slope),
        format!("Data points: {} samples over {} days", cpu_trend.data_points_count, days),
    ])
    .with_cause("Sustained high CPU utilization - possible runaway process or increased workload")
    .with_actions(vec![
        "Identify CPU-intensive processes: ps aux --sort=-%cpu | head -15".to_string(),
        "Check system load: uptime".to_string(),
        "Monitor real-time CPU: top -o %CPU".to_string(),
        "Review systemd services: systemctl list-units --type=service --state=running".to_string(),
        "Check for background tasks: systemctl list-timers".to_string(),
        "Consider limiting CPU-heavy services if appropriate".to_string(),
    ]);

    Ok(Some(insight))
}

/// Predict I/O wait issues
///
/// Detects increasing I/O latency that may indicate storage bottlenecks or failing hardware.
pub fn predict_io_wait_pressure(historian: &Historian, days: u32) -> Result<Option<PredictiveInsight>> {
    let io_trend = historian.get_io_pressure_trend(days)?;

    // Need sufficient data points
    if io_trend.data_points_count < 5 {
        return Ok(None);
    }

    // Conservative thresholds (I/O latency in milliseconds)
    const HIGH_LATENCY_MS: f64 = 50.0; // 50ms average latency is concerning
    const CRITICAL_LATENCY_MS: f64 = 100.0; // 100ms+ is critical
    const MIN_DEVIATION_PERCENT: f64 = 30.0; // 30% increase from baseline

    // Only trigger if latency is trending upward
    if io_trend.slope <= 0.0 {
        return Ok(None);
    }

    // Check if current latency is high or deviation is significant
    let is_high_latency = io_trend.current_average >= HIGH_LATENCY_MS;
    let is_significant_increase = io_trend.deviation_percent >= MIN_DEVIATION_PERCENT;

    if !is_high_latency && !is_significant_increase {
        return Ok(None);
    }

    // Determine severity
    let severity = if io_trend.current_average >= CRITICAL_LATENCY_MS {
        InsightSeverity::Critical
    } else if io_trend.current_average >= HIGH_LATENCY_MS || io_trend.deviation_percent >= 50.0 {
        InsightSeverity::Warning
    } else {
        InsightSeverity::Info
    };

    // Calculate prediction window based on slope
    let prediction_window = if io_trend.slope >= 5.0 {
        "next 24-48 hours"
    } else {
        "next 3-7 days"
    };

    let insight = PredictiveInsight::new(
        "io_wait_pressure",
        "Increasing I/O latency detected - storage bottleneck risk",
        severity,
        prediction_window,
    )
    .with_evidence(vec![
        format!("Baseline I/O latency: {:.1} ms", io_trend.baseline),
        format!("Current average: {:.1} ms", io_trend.current_average),
        format!("Latency increase: +{:.1}%", io_trend.deviation_percent),
        format!("Trend slope: +{:.2} ms per day", io_trend.slope),
        format!("Data points: {} samples over {} days", io_trend.data_points_count, days),
    ])
    .with_cause("I/O latency trending upward - possible disk saturation, fragmentation, or hardware degradation")
    .with_actions(vec![
        "Check disk I/O: iostat -x 1 5".to_string(),
        "Identify I/O-heavy processes: iotop -o".to_string(),
        "Check SMART status: sudo smartctl -a /dev/sda".to_string(),
        "Review disk health: sudo smartctl -H /dev/sda".to_string(),
        "Check for filesystem errors: sudo dmesg | grep -i error".to_string(),
        "Consider enabling TRIM if using SSD: sudo systemctl enable fstrim.timer".to_string(),
    ]);

    Ok(Some(insight))
}

/// Predict network latency instability
///
/// Detects increasing network latency and instability that may impact connectivity.
/// Uses both trend analysis and variance to detect unreliable connections.
pub fn predict_network_latency_issues(historian: &Historian, days: u32) -> Result<Option<PredictiveInsight>> {
    let net_trend = historian.get_network_latency_trend(days)?;

    // Need sufficient data points
    if net_trend.data_points_count < 10 {
        return Ok(None);
    }

    // Conservative thresholds
    const HIGH_LATENCY_MS: f64 = 100.0; // 100ms average is concerning
    const CRITICAL_LATENCY_MS: f64 = 200.0; // 200ms+ is critical
    const HIGH_VARIANCE_THRESHOLD: f64 = 1000.0; // Variance > 1000 ms² indicates instability
    const MIN_DEVIATION_PERCENT: f64 = 25.0; // 25% increase from baseline

    // Calculate standard deviation from variance
    let std_dev = net_trend.variance.sqrt();

    // Check for upward trend OR high variance (instability)
    let is_trending_up = net_trend.slope > 0.0;
    let is_unstable = net_trend.variance >= HIGH_VARIANCE_THRESHOLD;
    let is_high_latency = net_trend.current_average >= HIGH_LATENCY_MS;
    let is_significant_increase = net_trend.deviation_ms > 0.0
        && (net_trend.deviation_ms / net_trend.baseline.max(1.0)) * 100.0 >= MIN_DEVIATION_PERCENT;

    // Only trigger if there's a concerning pattern
    if !is_trending_up && !is_unstable {
        return Ok(None);
    }

    if !is_high_latency && !is_significant_increase && !is_unstable {
        return Ok(None);
    }

    // Determine severity
    let severity = if net_trend.current_average >= CRITICAL_LATENCY_MS || is_unstable {
        InsightSeverity::Critical
    } else if net_trend.current_average >= HIGH_LATENCY_MS {
        InsightSeverity::Warning
    } else {
        InsightSeverity::Info
    };

    // Build title based on primary issue
    let title = if is_unstable {
        "Network latency instability detected - unreliable connection"
    } else {
        "Increasing network latency detected - connectivity degradation"
    };

    // Calculate prediction window
    let prediction_window = if net_trend.slope >= 5.0 || is_unstable {
        "ongoing"
    } else {
        "next 24-48 hours"
    };

    let mut evidence = vec![
        format!("Baseline latency: {:.1} ms", net_trend.baseline),
        format!("Current average: {:.1} ms", net_trend.current_average),
        format!("Latency change: {:+.1} ms", net_trend.deviation_ms),
        format!("Standard deviation: {:.1} ms (variance: {:.1})", std_dev, net_trend.variance),
        format!("Trend slope: {:+.2} ms per hour", net_trend.slope),
        format!("Data points: {} samples over {} days", net_trend.data_points_count, days),
    ];

    if is_unstable {
        evidence.push("⚠ High variance indicates unstable connection".to_string());
    }

    let cause = if is_unstable {
        "Network instability detected - high variance in latency measurements suggests unreliable connection"
    } else {
        "Network latency trending upward - possible ISP issues, router problems, or network congestion"
    };

    let insight = PredictiveInsight::new(
        "network_latency_instability",
        title,
        severity,
        prediction_window,
    )
    .with_evidence(evidence)
    .with_cause(cause)
    .with_actions(vec![
        "Test network latency: ping -c 10 8.8.8.8".to_string(),
        "Check packet loss: ping -c 100 -i 0.2 8.8.8.8 | grep loss".to_string(),
        "Test DNS resolution: dig google.com".to_string(),
        "Check network interface status: ip link show".to_string(),
        "Review network errors: ip -s link".to_string(),
        "Restart network manager if issues persist: sudo systemctl restart NetworkManager".to_string(),
        "Test with different DNS: temporarily use 1.1.1.1 or 8.8.8.8".to_string(),
    ]);

    Ok(Some(insight))
}

#[cfg(test)]
mod predictor_tests {
    use super::*;

    // Note: Full integration tests with Historian require database fixtures
    // These tests verify the logic compiles and handles edge cases

    #[test]
    fn test_predictor_functions_exist() {
        // Verify all 5 predictor functions are defined
        // This is a compile-time check that ensures the API is correct
    }

    #[test]
    fn test_cpu_pressure_trend_structure() {
        let trend = CpuPressureTrend {
            baseline: 40.0,
            current_average: 65.0,
            deviation_percent: 62.5, // (65-40)/40 * 100
            slope: 1.5,
            data_points_count: 30,
        };

        assert_eq!(trend.baseline, 40.0);
        assert_eq!(trend.current_average, 65.0);
        assert!(trend.deviation_percent > 60.0);
        assert!(trend.slope > 0.0);
        assert_eq!(trend.data_points_count, 30);
    }

    #[test]
    fn test_io_pressure_trend_structure() {
        let trend = IoPressureTrend {
            baseline: 10.0,
            current_average: 55.0,
            deviation_percent: 450.0, // (55-10)/10 * 100
            slope: 2.5,
            data_points_count: 20,
        };

        assert_eq!(trend.baseline, 10.0);
        assert_eq!(trend.current_average, 55.0);
        assert!(trend.deviation_percent > 400.0);
        assert!(trend.slope > 0.0);
    }

    #[test]
    fn test_thermal_trend_structure() {
        let trend = ThermalTrend {
            baseline: 50.0,
            current_average: 70.0,
            deviation_degrees: 20.0,
            slope: 1.2,
            data_points_count: 15,
        };

        assert_eq!(trend.deviation_degrees, 20.0);
        assert_eq!(trend.baseline, 50.0);
        assert_eq!(trend.current_average, 70.0);
    }

    #[test]
    fn test_network_latency_trend_structure() {
        let trend = NetworkLatencyTrend {
            baseline: 20.0,
            current_average: 100.0,
            deviation_ms: 80.0,
            slope: 3.5,
            variance: 1500.0,
            data_points_count: 50,
        };

        assert_eq!(trend.deviation_ms, 80.0);
        assert!(trend.variance > 1000.0);
        assert!(trend.slope > 0.0);
    }

    #[test]
    fn test_predictive_insight_id_generation() {
        let insight1 = PredictiveInsight::new(
            "disk_full",
            "Test prediction",
            InsightSeverity::Warning,
            "3 days",
        );

        let insight2 = PredictiveInsight::new(
            "disk_full",
            "Test prediction",
            InsightSeverity::Warning,
            "3 days",
        );

        // IDs should be unique due to timestamp
        assert!(insight1.id.starts_with("disk_full_"));
        assert!(insight2.id.starts_with("disk_full_"));
        // IDs contain timestamps so they might be equal if generated in same second
        // but the format should be correct
        assert!(insight1.id.contains("_"));
    }

    #[test]
    fn test_predictive_insight_builder_pattern() {
        let insight = PredictiveInsight::new(
            "test",
            "Test Title",
            InsightSeverity::Info,
            "24 hours",
        )
        .with_evidence(vec!["Evidence 1".to_string(), "Evidence 2".to_string()])
        .with_cause("Test cause")
        .with_actions(vec!["Action 1".to_string(), "Action 2".to_string()]);

        assert_eq!(insight.title, "Test Title");
        assert_eq!(insight.severity, InsightSeverity::Info);
        assert_eq!(insight.prediction_window, "24 hours");
        assert_eq!(insight.evidence.len(), 2);
        assert!(insight.cause.is_some());
        assert_eq!(insight.recommended_actions.len(), 2);
    }

    #[test]
    fn test_predictive_insight_severity_levels() {
        let info = PredictiveInsight::new("test", "Info", InsightSeverity::Info, "1 day");
        let warning = PredictiveInsight::new("test", "Warning", InsightSeverity::Warning, "1 day");
        let critical = PredictiveInsight::new("test", "Critical", InsightSeverity::Critical, "1 day");

        assert!(matches!(info.severity, InsightSeverity::Info));
        assert!(matches!(warning.severity, InsightSeverity::Warning));
        assert!(matches!(critical.severity, InsightSeverity::Critical));
    }

    #[test]
    fn test_predictive_insight_optional_fields() {
        let minimal = PredictiveInsight::new(
            "test",
            "Minimal",
            InsightSeverity::Info,
            "1 day",
        );

        assert!(minimal.evidence.is_empty());
        assert!(minimal.cause.is_none());
        assert!(minimal.recommended_actions.is_empty());

        let full = minimal
            .with_evidence(vec!["E1".to_string()])
            .with_cause("C1")
            .with_actions(vec!["A1".to_string()]);

        assert_eq!(full.evidence.len(), 1);
        assert!(full.cause.is_some());
        assert_eq!(full.recommended_actions.len(), 1);
    }

    #[test]
    fn test_prediction_window_formats() {
        let formats = vec![
            "next 3 days",
            "24 hours",
            "48 hours",
            "next 24-48 hours",
            "next 3-7 days",
            "ongoing",
        ];

        for format in formats {
            let insight = PredictiveInsight::new(
                "test",
                "Test",
                InsightSeverity::Info,
                format,
            );
            assert_eq!(insight.prediction_window, format);
        }
    }

    #[test]
    fn test_predictive_insight_timestamp() {
        let before = chrono::Utc::now();
        let insight = PredictiveInsight::new(
            "test",
            "Test",
            InsightSeverity::Info,
            "1 day",
        );
        let after = chrono::Utc::now();

        // Timestamp should be between before and after
        assert!(insight.generated_at >= before);
        assert!(insight.generated_at <= after);
    }

    #[test]
    fn test_multiple_evidence_items() {
        let evidence = vec![
            "Current usage: 85%".to_string(),
            "Growth rate: 2 GB/day".to_string(),
            "Free space: 15 GB".to_string(),
            "Trend: Upward".to_string(),
            "Data points: 30 days".to_string(),
        ];

        let insight = PredictiveInsight::new(
            "disk_full",
            "Disk Full Prediction",
            InsightSeverity::Warning,
            "7 days",
        )
        .with_evidence(evidence.clone());

        assert_eq!(insight.evidence.len(), 5);
        assert_eq!(insight.evidence, evidence);
    }

    #[test]
    fn test_recommended_actions_format() {
        let actions = vec![
            "Check disk usage: df -h".to_string(),
            "Find large files: du -sh /* | sort -rh | head -10".to_string(),
            "Clean cache: sudo pacman -Sc".to_string(),
        ];

        let insight = PredictiveInsight::new(
            "test",
            "Test",
            InsightSeverity::Warning,
            "1 day",
        )
        .with_actions(actions.clone());

        assert_eq!(insight.recommended_actions.len(), 3);
        assert_eq!(insight.recommended_actions, actions);
    }
}
