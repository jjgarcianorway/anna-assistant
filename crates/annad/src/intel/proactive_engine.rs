//! Proactive Engine - Deterministic Correlation and Root-Cause Analysis
//!
//! Beta.270: Proactive Sysadmin Autonomy Level 1
//!
//! This module implements deterministic correlation of multiple telemetry signals
//! into root-cause diagnoses with actionable remediation guidance.
//!
//! Architecture principles:
//! - 100% deterministic (zero LLM involvement)
//! - Rule-based correlation following ROOT_CAUSE_CORRELATION_MATRIX.md
//! - Temporal awareness (15min, 1h, 24h windows)
//! - Confidence-based filtering (only >= 0.7 surfaced)
//! - Recovery tracking with 24h TTL
//!
//! Citation: [PROACTIVE_ENGINE_DESIGN.md]

use super::sysadmin_brain::DiagnosticInsight;
use crate::steward::HealthReport;
use anna_common::network_monitoring::NetworkMonitoring;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Minimum confidence to surface an issue (70%)
const MIN_CONFIDENCE: f32 = 0.7;

/// Health score weights
const WEIGHT_CRITICAL: u8 = 20;
const WEIGHT_WARNING: u8 = 10;
const WEIGHT_TREND: u8 = 5;
const WEIGHT_FLAPPING: u8 = 3;

/// Maximum issues to track internally
const MAX_ISSUES: usize = 50;

/// Maximum issues to display
pub const MAX_DISPLAYED_ISSUES: usize = 10;

/// Recovery notice TTL (24 hours)
const RECOVERY_TTL_HOURS: i64 = 24;

/// Temporal windows
const WINDOW_SHORT_MINUTES: i64 = 15;
const WINDOW_MEDIUM_MINUTES: i64 = 60;
const WINDOW_DAILY_HOURS: i64 = 24;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

/// Complete proactive assessment of system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveAssessment {
    /// When this assessment was generated
    pub timestamp: DateTime<Utc>,

    /// Correlated issues (root causes + symptoms)
    pub correlated_issues: Vec<CorrelatedIssue>,

    /// Detected trends (degradation, improvement, flapping)
    pub trends: Vec<TrendObservation>,

    /// Recovery notices (issues resolved since last check)
    pub recoveries: Vec<RecoveryNotice>,

    /// Overall system health score (0-100)
    pub health_score: u8,

    /// Highest severity present
    pub max_severity: IssueSeverity,

    /// Total issue count by severity
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub trend_count: usize,
}

/// A correlated issue with root-cause analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedIssue {
    /// Unique correlation ID
    pub correlation_id: String,

    /// Root cause identified
    pub root_cause: RootCause,

    /// All contributing signals
    pub contributing_signals: Vec<Signal>,

    /// Severity (highest from all signals)
    pub severity: IssueSeverity,

    /// Human-readable summary
    pub summary: String,

    /// Technical details
    pub details: String,

    /// Remediation steps (shell commands)
    pub remediation_commands: Vec<String>,

    /// Confidence level (0.0-1.0)
    pub confidence: f32,

    /// First detected timestamp
    pub first_seen: DateTime<Utc>,

    /// Last updated timestamp
    pub last_seen: DateTime<Utc>,
}

/// Root cause categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RootCause {
    // Network root causes
    NetworkRoutingConflict {
        duplicate_routes: Vec<String>,
    },
    NetworkPriorityMismatch {
        slow_interface: String,
        fast_interface: String,
        slow_speed_mbps: u32,
        fast_speed_mbps: u32,
    },
    NetworkQualityDegradation {
        packet_loss_percent: Option<f64>,
        latency_ms: Option<f64>,
        interface_errors: Option<u32>,
    },

    // Disk root causes
    DiskPressure {
        mountpoint: String,
        usage_percent: u8,
        inode_exhaustion: bool,
    },
    DiskLogGrowth {
        log_path: String,
        growth_rate_mb_per_hour: f64,
    },

    // Service root causes
    ServiceFlapping {
        service_name: String,
        restart_count: u32,
        window_minutes: u32,
    },
    ServiceUnderLoad {
        service_name: String,
        cpu_percent: f64,
        memory_mb: u64,
    },
    ServiceConfigError {
        service_name: String,
        error_message: String,
        exit_code: i32,
    },

    // Resource root causes
    MemoryPressure {
        ram_percent: f64,
        swap_percent: Option<f64>,
    },
    CpuOverload {
        load_per_core: f64,
        runaway_process: Option<String>,
    },

    // System root causes
    KernelRegression {
        old_version: String,
        new_version: String,
        degradation_symptoms: String,
    },
    DeviceHotplug {
        added: Vec<String>,
        removed: Vec<String>,
    },
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info = 0,
    Trend = 1,
    Warning = 2,
    Critical = 3,
}

/// Individual telemetry signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Where signal came from
    pub source: SignalSource,

    /// What was detected
    pub observation: String,

    /// Raw telemetry value
    pub value: SignalValue,

    /// When it was observed
    pub timestamp: DateTime<Utc>,

    /// Confidence of this signal (0.0-1.0)
    pub confidence: f32,
}

/// Signal source type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalSource {
    BrainInsight { rule_id: String },
    HealthReport { subsystem: String },
    NetworkMonitoring { metric: String },
    SystemdJournal { unit: String },
    ProcessMonitoring { process: String },
}

/// Signal value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalValue {
    Boolean(bool),
    Percentage(f64),
    Count(u32),
    Latency(f64), // milliseconds
    Text(String),
}

/// Trend observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendObservation {
    /// What is trending
    pub subject: String,

    /// Type of trend
    pub trend_type: TrendType,

    /// How long trend has been observed (minutes)
    pub duration_minutes: u32,

    /// Severity if trend continues
    pub projected_severity: IssueSeverity,

    /// Recommendation to prevent escalation
    pub recommendation: String,

    /// When trend was first detected
    pub first_detected: DateTime<Utc>,
}

/// Trend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendType {
    Escalating,   // Getting worse
    Flapping,     // Oscillating
    Degrading,    // Slowly declining
    Improving,    // Getting better
    Recurring,    // Pattern repeats
}

/// Recovery notice for resolved issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryNotice {
    /// What recovered
    pub subject: String,

    /// When it recovered
    pub recovery_time: DateTime<Utc>,

    /// How long it was an issue (hours)
    pub duration_hours: u32,

    /// What fixed it (if known)
    pub resolution: Option<String>,

    /// Original severity
    pub original_severity: IssueSeverity,
}

// ============================================================================
// INPUT STRUCTURES
// ============================================================================

/// Input data for proactive assessment
pub struct ProactiveInput<'a> {
    /// Current health report
    pub current_health: &'a HealthReport,

    /// Brain insights from current analysis
    pub brain_insights: &'a [DiagnosticInsight],

    /// Network monitoring data
    pub network_monitoring: Option<&'a NetworkMonitoring>,

    /// Previous assessment (for trend/recovery detection)
    pub previous_assessment: Option<&'a ProactiveAssessment>,

    /// Historian data (limited scope for Beta.270)
    pub historian_context: Option<HistorianContext>,
}

/// Historian context (limited scope)
#[derive(Debug, Clone)]
pub struct HistorianContext {
    /// Recent kernel changes (package updates)
    pub kernel_changes: Vec<KernelChange>,

    /// Recent boot events
    pub boot_events: Vec<BootEvent>,

    /// Service state changes
    pub service_changes: Vec<ServiceChange>,
}

#[derive(Debug, Clone)]
pub struct KernelChange {
    pub timestamp: DateTime<Utc>,
    pub old_version: String,
    pub new_version: String,
}

#[derive(Debug, Clone)]
pub struct BootEvent {
    pub timestamp: DateTime<Utc>,
    pub error_count: u32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceChange {
    pub timestamp: DateTime<Utc>,
    pub service_name: String,
    pub change_type: String, // "restart", "enable", "disable", "fail"
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

/// Compute proactive assessment from current telemetry
///
/// This is the main entry point for the proactive engine. It correlates
/// multiple signals, detects trends, tracks recoveries, and computes a
/// health score.
///
/// Returns ProactiveAssessment with only high-confidence issues (>= 0.7).
pub fn compute_proactive_assessment(input: &ProactiveInput) -> ProactiveAssessment {
    let now = Utc::now();

    // Step 1: Collect all signals from various sources
    let signals = collect_signals(input);

    // Step 2: Run correlation rules to identify root causes
    let mut correlated_issues = correlate_signals(&signals, input, now);

    // Step 3: Filter by confidence threshold
    correlated_issues.retain(|issue| issue.confidence >= MIN_CONFIDENCE);

    // Step 4: Deduplicate correlated issues
    let correlated_issues = deduplicate_issues(correlated_issues);

    // Step 5: Detect trends
    let trends = detect_trends(&correlated_issues, input, now);

    // Step 6: Detect recoveries
    let recoveries = detect_recoveries(&correlated_issues, input, now);

    // Step 7: Sort issues by severity and confidence
    let mut sorted_issues = correlated_issues;
    sorted_issues.sort_by(|a, b| {
        b.severity.cmp(&a.severity)
            .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Step 8: Cap at MAX_ISSUES for internal tracking
    if sorted_issues.len() > MAX_ISSUES {
        sorted_issues.truncate(MAX_ISSUES);
    }

    // Step 9: Count by severity
    let critical_count = sorted_issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
    let warning_count = sorted_issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();
    let info_count = sorted_issues.iter().filter(|i| i.severity == IssueSeverity::Info).count();
    let trend_count = trends.len();

    // Step 10: Determine max severity
    let max_severity = sorted_issues
        .first()
        .map(|i| i.severity)
        .unwrap_or(IssueSeverity::Info);

    // Step 11: Calculate health score
    let health_score = calculate_health_score(
        critical_count,
        warning_count,
        &trends,
    );

    ProactiveAssessment {
        timestamp: now,
        correlated_issues: sorted_issues,
        trends,
        recoveries,
        health_score,
        max_severity,
        critical_count,
        warning_count,
        info_count,
        trend_count,
    }
}

// ============================================================================
// SIGNAL COLLECTION
// ============================================================================

/// Collect all signals from input sources
fn collect_signals(input: &ProactiveInput) -> Vec<Signal> {
    let mut signals = Vec::new();
    let now = Utc::now();

    // Signals from brain insights
    for insight in input.brain_insights {
        signals.push(Signal {
            source: SignalSource::BrainInsight {
                rule_id: insight.rule_id.clone(),
            },
            observation: insight.summary.clone(),
            value: SignalValue::Text(insight.evidence.clone()),
            timestamp: now,
            confidence: 0.9, // Brain insights are high confidence
        });
    }

    // Signals from health report (services)
    for service in &input.current_health.services {
        if service.state == "failed" || service.state == "degraded" {
            signals.push(Signal {
                source: SignalSource::HealthReport {
                    subsystem: "systemd".to_string(),
                },
                observation: format!("Service {} is {}", service.name, service.state),
                value: SignalValue::Text(service.state.clone()),
                timestamp: now,
                confidence: 0.95, // Direct systemd state is very reliable
            });
        }
    }

    // Signals from network monitoring
    if let Some(network) = input.network_monitoring {
        // Check for duplicate default routes
        let default_route_count = network.routes.iter()
            .filter(|r| r.destination == "default")
            .count();

        if default_route_count > 1 {
            signals.push(Signal {
                source: SignalSource::NetworkMonitoring {
                    metric: "duplicate_default_routes".to_string(),
                },
                observation: format!("{} default routes detected", default_route_count),
                value: SignalValue::Count(default_route_count as u32),
                timestamp: now,
                confidence: 1.0, // Routing table is ground truth
            });
        }

        // Check for packet loss
        for iface in &network.interfaces {
            let total_packets = iface.stats.rx_packets + iface.stats.tx_packets;
            if total_packets > 0 {
                let error_rate = (iface.stats.rx_errors + iface.stats.tx_errors) as f64 / total_packets as f64 * 100.0;
                if error_rate > 1.0 {
                    signals.push(Signal {
                        source: SignalSource::NetworkMonitoring {
                            metric: "interface_errors".to_string(),
                        },
                        observation: format!("Interface {} has {:.1}% error rate", iface.name, error_rate),
                        value: SignalValue::Percentage(error_rate),
                        timestamp: now,
                        confidence: 0.8,
                    });
                }
            }
        }
    }

    signals
}

// ============================================================================
// CORRELATION RULES
// ============================================================================

/// Correlate signals into root-cause issues
fn correlate_signals(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Run all correlation rules
    issues.extend(correlate_network_routing_conflict(signals, input, now));
    issues.extend(correlate_network_priority_mismatch(signals, input, now));
    issues.extend(correlate_network_quality_degradation(signals, input, now));
    issues.extend(correlate_disk_pressure(signals, input, now));
    issues.extend(correlate_service_flapping(signals, input, now));
    issues.extend(correlate_service_config_error(signals, input, now));
    issues.extend(correlate_memory_pressure(signals, input, now));
    issues.extend(correlate_cpu_overload(signals, input, now));
    issues.extend(correlate_kernel_regression(signals, input, now)); // Beta.279: SYS-001

    issues
}

/// Rule NET-001: Network Routing Conflict Detection
fn correlate_network_routing_conflict(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Check for duplicate_default_routes brain insight OR direct signal
    let has_duplicate_routes_brain = input.brain_insights.iter()
        .any(|i| i.rule_id == "duplicate_default_routes");

    let duplicate_route_signal = signals.iter()
        .find(|s| matches!(&s.source, SignalSource::NetworkMonitoring { metric } if metric == "duplicate_default_routes"));

    if !has_duplicate_routes_brain && duplicate_route_signal.is_none() {
        return issues;
    }

    // Collect interface names with default routes
    let mut duplicate_routes = Vec::new();
    if let Some(network) = input.network_monitoring {
        for route in &network.routes {
            if route.destination == "default" {
                duplicate_routes.push(route.interface.clone());
            }
        }
    }

    if duplicate_routes.len() < 2 {
        return issues;
    }

    // Calculate confidence
    let mut confidence = 0.8; // Base confidence

    // +10% if packet loss present
    if signals.iter().any(|s| s.observation.contains("packet loss") || s.observation.contains("error rate")) {
        confidence += 0.1;
    }

    // Collect contributing signals
    let mut contributing_signals = Vec::new();
    if let Some(insight_signal) = signals.iter().find(|s| {
        matches!(&s.source, SignalSource::BrainInsight { rule_id } if rule_id == "duplicate_default_routes")
    }) {
        contributing_signals.push(insight_signal.clone());
    }
    if let Some(route_signal) = duplicate_route_signal {
        contributing_signals.push(route_signal.clone());
    }

    // Determine severity
    let severity = if confidence > 0.85 {
        IssueSeverity::Critical
    } else {
        IssueSeverity::Warning
    };

    let correlation_id = format!("NET-001-{}", now.timestamp());

    issues.push(CorrelatedIssue {
        correlation_id,
        root_cause: RootCause::NetworkRoutingConflict {
            duplicate_routes: duplicate_routes.clone(),
        },
        contributing_signals,
        severity,
        summary: format!("Duplicate default routes detected on interfaces: {}", duplicate_routes.join(", ")),
        details: format!(
            "Multiple default routes are configured, causing unpredictable routing behavior. \
            This can result in connection timeouts, inconsistent DNS resolution, and VPN/firewall issues. \
            Only one default route should be active."
        ),
        remediation_commands: vec![
            "ip route".to_string(),
            "nmcli device status".to_string(),
            "sudo ip route del default via <gateway> dev <interface>".to_string(),
            "sudo systemctl restart NetworkManager".to_string(),
        ],
        confidence,
        first_seen: now,
        last_seen: now,
    });

    issues
}

/// Rule NET-002: Network Priority Mismatch Correlation
fn correlate_network_priority_mismatch(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Check for network_priority_mismatch brain insight
    let priority_mismatch_insight = input.brain_insights.iter()
        .find(|i| i.rule_id == "network_priority_mismatch");

    if priority_mismatch_insight.is_none() {
        return issues;
    }

    // Parse evidence to extract interface names and speeds
    // Evidence format: "Ethernet eth0 (100 Mbps) has default route, WiFi wlan0 (866 Mbps) does not"
    let insight = priority_mismatch_insight.unwrap();
    let evidence = &insight.evidence;

    // Simple parsing
    let (slow_interface, slow_speed, fast_interface, fast_speed) =
        if let Some((eth_part, wifi_part)) = evidence.split_once(", WiFi") {
            // Extract eth info
            let eth_name = eth_part.split_whitespace()
                .nth(1)
                .unwrap_or("eth0")
                .to_string();
            let eth_speed = eth_part.split('(')
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(100);

            // Extract wifi info
            let wifi_name = wifi_part.trim()
                .split_whitespace()
                .next()
                .unwrap_or("wlan0")
                .to_string();
            let wifi_speed = wifi_part.split('(')
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(866);

            (eth_name, eth_speed, wifi_name, wifi_speed)
        } else {
            ("eth0".to_string(), 100, "wlan0".to_string(), 866)
        };

    let confidence = 0.9; // Brain already confirmed mismatch

    let correlation_id = format!("NET-002-{}", now.timestamp());

    issues.push(CorrelatedIssue {
        correlation_id,
        root_cause: RootCause::NetworkPriorityMismatch {
            slow_interface: slow_interface.clone(),
            fast_interface: fast_interface.clone(),
            slow_speed_mbps: slow_speed,
            fast_speed_mbps: fast_speed,
        },
        contributing_signals: vec![Signal {
            source: SignalSource::BrainInsight {
                rule_id: "network_priority_mismatch".to_string(),
            },
            observation: insight.summary.clone(),
            value: SignalValue::Text(insight.evidence.clone()),
            timestamp: now,
            confidence: 0.9,
        }],
        severity: IssueSeverity::Warning,
        summary: format!(
            "Network priority issue: {} ({}Mbps) prioritized over {} ({}Mbps)",
            slow_interface, slow_speed, fast_interface, fast_speed
        ),
        details: format!(
            "Your system is using a slower {} connection ({} Mbps) for routing instead of \
            a faster {} connection ({} Mbps). This typically happens when Ethernet is connected \
            via a USB adapter or dock while WiFi has better link quality. NetworkManager assigns \
            priority based on interface type by default, not speed.",
            slow_interface, slow_speed, fast_interface, fast_speed
        ),
        remediation_commands: vec![
            "nmcli connection show".to_string(),
            format!("nmcli connection down {}", slow_interface),
            "ip route".to_string(),
            format!("nmcli connection modify {} ipv4.route-metric 100", fast_interface),
            format!("nmcli connection modify {} ipv4.route-metric 200", slow_interface),
        ],
        confidence,
        first_seen: now,
        last_seen: now,
    });

    issues
}

/// Rule NET-003: Network Quality Degradation
fn correlate_network_quality_degradation(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Check for high_packet_loss or high_latency brain insights
    let packet_loss_insight = input.brain_insights.iter()
        .find(|i| i.rule_id == "high_packet_loss");

    let latency_insight = input.brain_insights.iter()
        .find(|i| i.rule_id == "high_latency");

    // Check for interface error signals
    let error_signals: Vec<&Signal> = signals.iter()
        .filter(|s| matches!(&s.source, SignalSource::NetworkMonitoring { metric } if metric == "interface_errors"))
        .collect();

    // Beta.279: NET-003: Check historian for network degradation trend
    let events = load_recent_history(Duration::hours(1));
    let has_trend = !events.is_empty() && detect_network_trend(&events).is_some();

    if packet_loss_insight.is_none() && latency_insight.is_none() && error_signals.is_empty() && !has_trend {
        return issues;
    }

    // Don't correlate if we already have routing conflicts
    if input.brain_insights.iter().any(|i| i.rule_id == "duplicate_default_routes") {
        return issues;
    }

    let mut packet_loss: Option<f64> = None;
    let mut latency: Option<f64> = None;
    let mut interface_errors: Option<u32> = None;
    let mut contributing_signals = Vec::new();

    // Extract packet loss
    if let Some(insight) = packet_loss_insight {
        // Parse "High packet loss detected: 35%"
        if let Some(pct_str) = insight.summary.split(':').nth(1) {
            if let Some(pct) = pct_str.trim().trim_end_matches('%').parse::<f64>().ok() {
                packet_loss = Some(pct);
            }
        }
        contributing_signals.push(Signal {
            source: SignalSource::BrainInsight { rule_id: insight.rule_id.clone() },
            observation: insight.summary.clone(),
            value: SignalValue::Percentage(packet_loss.unwrap_or(0.0)),
            timestamp: now,
            confidence: 0.8,
        });
    }

    // Extract latency
    if let Some(insight) = latency_insight {
        // Parse "High latency detected: 350ms"
        if let Some(ms_str) = insight.summary.split(':').nth(1) {
            if let Some(ms) = ms_str.trim().trim_end_matches("ms").parse::<f64>().ok() {
                latency = Some(ms);
            }
        }
        contributing_signals.push(Signal {
            source: SignalSource::BrainInsight { rule_id: insight.rule_id.clone() },
            observation: insight.summary.clone(),
            value: SignalValue::Latency(latency.unwrap_or(0.0)),
            timestamp: now,
            confidence: 0.7,
        });
    }

    // Count interface errors
    for sig in &error_signals {
        if let SignalValue::Percentage(pct) = sig.value {
            // Convert error rate percentage to approximate error count
            interface_errors = Some((pct * 100.0) as u32);
        }
        contributing_signals.push((*sig).clone());
    }

    // Beta.279: Add historian trend evidence
    if has_trend {
        let trend = detect_network_trend(&events).unwrap();
        let first_loss = events.first().unwrap().network_packet_loss_pct;
        let last_loss = events.last().unwrap().network_packet_loss_pct;
        let first_latency = events.first().unwrap().network_latency_ms;
        let last_latency = events.last().unwrap().network_latency_ms;

        contributing_signals.push(Signal {
            source: SignalSource::HealthReport {
                subsystem: "historian".to_string(),
            },
            observation: format!(
                "Network degradation trend: loss {}%→{}%, latency {}ms→{}ms over {}min",
                first_loss, last_loss, first_latency, last_latency, trend.duration_minutes
            ),
            value: SignalValue::Percentage(last_loss as f64),
            timestamp: events.first().unwrap().timestamp_utc,
            confidence: 0.85,
        });

        // Use historian data if current snapshot doesn't have values
        if packet_loss.is_none() {
            packet_loss = Some(last_loss as f64);
        }
        if latency.is_none() {
            latency = Some(last_latency as f64);
        }
    }

    // Calculate confidence
    let mut confidence = 0.0;
    let signal_count = [packet_loss.is_some(), latency.is_some(), interface_errors.is_some()]
        .iter()
        .filter(|&&x| x)
        .count();

    confidence += match signal_count {
        1 => if packet_loss.is_some() { 0.7 } else if latency.is_some() { 0.6 } else { 0.5 },
        2 => 0.85,
        3 => 0.9,
        _ => 0.5,
    };

    // Boost confidence if trend detected
    if has_trend {
        confidence = (confidence as f32 + 0.1_f32).min(0.95_f32);
    }

    if confidence < MIN_CONFIDENCE {
        return issues;
    }

    // Determine severity
    let severity = if packet_loss.unwrap_or(0.0) > 20.0 || latency.unwrap_or(0.0) > 500.0 {
        IssueSeverity::Critical
    } else {
        IssueSeverity::Warning
    };

    let correlation_id = format!("NET-003-{}", now.timestamp());

    let summary = if let Some(pl) = packet_loss {
        format!(
            "High packet loss detected ({:.1}%){}",
            pl,
            if has_trend { " (trending up)" } else { "" }
        )
    } else if let Some(lat) = latency {
        format!(
            "High latency detected ({:.0}ms){}",
            lat,
            if has_trend { " (trending up)" } else { "" }
        )
    } else {
        "Network quality degradation detected".to_string()
    };

    issues.push(CorrelatedIssue {
        correlation_id,
        root_cause: RootCause::NetworkQualityDegradation {
            packet_loss_percent: packet_loss,
            latency_ms: latency,
            interface_errors: interface_errors.map(|e| e.min(9999)),
        },
        contributing_signals,
        severity,
        summary,
        details: format!(
            "Network connection is experiencing quality issues. {}Common causes include WiFi interference, \
            weak signal strength, faulty cables, congested network, or overloaded router. This degrades \
            performance for real-time applications and can cause connection timeouts.",
            if has_trend {
                "Network quality has been degrading over the past hour. "
            } else {
                ""
            }
        ),
        remediation_commands: vec![
            "ping -c 20 $(ip route | grep default | awk '{print $3}')".to_string(),
            "ping -c 20 1.1.1.1".to_string(),
            "nmcli device wifi".to_string(),
            "ip -s link show".to_string(),
            "sudo systemctl restart NetworkManager".to_string(),
        ],
        confidence,
        first_seen: if has_trend {
            events.first().map(|e| e.timestamp_utc).unwrap_or(now)
        } else {
            now
        },
        last_seen: now,
    });

    issues
}

/// Rule DISK-001: Disk Pressure Detection
fn correlate_disk_pressure(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // DISK-001: Current disk pressure from brain insights
    for insight in input.brain_insights {
        if insight.rule_id != "disk_space_critical" && insight.rule_id != "disk_space_warning" {
            continue;
        }

        // Parse mountpoint and usage from summary
        // Format: "Disk usage on / is 92%"
        let (mountpoint, usage_percent) = if let Some(parts) = insight.summary.split(" on ").nth(1) {
            if let Some((mp, pct_str)) = parts.split_once(" is ") {
                let pct = pct_str.trim_end_matches('%').parse::<u8>().unwrap_or(85);
                (mp.to_string(), pct)
            } else {
                ("/".to_string(), 85)
            }
        } else {
            ("/".to_string(), 85)
        };

        // Check for inode exhaustion (heuristic: mentioned in details)
        let inode_exhaustion = insight.details.contains("inode") || insight.details.contains("Inode");

        let severity = if insight.rule_id == "disk_space_critical" {
            IssueSeverity::Critical
        } else {
            IssueSeverity::Warning
        };

        let mut confidence = 0.9; // Disk usage is reliable
        if inode_exhaustion {
            confidence += 0.05;
        }

        let correlation_id = format!("DISK-001-{}-{}", mountpoint.replace('/', "_"), now.timestamp());

        issues.push(CorrelatedIssue {
            correlation_id,
            root_cause: RootCause::DiskPressure {
                mountpoint: mountpoint.clone(),
                usage_percent,
                inode_exhaustion,
            },
            contributing_signals: vec![Signal {
                source: SignalSource::BrainInsight { rule_id: insight.rule_id.clone() },
                observation: insight.summary.clone(),
                value: SignalValue::Percentage(usage_percent as f64),
                timestamp: now,
                confidence: 0.9,
            }],
            severity,
            summary: if inode_exhaustion {
                format!("Disk pressure on {} ({}% full, inodes exhausted)", mountpoint, usage_percent)
            } else {
                format!("Disk pressure on {} ({}% full)", mountpoint, usage_percent)
            },
            details: format!(
                "Filesystem {} is {} full. This will cause system instability and application failures. \
                Common causes include package cache buildup, large log files, and accumulated temporary files.",
                mountpoint,
                if usage_percent >= 95 { "critically" } else { "approaching capacity" }
            ),
            remediation_commands: vec![
                format!("df -h {}", mountpoint),
                format!("df -i {}", mountpoint),
                format!("du -h {} | sort -h | tail -20", mountpoint),
                "sudo pacman -Sc".to_string(),
                "sudo journalctl --vacuum-size=100M".to_string(),
            ],
            confidence,
            first_seen: now,
            last_seen: now,
        });
    }

    // Beta.279: DISK-002: Disk growth trend detection using historian
    let events = load_recent_history(Duration::hours(24));
    if !events.is_empty() {
        if let Some(trend) = detect_disk_growth(&events) {
            let first_usage = events.first().unwrap().disk_root_usage_pct;
            let last_usage = events.last().unwrap().disk_root_usage_pct;
            let growth = last_usage.saturating_sub(first_usage);

            // Confidence based on growth rate and consistency
            let confidence = (0.7 + (growth as f32 * 0.01)).min(0.95);

            if confidence >= MIN_CONFIDENCE {
                let correlation_id = format!("DISK-002-growth-{}", now.timestamp());

                let mut contributing_signals = Vec::new();
                // Sample history events (first, middle, last)
                for (idx, event) in events.iter().enumerate() {
                    if idx == 0 || idx == events.len() / 2 || idx == events.len() - 1 {
                        contributing_signals.push(Signal {
                            source: SignalSource::HealthReport {
                                subsystem: "historian".to_string(),
                            },
                            observation: format!(
                                "Root disk usage: {}% at {}",
                                event.disk_root_usage_pct,
                                event.timestamp_utc.format("%H:%M")
                            ),
                            value: SignalValue::Percentage(event.disk_root_usage_pct as f64),
                            timestamp: event.timestamp_utc,
                            confidence: 0.9,
                        });
                    }
                }

                issues.push(CorrelatedIssue {
                    correlation_id,
                    root_cause: RootCause::DiskPressure {
                        mountpoint: "/".to_string(),
                        usage_percent: last_usage,
                        inode_exhaustion: false,
                    },
                    contributing_signals,
                    severity: trend.projected_severity,
                    summary: format!(
                        "Disk growth trend: +{}% over {}min (now {}%)",
                        growth, trend.duration_minutes, last_usage
                    ),
                    details: format!(
                        "Root disk usage has grown from {}% to {}% over the past {} minutes. \
                        This trend suggests ongoing log accumulation, package cache growth, or data generation. \
                        Without cleanup, the system will reach capacity.",
                        first_usage, last_usage, trend.duration_minutes
                    ),
                    remediation_commands: vec![
                        "df -h /".to_string(),
                        "du -h / | sort -h | tail -20".to_string(),
                        "sudo journalctl --vacuum-size=100M".to_string(),
                        "sudo pacman -Sc".to_string(),
                        "find /var/log -type f -mtime +30 -ls".to_string(),
                    ],
                    confidence,
                    first_seen: events.first().map(|e| e.timestamp_utc).unwrap_or(now),
                    last_seen: now,
                });
            }
        }
    }

    issues
}

/// Rule SVC-001: Service Flapping Detection
fn correlate_service_flapping(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Beta.279: Use historian to detect flapping (SVC-001)
    let events = load_recent_history(Duration::minutes(60));
    if events.is_empty() {
        // Fallback to heuristic if no history
        return issues;
    }

    // Detect flapping pattern using historian helper
    if let Some(trend) = detect_service_flapping(&events) {
        // Count transitions for confidence calculation
        let mut transitions = 0;
        let mut last_had_failures = events[0].failed_services_count > 0;
        for event in events.iter().skip(1) {
            let current_has_failures = event.failed_services_count > 0;
            if current_has_failures != last_had_failures {
                transitions += 1;
            }
            last_had_failures = current_has_failures;
        }

        // Base confidence on transition count
        let confidence = (0.7 + (transitions as f32 * 0.05)).min(0.95);

        // Only emit if confidence meets threshold
        if confidence >= MIN_CONFIDENCE {
            let correlation_id = format!("SVC-001-flapping-{}", now.timestamp());

            // Build evidence signals from history
            let mut contributing_signals = Vec::new();
            for (idx, event) in events.iter().enumerate().take(5) {
                contributing_signals.push(Signal {
                    source: SignalSource::HealthReport {
                        subsystem: "historian".to_string(),
                    },
                    observation: format!(
                        "Failed services: {} at {}",
                        event.failed_services_count,
                        event.timestamp_utc.format("%H:%M")
                    ),
                    value: SignalValue::Count(event.failed_services_count as u32),
                    timestamp: event.timestamp_utc,
                    confidence: 0.9,
                });
            }

            issues.push(CorrelatedIssue {
                correlation_id,
                root_cause: RootCause::ServiceFlapping {
                    service_name: "system services".to_string(),
                    restart_count: transitions,
                    window_minutes: trend.duration_minutes,
                },
                contributing_signals,
                severity: trend.projected_severity,
                summary: format!("Service flapping detected: {} transitions in {}min", transitions, trend.duration_minutes),
                details: format!(
                    "System services have shown unstable behavior with {} state transitions in the last {} minutes. \
                    This indicates recurring failures and restarts, suggesting dependency issues, configuration errors, \
                    or resource constraints.",
                    transitions, trend.duration_minutes
                ),
                remediation_commands: vec![
                    "systemctl list-units --state=failed".to_string(),
                    "journalctl -p err --since '1 hour ago'".to_string(),
                    "systemctl status".to_string(),
                ],
                confidence,
                first_seen: events.first().map(|e| e.timestamp_utc).unwrap_or(now),
                last_seen: now,
            });
        }
    }

    issues
}

/// Rule SVC-003: Service Configuration Error
fn correlate_service_config_error(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Check for failed_services brain insight
    let failed_insight = input.brain_insights.iter()
        .find(|i| i.rule_id == "failed_services");

    if failed_insight.is_none() {
        return issues;
    }

    let insight = failed_insight.unwrap();

    // Extract service names from details
    // Format: "• sshd.service - OpenSSH Daemon"
    for line in insight.details.lines() {
        if !line.contains(".service") {
            continue;
        }

        let service_name = line.split_whitespace()
            .find(|s| s.contains(".service"))
            .unwrap_or("unknown.service")
            .trim_start_matches('•')
            .trim()
            .to_string();

        // Heuristic: Assume config error if service recently changed or contains "config" in name
        // Full implementation would check exit codes from systemd journal
        let is_config_error = service_name.contains("config") || line.contains("configuration");

        if !is_config_error {
            // For Beta.270, we create a generic service failure issue
            // Config error detection requires systemd journal parsing
            continue;
        }

        let correlation_id = format!("SVC-003-{}-{}", service_name.replace(".service", ""), now.timestamp());

        issues.push(CorrelatedIssue {
            correlation_id,
            root_cause: RootCause::ServiceConfigError {
                service_name: service_name.clone(),
                error_message: "Configuration error detected".to_string(),
                exit_code: 1,
            },
            contributing_signals: vec![Signal {
                source: SignalSource::BrainInsight { rule_id: insight.rule_id.clone() },
                observation: format!("Service {} failed", service_name),
                value: SignalValue::Text("failed".to_string()),
                timestamp: now,
                confidence: 0.8,
            }],
            severity: IssueSeverity::Critical,
            summary: format!("Service configuration error: {}", service_name),
            details: format!(
                "Service {} has failed due to a configuration error. This prevents critical system functionality. \
                Check the service configuration file for syntax errors or invalid settings.",
                service_name
            ),
            remediation_commands: vec![
                format!("systemctl status {}", service_name),
                format!("journalctl -u {} -n 50 --no-pager", service_name),
                format!("systemctl cat {}", service_name),
            ],
            confidence: 0.8,
            first_seen: now,
            last_seen: now,
        });
    }

    issues
}

/// Rule RES-001: Memory Pressure Correlation
fn correlate_memory_pressure(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Check for memory_pressure brain insights (current snapshot)
    for insight in input.brain_insights {
        if insight.rule_id != "memory_pressure_critical" && insight.rule_id != "memory_pressure_warning" {
            continue;
        }

        // Parse memory percentage from summary
        // Format: "Memory usage at 94.2%"
        let ram_percent = if let Some(pct_str) = insight.summary.split(" at ").nth(1) {
            pct_str.trim_end_matches('%').parse::<f64>().unwrap_or(85.0)
        } else {
            85.0
        };

        // Parse swap percentage from details if present
        let swap_percent = if insight.details.contains("Swap at") || insight.details.contains("swap") {
            if let Some(swap_str) = insight.details.split("Swap at ").nth(1) {
                swap_str.split('%').next()
                    .and_then(|s| s.trim().parse::<f64>().ok())
            } else if insight.details.contains("No swap") {
                None
            } else {
                Some(0.0)
            }
        } else {
            None
        };

        let severity = if insight.rule_id == "memory_pressure_critical" {
            IssueSeverity::Critical
        } else {
            IssueSeverity::Warning
        };

        // Beta.279: Check historian to distinguish sustained vs spike
        let events = load_recent_history(Duration::hours(1));
        let sustained_pressure = if !events.is_empty() {
            detect_resource_pressure(&events)
                .filter(|t| t.subject == "Memory usage")
                .is_some()
        } else {
            false
        };

        let mut confidence = 0.85;
        if swap_percent.is_some() && swap_percent.unwrap() > 50.0 {
            confidence += 0.05;
        }
        if sustained_pressure {
            confidence += 0.1; // Higher confidence if sustained over time
        }

        let correlation_id = format!("RES-001-{}", now.timestamp());

        let mut contributing_signals = vec![Signal {
            source: SignalSource::BrainInsight { rule_id: insight.rule_id.clone() },
            observation: insight.summary.clone(),
            value: SignalValue::Percentage(ram_percent),
            timestamp: now,
            confidence: 0.85,
        }];

        // Add historian evidence if sustained
        if sustained_pressure && !events.is_empty() {
            let high_memory_count = events.iter().filter(|e| e.high_memory_flag).count();
            contributing_signals.push(Signal {
                source: SignalSource::HealthReport {
                    subsystem: "historian".to_string(),
                },
                observation: format!(
                    "High memory observed in {} of {} checks over past hour",
                    high_memory_count, events.len()
                ),
                value: SignalValue::Count(high_memory_count as u32),
                timestamp: events.first().unwrap().timestamp_utc,
                confidence: 0.9,
            });
        }

        issues.push(CorrelatedIssue {
            correlation_id,
            root_cause: RootCause::MemoryPressure {
                ram_percent,
                swap_percent,
            },
            contributing_signals,
            severity,
            summary: if let Some(swap) = swap_percent {
                format!(
                    "Memory pressure: {:.1}% RAM, {:.1}% swap{}",
                    ram_percent,
                    swap,
                    if sustained_pressure { " (sustained)" } else { "" }
                )
            } else {
                format!(
                    "Memory pressure: {:.1}% RAM (no swap){}",
                    ram_percent,
                    if sustained_pressure { " (sustained)" } else { "" }
                )
            },
            details: format!(
                "System RAM usage is at {:.1}%. {}. {}This can cause system slowdown, application crashes, \
                and OOM killer events. Identify memory-hungry processes and consider adding swap or \
                increasing available RAM.",
                ram_percent,
                if let Some(swap) = swap_percent {
                    format!("Swap usage at {:.1}%", swap)
                } else {
                    "No swap configured".to_string()
                },
                if sustained_pressure {
                    "This pressure has been sustained over the past hour. "
                } else {
                    ""
                }
            ),
            remediation_commands: vec![
                "free -h".to_string(),
                "swapon --show".to_string(),
                "ps aux --sort=-%mem | head -10".to_string(),
                "journalctl -p err | grep -i oom".to_string(),
            ],
            confidence,
            first_seen: now,
            last_seen: now,
        });
    }

    issues
}

/// Rule RES-002: CPU Overload Correlation
fn correlate_cpu_overload(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Check for CPU overload brain insights (current snapshot)
    for insight in input.brain_insights {
        if insight.rule_id != "cpu_overload_critical" && insight.rule_id != "cpu_high_load" {
            continue;
        }

        // Parse load from summary
        // Format: "CPU usage sustained at 98.5%" or "CPU load high"
        let load_per_core = if let Some(pct_str) = insight.summary.split(" at ").nth(1) {
            pct_str.trim_end_matches('%').parse::<f64>().unwrap_or(2.0) / 100.0 * 4.0 // estimate
        } else {
            2.0
        };

        // Check for runaway process in details
        let runaway_process = if insight.details.contains("process") {
            insight.details.lines()
                .find(|l| l.contains("process"))
                .and_then(|l| l.split_whitespace().next())
                .map(|s| s.to_string())
        } else {
            None
        };

        let severity = if insight.rule_id == "cpu_overload_critical" {
            IssueSeverity::Critical
        } else {
            IssueSeverity::Warning
        };

        // Beta.279: Check historian to distinguish sustained vs spike
        let events = load_recent_history(Duration::hours(1));
        let sustained_overload = if !events.is_empty() {
            detect_resource_pressure(&events)
                .filter(|t| t.subject == "CPU load")
                .is_some()
        } else {
            false
        };

        let mut confidence = 0.8;
        if runaway_process.is_some() {
            confidence += 0.15;
        }
        if sustained_overload {
            confidence += 0.1; // Higher confidence if sustained over time
        }

        let correlation_id = format!("RES-002-{}", now.timestamp());

        let mut contributing_signals = vec![Signal {
            source: SignalSource::BrainInsight { rule_id: insight.rule_id.clone() },
            observation: insight.summary.clone(),
            value: SignalValue::Percentage(load_per_core * 100.0),
            timestamp: now,
            confidence: 0.8,
        }];

        // Add historian evidence if sustained
        if sustained_overload && !events.is_empty() {
            let high_cpu_count = events.iter().filter(|e| e.high_cpu_flag).count();
            contributing_signals.push(Signal {
                source: SignalSource::HealthReport {
                    subsystem: "historian".to_string(),
                },
                observation: format!(
                    "High CPU observed in {} of {} checks over past hour",
                    high_cpu_count, events.len()
                ),
                value: SignalValue::Count(high_cpu_count as u32),
                timestamp: events.first().unwrap().timestamp_utc,
                confidence: 0.9,
            });
        }

        issues.push(CorrelatedIssue {
            correlation_id,
            root_cause: RootCause::CpuOverload {
                load_per_core,
                runaway_process: runaway_process.clone(),
            },
            contributing_signals,
            severity,
            summary: if let Some(proc) = &runaway_process {
                format!(
                    "CPU overload: {:.1} load per core (process: {}){}",
                    load_per_core,
                    proc,
                    if sustained_overload { " (sustained)" } else { "" }
                )
            } else {
                format!(
                    "CPU overload: {:.1} load per core{}",
                    load_per_core,
                    if sustained_overload { " (sustained)" } else { "" }
                )
            },
            details: format!(
                "CPU load is critically high ({:.1} per core). {}. {}This causes system slowdown and \
                can lead to unresponsiveness. Identify and address CPU-intensive processes.",
                load_per_core,
                if let Some(proc) = &runaway_process {
                    format!("Runaway process identified: {}", proc)
                } else {
                    "Check for runaway processes".to_string()
                },
                if sustained_overload {
                    "This overload has been sustained over the past hour. "
                } else {
                    ""
                }
            ),
            remediation_commands: vec![
                "uptime".to_string(),
                "top -o %CPU".to_string(),
                "ps aux --sort=-%cpu | head -10".to_string(),
            ],
            confidence,
            first_seen: now,
            last_seen: now,
        });
    }

    issues
}

/// Rule SYS-001: Kernel Regression Detection
/// Beta.279: Detects health degradation after kernel upgrade
fn correlate_kernel_regression(
    signals: &[Signal],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<CorrelatedIssue> {
    let mut issues = Vec::new();

    // Beta.279: Use historian to detect kernel regression pattern
    let events = load_recent_history(Duration::hours(24));
    if events.is_empty() {
        return issues;
    }

    // Check if there's a kernel change followed by degradation
    if let Some(trend) = detect_kernel_regression(&events) {
        // Find the kernel change event
        let kernel_change_idx = events.iter()
            .position(|e| e.kernel_changed)
            .unwrap();

        let before_events = &events[..kernel_change_idx];
        let after_events = &events[kernel_change_idx + 1..];

        let before_avg_failures = before_events.iter()
            .map(|e| (e.failed_services_count + e.degraded_services_count) as f64)
            .sum::<f64>() / before_events.len() as f64;

        let after_avg_failures = after_events.iter()
            .map(|e| (e.failed_services_count + e.degraded_services_count) as f64)
            .sum::<f64>() / after_events.len() as f64;

        let failure_increase = after_avg_failures - before_avg_failures;

        // Confidence based on magnitude of degradation
        let confidence = (0.7 + (failure_increase * 0.05)) as f32;
        let confidence = confidence.min(0.95);

        if confidence >= MIN_CONFIDENCE {
            let correlation_id = format!("SYS-001-kernel-regression-{}", now.timestamp());

            let old_kernel = before_events.last()
                .map(|e| e.kernel_version.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let new_kernel = after_events.first()
                .map(|e| e.kernel_version.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let mut contributing_signals = Vec::new();

            // Add evidence signals
            contributing_signals.push(Signal {
                source: SignalSource::HealthReport {
                    subsystem: "historian".to_string(),
                },
                observation: format!(
                    "Kernel upgrade detected: {} → {}",
                    old_kernel, new_kernel
                ),
                value: SignalValue::Text(new_kernel.clone()),
                timestamp: events[kernel_change_idx].timestamp_utc,
                confidence: 1.0,
            });

            contributing_signals.push(Signal {
                source: SignalSource::HealthReport {
                    subsystem: "historian".to_string(),
                },
                observation: format!(
                    "Average service failures before upgrade: {:.1}, after: {:.1}",
                    before_avg_failures, after_avg_failures
                ),
                value: SignalValue::Count(after_avg_failures as u32),
                timestamp: after_events.first().unwrap().timestamp_utc,
                confidence: 0.9,
            });

            issues.push(CorrelatedIssue {
                correlation_id,
                root_cause: RootCause::KernelRegression {
                    old_version: old_kernel.clone(),
                    new_version: new_kernel.clone(),
                    degradation_symptoms: format!(
                        "Service failures increased by {:.1} on average",
                        failure_increase
                    ),
                },
                contributing_signals,
                severity: if failure_increase > 3.0 {
                    IssueSeverity::Critical
                } else {
                    trend.projected_severity
                },
                summary: format!(
                    "Kernel regression: degradation after upgrade to {}",
                    new_kernel
                ),
                details: format!(
                    "System health degraded after kernel upgrade from {} to {}. \
                    Service failures increased by {:.1} on average over {} observations. \
                    This suggests a kernel regression. Consider rolling back or checking kernel logs.",
                    old_kernel, new_kernel, failure_increase, after_events.len()
                ),
                remediation_commands: vec![
                    "journalctl -k --since '24 hours ago' | grep -i error".to_string(),
                    "journalctl -k --since '24 hours ago' | grep -i fail".to_string(),
                    "uname -r".to_string(),
                    "dmesg | tail -50".to_string(),
                    "# Consider: grub-reboot to previous kernel".to_string(),
                ],
                confidence,
                first_seen: events[kernel_change_idx].timestamp_utc,
                last_seen: now,
            });
        }
    }

    issues
}

// ============================================================================
// DEDUPLICATION
// ============================================================================

/// Deduplicate correlated issues
fn deduplicate_issues(mut issues: Vec<CorrelatedIssue>) -> Vec<CorrelatedIssue> {
    let mut deduplicated: Vec<CorrelatedIssue> = Vec::new();

    while let Some(issue) = issues.pop() {
        // Find matching issue by root cause type
        if let Some(existing) = deduplicated.iter_mut().find(|i| {
            std::mem::discriminant(&i.root_cause) == std::mem::discriminant(&issue.root_cause)
        }) {
            // Merge signals
            existing.contributing_signals.extend(issue.contributing_signals);

            // Update confidence (weighted average favoring higher)
            existing.confidence = existing.confidence.max(issue.confidence);

            // Update timestamps
            existing.last_seen = existing.last_seen.max(issue.last_seen);
            existing.first_seen = existing.first_seen.min(issue.first_seen);
        } else {
            deduplicated.push(issue);
        }
    }

    deduplicated
}

// ============================================================================
// TREND DETECTION
// ============================================================================

/// Detect trends from correlated issues
fn detect_trends(
    correlated_issues: &[CorrelatedIssue],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<TrendObservation> {
    let mut trends = Vec::new();

    // Compare with previous assessment for escalation/improvement detection
    if let Some(prev) = input.previous_assessment {
        for current_issue in correlated_issues {
            // Find matching issue in previous assessment
            if let Some(prev_issue) = prev.correlated_issues.iter().find(|i| {
                std::mem::discriminant(&i.root_cause) == std::mem::discriminant(&current_issue.root_cause)
            }) {
                // Check for escalation
                if current_issue.severity > prev_issue.severity {
                    let duration_minutes = (now - prev_issue.first_seen).num_minutes() as u32;

                    trends.push(TrendObservation {
                        subject: current_issue.summary.clone(),
                        trend_type: TrendType::Escalating,
                        duration_minutes,
                        projected_severity: IssueSeverity::Critical,
                        recommendation: "Issue severity has increased. Take immediate action before complete failure.".to_string(),
                        first_detected: prev_issue.first_seen,
                    });
                }

                // Check for flapping (severity changing)
                let duration_minutes = (now - prev_issue.first_seen).num_minutes() as u32;
                if duration_minutes < WINDOW_SHORT_MINUTES as u32 && current_issue.severity != prev_issue.severity {
                    trends.push(TrendObservation {
                        subject: current_issue.summary.clone(),
                        trend_type: TrendType::Flapping,
                        duration_minutes,
                        projected_severity: current_issue.severity,
                        recommendation: "Issue severity is oscillating. Investigate intermittent cause.".to_string(),
                        first_detected: prev_issue.first_seen,
                    });
                }
            }
        }

        // Check for improvements (issues that decreased in severity or disappeared)
        for prev_issue in &prev.correlated_issues {
            if let Some(current_issue) = correlated_issues.iter().find(|i| {
                std::mem::discriminant(&i.root_cause) == std::mem::discriminant(&prev_issue.root_cause)
            }) {
                if current_issue.severity < prev_issue.severity {
                    let duration_minutes = (now - prev_issue.first_seen).num_minutes() as u32;

                    trends.push(TrendObservation {
                        subject: current_issue.summary.clone(),
                        trend_type: TrendType::Improving,
                        duration_minutes,
                        projected_severity: IssueSeverity::Info,
                        recommendation: "Issue is improving. Continue monitoring.".to_string(),
                        first_detected: prev_issue.first_seen,
                    });
                }
            }
        }
    }

    // Limit trends to reasonable count
    trends.truncate(10);

    trends
}

// ============================================================================
// RECOVERY DETECTION
// ============================================================================

/// Detect recoveries from previous assessment
fn detect_recoveries(
    correlated_issues: &[CorrelatedIssue],
    input: &ProactiveInput,
    now: DateTime<Utc>,
) -> Vec<RecoveryNotice> {
    let mut recoveries = Vec::new();

    if let Some(prev) = input.previous_assessment {
        // Find issues that were in previous assessment but not in current
        for prev_issue in &prev.correlated_issues {
            let still_present = correlated_issues.iter().any(|i| {
                std::mem::discriminant(&i.root_cause) == std::mem::discriminant(&prev_issue.root_cause)
            });

            if !still_present {
                let duration_hours = (prev_issue.last_seen - prev_issue.first_seen).num_hours().max(0) as u32;

                // Check if within 24h TTL
                let recovery_age_hours = (now - prev_issue.last_seen).num_hours();
                if recovery_age_hours <= RECOVERY_TTL_HOURS {
                    recoveries.push(RecoveryNotice {
                        subject: prev_issue.summary.clone(),
                        recovery_time: now,
                        duration_hours,
                        resolution: None, // Would be populated from historian in full implementation
                        original_severity: prev_issue.severity,
                    });
                }
            }
        }

        // Also include recoveries from previous assessment that are still within TTL
        for prev_recovery in &prev.recoveries {
            let recovery_age_hours = (now - prev_recovery.recovery_time).num_hours();
            if recovery_age_hours <= RECOVERY_TTL_HOURS {
                recoveries.push(prev_recovery.clone());
            }
        }
    }

    // Limit recoveries to reasonable count
    recoveries.truncate(10);

    recoveries
}

// ============================================================================
// HEALTH SCORE CALCULATION
// ============================================================================

/// Calculate overall health score (0-100)
fn calculate_health_score(
    critical_count: usize,
    warning_count: usize,
    trends: &[TrendObservation],
) -> u8 {
    let mut score: i32 = 100;

    // Subtract for critical issues
    score -= (critical_count as i32) * (WEIGHT_CRITICAL as i32);

    // Subtract for warning issues
    score -= (warning_count as i32) * (WEIGHT_WARNING as i32);

    // Subtract for escalating trends
    let escalating_count = trends.iter()
        .filter(|t| t.trend_type == TrendType::Escalating)
        .count();
    score -= (escalating_count as i32) * (WEIGHT_TREND as i32);

    // Subtract for flapping issues
    let flapping_count = trends.iter()
        .filter(|t| t.trend_type == TrendType::Flapping)
        .count();
    score -= (flapping_count as i32) * (WEIGHT_FLAPPING as i32);

    // Clamp to [0, 100]
    score.clamp(0, 100) as u8
}

// ============================================================================
// CONVERSION TO USER-SAFE TYPES (Beta.271)
// ============================================================================

/// Convert RootCause to user-safe string label
pub fn root_cause_to_string(root_cause: &RootCause) -> String {
    match root_cause {
        RootCause::NetworkRoutingConflict { .. } => "network_routing_conflict".to_string(),
        RootCause::NetworkPriorityMismatch { .. } => "network_priority_mismatch".to_string(),
        RootCause::NetworkQualityDegradation { .. } => "network_quality_degradation".to_string(),
        RootCause::DiskPressure { .. } => "disk_pressure".to_string(),
        RootCause::DiskLogGrowth { .. } => "disk_log_growth".to_string(),
        RootCause::ServiceFlapping { .. } => "service_flapping".to_string(),
        RootCause::ServiceUnderLoad { .. } => "service_under_load".to_string(),
        RootCause::ServiceConfigError { .. } => "service_config_error".to_string(),
        RootCause::MemoryPressure { .. } => "memory_pressure".to_string(),
        RootCause::CpuOverload { .. } => "cpu_overload".to_string(),
        RootCause::KernelRegression { .. } => "kernel_regression".to_string(),
        RootCause::DeviceHotplug { .. } => "device_hotplug".to_string(),
    }
}

/// Convert IssueSeverity to user-safe string
pub fn severity_to_string(severity: IssueSeverity) -> String {
    match severity {
        IssueSeverity::Critical => "critical".to_string(),
        IssueSeverity::Warning => "warning".to_string(),
        IssueSeverity::Info => "info".to_string(),
        IssueSeverity::Trend => "trend".to_string(),
    }
}

/// Convert ProactiveAssessment to user-safe issue summaries
///
/// Only includes issues with confidence >= 0.7
/// Caps at MAX_DISPLAYED_ISSUES (10)
/// Sorts by severity (critical first)
pub fn assessment_to_summaries(assessment: &ProactiveAssessment) -> Vec<crate::steward::ProactiveIssueSummary> {
    assessment.correlated_issues
        .iter()
        .take(MAX_DISPLAYED_ISSUES)
        .map(|issue| crate::steward::ProactiveIssueSummary {
            root_cause: root_cause_to_string(&issue.root_cause),
            severity: severity_to_string(issue.severity),
            summary: issue.summary.clone(),
            rule_id: None, // Will be populated when we map to remediation rules
            confidence: issue.confidence,
            first_seen: issue.first_seen.to_rfc3339(),
            last_seen: issue.last_seen.to_rfc3339(),
        })
        .collect()
}

// ============================================================================
// Beta.279: HISTORIAN QUERY HELPERS
// ============================================================================

use crate::historian::{Historian, HistoryEvent};
use chrono::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Global historian instance for proactive engine
static PROACTIVE_HISTORIAN: once_cell::sync::Lazy<Arc<Mutex<Option<Historian>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Load recent history within a time window
///
/// Returns events in chronological order (oldest first).
/// Returns empty vec if historian is not available.
fn load_recent_history(window: Duration) -> Vec<HistoryEvent> {
    // Try non-blocking lock
    if let Ok(mut hist_lock) = PROACTIVE_HISTORIAN.try_lock() {
        if hist_lock.is_none() {
            // Initialize historian
            let state_dir = std::path::PathBuf::from("/var/lib/anna/state");
            match Historian::new(&state_dir) {
                Ok(historian) => {
                    *hist_lock = Some(historian);
                }
                Err(e) => {
                    tracing::warn!("Proactive engine: Failed to initialize historian: {}. Temporal correlation disabled.", e);
                    return Vec::new();
                }
            }
        }

        // Load history while holding lock
        if let Some(ref historian) = *hist_lock {
            historian.load_recent(window).unwrap_or_else(|e| {
                tracing::warn!("Failed to load historian data: {}. Correlation may be incomplete.", e);
                Vec::new()
            })
        } else {
            Vec::new()
        }
    } else {
        // Lock contention - skip historian for this cycle
        Vec::new()
    }
}

/// Detect service flapping (SVC-001)
///
/// Detects when failed_services_count oscillates up/down multiple times
/// within a short window (indicating restart loops or flapping).
fn detect_service_flapping(events: &[HistoryEvent]) -> Option<TrendObservation> {
    if events.len() < 4 {
        return None; // Need at least 4 events to detect pattern
    }

    // Count transitions from 0 -> non-zero and non-zero -> 0
    let mut transitions = 0;
    let mut last_had_failures = events[0].failed_services_count > 0;

    for event in events.iter().skip(1) {
        let current_has_failures = event.failed_services_count > 0;
        if current_has_failures != last_had_failures {
            transitions += 1;
        }
        last_had_failures = current_has_failures;
    }

    // Require at least 3 transitions (indicating flapping pattern)
    if transitions >= 3 {
        let duration_minutes = events.len() as u32 * 5; // Rough estimate
        Some(TrendObservation {
            subject: "Service stability".to_string(),
            trend_type: TrendType::Flapping,
            duration_minutes,
            projected_severity: IssueSeverity::Warning,
            recommendation: "Investigate service restart loops with 'journalctl -u <service>'".to_string(),
            first_detected: events.first().unwrap().timestamp_utc,
        })
    } else {
        None
    }
}

/// Detect disk growth (DISK-002)
///
/// Detects when root disk usage shows clear upward trend
/// from safe levels into high-risk territory.
fn detect_disk_growth(events: &[HistoryEvent]) -> Option<TrendObservation> {
    if events.len() < 3 {
        return None; // Need at least 3 points for trend
    }

    // Check for monotonic or mostly-increasing growth
    let first_usage = events[0].disk_root_usage_pct;
    let last_usage = events.last().unwrap().disk_root_usage_pct;

    // Require significant growth (at least 15 percentage points)
    let growth = last_usage.saturating_sub(first_usage);
    if growth < 15 {
        return None;
    }

    // Require trend into danger zone (>80%)
    if last_usage < 80 {
        return None;
    }

    // Check that it's mostly increasing (allow small dips)
    let mut increasing_count = 0;
    for i in 1..events.len() {
        if events[i].disk_root_usage_pct >= events[i - 1].disk_root_usage_pct {
            increasing_count += 1;
        }
    }

    // Require at least 70% of transitions to be increasing
    if increasing_count * 10 >= (events.len() - 1) * 7 {
        let duration_minutes = events.len() as u32 * 5;
        Some(TrendObservation {
            subject: "Root disk usage".to_string(),
            trend_type: TrendType::Degrading,
            duration_minutes,
            projected_severity: IssueSeverity::Warning,
            recommendation: "Clean up disk space with 'ncdu /' or 'pacman -Sc'".to_string(),
            first_detected: events.first().unwrap().timestamp_utc,
        })
    } else {
        None
    }
}

/// Detect resource pressure (RES-001, RES-002)
///
/// Detects sustained high CPU or memory flags across multiple events.
/// Single spikes are filtered out.
fn detect_resource_pressure(events: &[HistoryEvent]) -> Option<TrendObservation> {
    if events.len() < 3 {
        return None; // Need sustained pressure, not single spike
    }

    // Count events with high CPU or memory flags
    let high_cpu_count = events.iter().filter(|e| e.high_cpu_flag).count();
    let high_memory_count = events.iter().filter(|e| e.high_memory_flag).count();

    // Require sustained pressure (>60% of events)
    let threshold = (events.len() * 6) / 10;

    if high_cpu_count >= threshold {
        let duration_minutes = events.len() as u32 * 5;
        Some(TrendObservation {
            subject: "CPU load".to_string(),
            trend_type: TrendType::Degrading,
            duration_minutes,
            projected_severity: IssueSeverity::Warning,
            recommendation: "Check top CPU consumers with 'htop' or 'ps aux --sort=-pcpu'".to_string(),
            first_detected: events.first().unwrap().timestamp_utc,
        })
    } else if high_memory_count >= threshold {
        let duration_minutes = events.len() as u32 * 5;
        Some(TrendObservation {
            subject: "Memory usage".to_string(),
            trend_type: TrendType::Degrading,
            duration_minutes,
            projected_severity: IssueSeverity::Warning,
            recommendation: "Check memory hogs with 'free -h' and 'ps aux --sort=-pmem'".to_string(),
            first_detected: events.first().unwrap().timestamp_utc,
        })
    } else {
        None
    }
}

/// Detect kernel regression (SYS-001)
///
/// Detects when health degrades after a kernel change.
/// Compares failed/degraded counts before and after kernel_changed flag.
fn detect_kernel_regression(events: &[HistoryEvent]) -> Option<TrendObservation> {
    if events.len() < 4 {
        return None;
    }

    // Find kernel change event
    let kernel_change_idx = events.iter().position(|e| e.kernel_changed)?;

    // Need events both before and after
    if kernel_change_idx == 0 || kernel_change_idx >= events.len() - 1 {
        return None;
    }

    // Calculate average health before and after
    let before_events = &events[..kernel_change_idx];
    let after_events = &events[kernel_change_idx + 1..];

    let before_avg_failures: f64 = before_events
        .iter()
        .map(|e| (e.failed_services_count + e.degraded_services_count) as f64)
        .sum::<f64>()
        / before_events.len() as f64;

    let after_avg_failures: f64 = after_events
        .iter()
        .map(|e| (e.failed_services_count + e.degraded_services_count) as f64)
        .sum::<f64>()
        / after_events.len() as f64;

    // Require significant increase in failures after kernel change
    if after_avg_failures > before_avg_failures + 1.0 {
        let duration_minutes = after_events.len() as u32 * 5;
        Some(TrendObservation {
            subject: "Post-kernel-upgrade health".to_string(),
            trend_type: TrendType::Degrading,
            duration_minutes,
            projected_severity: IssueSeverity::Warning,
            recommendation: "Check kernel logs with 'journalctl -k' and consider rollback if issues persist".to_string(),
            first_detected: events[kernel_change_idx].timestamp_utc,
        })
    } else {
        None
    }
}

/// Detect network trend (NET-003)
///
/// Detects rising packet loss or latency that exceeds normal noise.
fn detect_network_trend(events: &[HistoryEvent]) -> Option<TrendObservation> {
    if events.len() < 3 {
        return None;
    }

    // Check for rising packet loss
    let first_loss = events[0].network_packet_loss_pct;
    let last_loss = events.last().unwrap().network_packet_loss_pct;

    if last_loss > 5 && last_loss > first_loss + 3 {
        let duration_minutes = events.len() as u32 * 5;
        return Some(TrendObservation {
            subject: "Network packet loss".to_string(),
            trend_type: TrendType::Degrading,
            duration_minutes,
            projected_severity: IssueSeverity::Warning,
            recommendation: "Check network connectivity with 'ping' and 'ip addr'".to_string(),
            first_detected: events.first().unwrap().timestamp_utc,
        });
    }

    // Check for rising latency
    let first_latency = events[0].network_latency_ms;
    let last_latency = events.last().unwrap().network_latency_ms;

    if last_latency > 100 && last_latency > first_latency + 50 {
        let duration_minutes = events.len() as u32 * 5;
        return Some(TrendObservation {
            subject: "Network latency".to_string(),
            trend_type: TrendType::Degrading,
            duration_minutes,
            projected_severity: IssueSeverity::Info,
            recommendation: "Monitor network performance with 'mtr' or 'traceroute'".to_string(),
            first_detected: events.first().unwrap().timestamp_utc,
        });
    }

    None
}

// ============================================================================
// HELPER FUNCTIONS FOR TESTING
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_score_perfect() {
        let score = calculate_health_score(0, 0, &[]);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_health_score_one_critical() {
        let score = calculate_health_score(1, 0, &[]);
        assert_eq!(score, 80); // 100 - 20
    }

    #[test]
    fn test_health_score_multiple_issues() {
        let score = calculate_health_score(2, 3, &[]);
        assert_eq!(score, 30); // 100 - (2*20) - (3*10) = 30
    }

    #[test]
    fn test_health_score_clamped_at_zero() {
        let score = calculate_health_score(10, 10, &[]);
        assert_eq!(score, 0); // Would be negative, clamped to 0
    }

    #[test]
    fn test_min_confidence_threshold() {
        assert_eq!(MIN_CONFIDENCE, 0.7);
    }

    #[test]
    fn test_max_displayed_issues() {
        assert_eq!(MAX_DISPLAYED_ISSUES, 10);
    }
}
