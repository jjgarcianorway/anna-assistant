//! Anna Reliability Engineering v0.0.22
//!
//! This module provides:
//! - Metrics collection for requests, tools, mutations, LLM calls
//! - Error budget definitions and burn rate calculations
//! - Budget alerts when thresholds exceeded
//! - Self-diagnostics report generation
//!
//! Storage:
//! - Metrics: /var/lib/anna/internal/metrics.json
//! - Ops log: /var/lib/anna/internal/ops_log.jsonl

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// =============================================================================
// Constants and Paths
// =============================================================================

/// Metrics storage file
pub const METRICS_FILE: &str = "/var/lib/anna/internal/metrics.json";

/// Operations log file (append-only JSONL)
pub const OPS_LOG_FILE: &str = "/var/lib/anna/internal/ops_log.jsonl";

/// Default error budget thresholds (percentage per day)
pub const DEFAULT_REQUEST_FAILURE_BUDGET: f64 = 1.0;    // max 1% request failures
pub const DEFAULT_TOOL_FAILURE_BUDGET: f64 = 2.0;       // max 2% tool failures
pub const DEFAULT_MUTATION_ROLLBACK_BUDGET: f64 = 0.5;  // max 0.5% mutation rollbacks
pub const DEFAULT_LLM_TIMEOUT_BUDGET: f64 = 3.0;        // max 3% LLM timeouts

/// Alert threshold multiplier (warn at 50% budget consumed, critical at 80%)
pub const BUDGET_WARN_THRESHOLD: f64 = 0.5;
pub const BUDGET_CRITICAL_THRESHOLD: f64 = 0.8;

// =============================================================================
// Metrics Types
// =============================================================================

/// A single metric event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Request started
    RequestStart,
    /// Request completed successfully
    RequestSuccess,
    /// Request failed
    RequestFailure,
    /// Tool call started
    ToolStart,
    /// Tool call succeeded
    ToolSuccess,
    /// Tool call failed
    ToolFailure,
    /// Mutation started
    MutationStart,
    /// Mutation succeeded
    MutationSuccess,
    /// Mutation auto-rolled back
    MutationRollback,
    /// Translator LLM call started
    TranslatorStart,
    /// Translator LLM call succeeded
    TranslatorSuccess,
    /// Translator LLM call timed out
    TranslatorTimeout,
    /// Junior LLM call started
    JuniorStart,
    /// Junior LLM call succeeded
    JuniorSuccess,
    /// Junior LLM call timed out
    JuniorTimeout,
    /// Cache hit
    CacheHit,
    /// Cache miss
    CacheMiss,
}

impl MetricType {
    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::RequestStart => "request_start",
            MetricType::RequestSuccess => "request_success",
            MetricType::RequestFailure => "request_failure",
            MetricType::ToolStart => "tool_start",
            MetricType::ToolSuccess => "tool_success",
            MetricType::ToolFailure => "tool_failure",
            MetricType::MutationStart => "mutation_start",
            MetricType::MutationSuccess => "mutation_success",
            MetricType::MutationRollback => "mutation_rollback",
            MetricType::TranslatorStart => "translator_start",
            MetricType::TranslatorSuccess => "translator_success",
            MetricType::TranslatorTimeout => "translator_timeout",
            MetricType::JuniorStart => "junior_start",
            MetricType::JuniorSuccess => "junior_success",
            MetricType::JuniorTimeout => "junior_timeout",
            MetricType::CacheHit => "cache_hit",
            MetricType::CacheMiss => "cache_miss",
        }
    }
}

/// Latency sample for percentile calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyRecord {
    /// Timestamp (epoch seconds)
    pub timestamp: u64,
    /// Category (e2e, translator, junior, tool)
    pub category: String,
    /// Latency in milliseconds
    pub latency_ms: u64,
}

/// Metrics storage for a single day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetrics {
    /// Date string (YYYY-MM-DD)
    pub date: String,
    /// Metric counts by type
    pub counts: HashMap<String, u64>,
    /// Latency samples for percentile calculations
    pub latencies: Vec<LatencyRecord>,
}

impl Default for DailyMetrics {
    fn default() -> Self {
        Self {
            date: today_string(),
            counts: HashMap::new(),
            latencies: Vec::new(),
        }
    }
}

impl DailyMetrics {
    /// Get count for a metric type
    pub fn get_count(&self, metric: MetricType) -> u64 {
        *self.counts.get(metric.as_str()).unwrap_or(&0)
    }

    /// Increment a metric counter
    pub fn increment(&mut self, metric: MetricType) {
        *self.counts.entry(metric.as_str().to_string()).or_insert(0) += 1;
    }

    /// Add a latency sample
    pub fn add_latency(&mut self, category: &str, latency_ms: u64) {
        self.latencies.push(LatencyRecord {
            timestamp: now_epoch(),
            category: category.to_string(),
            latency_ms,
        });

        // Keep max 10000 samples per day
        if self.latencies.len() > 10000 {
            self.latencies.drain(0..1000);
        }
    }

    /// Calculate percentile latency for a category
    pub fn percentile_latency(&self, category: &str, percentile: f64) -> Option<u64> {
        let mut samples: Vec<u64> = self.latencies
            .iter()
            .filter(|l| l.category == category)
            .map(|l| l.latency_ms)
            .collect();

        if samples.is_empty() {
            return None;
        }

        samples.sort_unstable();
        let idx = ((samples.len() as f64 * percentile / 100.0).ceil() as usize).saturating_sub(1);
        Some(samples[idx.min(samples.len() - 1)])
    }

    /// Calculate average latency for a category
    pub fn avg_latency(&self, category: &str) -> Option<u64> {
        let samples: Vec<u64> = self.latencies
            .iter()
            .filter(|l| l.category == category)
            .map(|l| l.latency_ms)
            .collect();

        if samples.is_empty() {
            return None;
        }

        Some(samples.iter().sum::<u64>() / samples.len() as u64)
    }
}

/// Full metrics store (rolling window)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStore {
    /// Schema version
    pub schema_version: u32,
    /// Daily metrics by date
    pub daily: HashMap<String, DailyMetrics>,
    /// Retention days
    pub retention_days: u32,
}

impl Default for MetricsStore {
    fn default() -> Self {
        Self {
            schema_version: 1,
            daily: HashMap::new(),
            retention_days: 7,
        }
    }
}

impl MetricsStore {
    /// Load metrics from file
    pub fn load() -> Self {
        let path = Path::new(METRICS_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(store) = serde_json::from_str::<MetricsStore>(&content) {
                    return store;
                }
            }
        }
        Self::default()
    }

    /// Save metrics to file
    pub fn save(&self) -> std::io::Result<()> {
        let parent = Path::new(METRICS_FILE).parent();
        if let Some(p) = parent {
            fs::create_dir_all(p)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(METRICS_FILE, &content)
    }

    /// Get or create today's metrics
    pub fn today(&mut self) -> &mut DailyMetrics {
        let today = today_string();
        self.daily.entry(today.clone()).or_insert_with(|| DailyMetrics {
            date: today,
            counts: HashMap::new(),
            latencies: Vec::new(),
        })
    }

    /// Get metrics for a specific date
    pub fn for_date(&self, date: &str) -> Option<&DailyMetrics> {
        self.daily.get(date)
    }

    /// Record a metric
    pub fn record(&mut self, metric: MetricType) {
        self.today().increment(metric);
    }

    /// Record a latency sample
    pub fn record_latency(&mut self, category: &str, latency_ms: u64) {
        self.today().add_latency(category, latency_ms);
    }

    /// Prune old data beyond retention
    pub fn prune(&mut self) {
        let cutoff = days_ago_string(self.retention_days as i64);
        self.daily.retain(|date, _| date >= &cutoff);
    }

    /// Get total counts for last N days
    pub fn total_counts(&self, days: u32) -> HashMap<String, u64> {
        let mut totals: HashMap<String, u64> = HashMap::new();
        let cutoff = days_ago_string(days as i64);

        for (date, metrics) in &self.daily {
            if date >= &cutoff {
                for (key, count) in &metrics.counts {
                    *totals.entry(key.clone()).or_insert(0) += count;
                }
            }
        }

        totals
    }
}

// =============================================================================
// Error Budgets
// =============================================================================

/// Error budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBudgets {
    /// Max request failure rate (percentage per day)
    #[serde(default = "default_request_budget")]
    pub request_failure_percent: f64,

    /// Max tool failure rate (percentage per day)
    #[serde(default = "default_tool_budget")]
    pub tool_failure_percent: f64,

    /// Max mutation rollback rate (percentage per day)
    #[serde(default = "default_mutation_budget")]
    pub mutation_rollback_percent: f64,

    /// Max LLM timeout rate (percentage per day)
    #[serde(default = "default_llm_budget")]
    pub llm_timeout_percent: f64,
}

fn default_request_budget() -> f64 { DEFAULT_REQUEST_FAILURE_BUDGET }
fn default_tool_budget() -> f64 { DEFAULT_TOOL_FAILURE_BUDGET }
fn default_mutation_budget() -> f64 { DEFAULT_MUTATION_ROLLBACK_BUDGET }
fn default_llm_budget() -> f64 { DEFAULT_LLM_TIMEOUT_BUDGET }

impl Default for ErrorBudgets {
    fn default() -> Self {
        Self {
            request_failure_percent: DEFAULT_REQUEST_FAILURE_BUDGET,
            tool_failure_percent: DEFAULT_TOOL_FAILURE_BUDGET,
            mutation_rollback_percent: DEFAULT_MUTATION_ROLLBACK_BUDGET,
            llm_timeout_percent: DEFAULT_LLM_TIMEOUT_BUDGET,
        }
    }
}

/// Budget status for a single category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// Category name
    pub category: String,
    /// Budget limit (percentage)
    pub budget_percent: f64,
    /// Current burn (percentage)
    pub current_percent: f64,
    /// Total events
    pub total_events: u64,
    /// Failed events
    pub failed_events: u64,
    /// Status: ok, warning, critical, exhausted
    pub status: BudgetState,
}

/// Budget state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetState {
    /// Budget healthy (< 50% consumed)
    Ok,
    /// Budget warning (50-80% consumed)
    Warning,
    /// Budget critical (80-100% consumed)
    Critical,
    /// Budget exhausted (> 100%)
    Exhausted,
}

impl BudgetState {
    pub fn as_str(&self) -> &'static str {
        match self {
            BudgetState::Ok => "ok",
            BudgetState::Warning => "warning",
            BudgetState::Critical => "critical",
            BudgetState::Exhausted => "exhausted",
        }
    }
}

/// Calculate budget status for all categories
pub fn calculate_budget_status(metrics: &DailyMetrics, budgets: &ErrorBudgets) -> Vec<BudgetStatus> {
    let mut statuses = Vec::new();

    // Request failures
    let request_total = metrics.get_count(MetricType::RequestSuccess) + metrics.get_count(MetricType::RequestFailure);
    let request_failed = metrics.get_count(MetricType::RequestFailure);
    if request_total > 0 {
        let rate = (request_failed as f64 / request_total as f64) * 100.0;
        statuses.push(BudgetStatus {
            category: "request_failures".to_string(),
            budget_percent: budgets.request_failure_percent,
            current_percent: rate,
            total_events: request_total,
            failed_events: request_failed,
            status: compute_budget_state(rate, budgets.request_failure_percent),
        });
    }

    // Tool failures
    let tool_total = metrics.get_count(MetricType::ToolSuccess) + metrics.get_count(MetricType::ToolFailure);
    let tool_failed = metrics.get_count(MetricType::ToolFailure);
    if tool_total > 0 {
        let rate = (tool_failed as f64 / tool_total as f64) * 100.0;
        statuses.push(BudgetStatus {
            category: "tool_failures".to_string(),
            budget_percent: budgets.tool_failure_percent,
            current_percent: rate,
            total_events: tool_total,
            failed_events: tool_failed,
            status: compute_budget_state(rate, budgets.tool_failure_percent),
        });
    }

    // Mutation rollbacks
    let mutation_total = metrics.get_count(MetricType::MutationSuccess) + metrics.get_count(MetricType::MutationRollback);
    let mutation_rollback = metrics.get_count(MetricType::MutationRollback);
    if mutation_total > 0 {
        let rate = (mutation_rollback as f64 / mutation_total as f64) * 100.0;
        statuses.push(BudgetStatus {
            category: "mutation_rollbacks".to_string(),
            budget_percent: budgets.mutation_rollback_percent,
            current_percent: rate,
            total_events: mutation_total,
            failed_events: mutation_rollback,
            status: compute_budget_state(rate, budgets.mutation_rollback_percent),
        });
    }

    // LLM timeouts (translator + junior)
    let translator_total = metrics.get_count(MetricType::TranslatorSuccess) + metrics.get_count(MetricType::TranslatorTimeout);
    let junior_total = metrics.get_count(MetricType::JuniorSuccess) + metrics.get_count(MetricType::JuniorTimeout);
    let llm_total = translator_total + junior_total;
    let llm_timeouts = metrics.get_count(MetricType::TranslatorTimeout) + metrics.get_count(MetricType::JuniorTimeout);
    if llm_total > 0 {
        let rate = (llm_timeouts as f64 / llm_total as f64) * 100.0;
        statuses.push(BudgetStatus {
            category: "llm_timeouts".to_string(),
            budget_percent: budgets.llm_timeout_percent,
            current_percent: rate,
            total_events: llm_total,
            failed_events: llm_timeouts,
            status: compute_budget_state(rate, budgets.llm_timeout_percent),
        });
    }

    statuses
}

fn compute_budget_state(current: f64, budget: f64) -> BudgetState {
    if budget <= 0.0 {
        return BudgetState::Exhausted;
    }

    let ratio = current / budget;
    if ratio >= 1.0 {
        BudgetState::Exhausted
    } else if ratio >= BUDGET_CRITICAL_THRESHOLD {
        BudgetState::Critical
    } else if ratio >= BUDGET_WARN_THRESHOLD {
        BudgetState::Warning
    } else {
        BudgetState::Ok
    }
}

/// Check if any budgets need alerts
pub fn check_budget_alerts(statuses: &[BudgetStatus]) -> Vec<BudgetAlert> {
    let mut alerts = Vec::new();

    for status in statuses {
        match status.status {
            BudgetState::Warning => {
                alerts.push(BudgetAlert {
                    category: status.category.clone(),
                    severity: AlertSeverity::Warning,
                    message: format!(
                        "Error budget warning: {} at {:.1}% (budget: {:.1}%)",
                        status.category, status.current_percent, status.budget_percent
                    ),
                    current_percent: status.current_percent,
                    budget_percent: status.budget_percent,
                });
            }
            BudgetState::Critical | BudgetState::Exhausted => {
                alerts.push(BudgetAlert {
                    category: status.category.clone(),
                    severity: AlertSeverity::Critical,
                    message: format!(
                        "Error budget critical: {} at {:.1}% (budget: {:.1}%)",
                        status.category, status.current_percent, status.budget_percent
                    ),
                    current_percent: status.current_percent,
                    budget_percent: status.budget_percent,
                });
            }
            BudgetState::Ok => {}
        }
    }

    alerts
}

/// Budget alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub category: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub current_percent: f64,
    pub budget_percent: f64,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

// =============================================================================
// Operations Log
// =============================================================================

/// A single operation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsLogEntry {
    /// Timestamp (epoch seconds)
    pub timestamp: u64,
    /// Operation type
    pub op_type: String,
    /// Request ID (if applicable)
    pub request_id: Option<String>,
    /// Tool name (if applicable)
    pub tool_name: Option<String>,
    /// Success/failure
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration in ms
    pub duration_ms: Option<u64>,
    /// Evidence ID
    pub evidence_id: Option<String>,
}

impl OpsLogEntry {
    /// Create a new ops log entry
    pub fn new(op_type: &str) -> Self {
        Self {
            timestamp: now_epoch(),
            op_type: op_type.to_string(),
            request_id: None,
            tool_name: None,
            success: true,
            error: None,
            duration_ms: None,
            evidence_id: None,
        }
    }

    /// Set as failure with error
    pub fn with_error(mut self, error: &str) -> Self {
        self.success = false;
        self.error = Some(error.to_string());
        self
    }

    /// Append to ops log file
    pub fn append(&self) -> std::io::Result<()> {
        use std::io::Write;

        let parent = Path::new(OPS_LOG_FILE).parent();
        if let Some(p) = parent {
            fs::create_dir_all(p)?;
        }

        let line = serde_json::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(OPS_LOG_FILE)?;

        writeln!(file, "{}", line)
    }
}

/// Load recent ops log entries
pub fn load_recent_ops_log(count: usize) -> Vec<OpsLogEntry> {
    let path = Path::new(OPS_LOG_FILE);
    if !path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let entries: Vec<OpsLogEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Return last N entries
    if entries.len() <= count {
        entries
    } else {
        entries[entries.len() - count..].to_vec()
    }
}

/// Load recent errors from ops log
pub fn load_recent_errors(count: usize) -> Vec<OpsLogEntry> {
    let all = load_recent_ops_log(1000);
    all.into_iter()
        .filter(|e| !e.success)
        .rev()
        .take(count)
        .collect()
}

// =============================================================================
// Self-Diagnostics Report
// =============================================================================

/// Self-diagnostics report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSection {
    pub title: String,
    pub evidence_id: String,
    pub content: Vec<String>,
    pub status: DiagStatus,
}

/// Diagnostic status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagStatus {
    Ok,
    Warning,
    Error,
}

impl DiagStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagStatus::Ok => "OK",
            DiagStatus::Warning => "WARNING",
            DiagStatus::Error => "ERROR",
        }
    }
}

/// Full self-diagnostics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    /// Report timestamp
    pub generated_at: String,
    /// Anna version
    pub version: String,
    /// Sections
    pub sections: Vec<DiagnosticsSection>,
    /// Overall status
    pub overall_status: DiagStatus,
    /// Evidence ID for the report
    pub report_evidence_id: String,
}

impl DiagnosticsReport {
    /// Generate a new diagnostics report
    pub fn generate() -> Self {
        let mut sections = Vec::new();
        let mut worst_status = DiagStatus::Ok;
        let mut evidence_counter = 1;

        // Version section
        let version = env!("CARGO_PKG_VERSION").to_string();
        sections.push(DiagnosticsSection {
            title: "Version".to_string(),
            evidence_id: format!("DIAG{:05}", evidence_counter),
            content: vec![format!("Anna v{}", version)],
            status: DiagStatus::Ok,
        });
        evidence_counter += 1;

        // Install review section
        let install_section = gather_install_review_section(&mut evidence_counter);
        if install_section.status as u8 > worst_status as u8 {
            worst_status = install_section.status;
        }
        sections.push(install_section);

        // Update state section
        let update_section = gather_update_state_section(&mut evidence_counter);
        if update_section.status as u8 > worst_status as u8 {
            worst_status = update_section.status;
        }
        sections.push(update_section);

        // Model readiness section
        let model_section = gather_model_readiness_section(&mut evidence_counter);
        if model_section.status as u8 > worst_status as u8 {
            worst_status = model_section.status;
        }
        sections.push(model_section);

        // Policy section
        let policy_section = gather_policy_section(&mut evidence_counter);
        if policy_section.status as u8 > worst_status as u8 {
            worst_status = policy_section.status;
        }
        sections.push(policy_section);

        // Storage section
        let storage_section = gather_storage_section(&mut evidence_counter);
        sections.push(storage_section);

        // Error budget section
        let budget_section = gather_budget_section(&mut evidence_counter);
        if budget_section.status as u8 > worst_status as u8 {
            worst_status = budget_section.status;
        }
        sections.push(budget_section);

        // Recent errors section (redacted)
        let errors_section = gather_errors_section(&mut evidence_counter);
        if errors_section.status as u8 > worst_status as u8 {
            worst_status = errors_section.status;
        }
        sections.push(errors_section);

        // Alerts section
        let alerts_section = gather_alerts_section(&mut evidence_counter);
        if alerts_section.status as u8 > worst_status as u8 {
            worst_status = alerts_section.status;
        }
        sections.push(alerts_section);

        Self {
            generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S %Z").to_string(),
            version,
            sections,
            overall_status: worst_status,
            report_evidence_id: format!("DIAG{:05}", evidence_counter),
        }
    }

    /// Format as human-readable text
    pub fn to_text(&self) -> String {
        let mut lines = Vec::new();

        lines.push("=== Anna Self-Diagnostics Report ===".to_string());
        lines.push(format!("Generated: {} [{}]", self.generated_at, self.report_evidence_id));
        lines.push(format!("Version: {}", self.version));
        lines.push(format!("Overall Status: {}", self.overall_status.as_str()));
        lines.push(String::new());

        for section in &self.sections {
            lines.push(format!("--- {} [{}] ({}) ---", section.title, section.evidence_id, section.status.as_str()));
            for line in &section.content {
                lines.push(format!("  {}", line));
            }
            lines.push(String::new());
        }

        lines.push("=== End Report ===".to_string());

        lines.join("\n")
    }
}

fn gather_install_review_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let install_state = crate::install_state::InstallState::load();

    match install_state {
        Some(state) => {
            match &state.last_review {
                Some(review) => {
                    let status = match &review.result {
                        crate::install_state::ReviewResult::Healthy => DiagStatus::Ok,
                        crate::install_state::ReviewResult::Repaired { .. } => DiagStatus::Ok,
                        crate::install_state::ReviewResult::NeedsAttention { .. } => DiagStatus::Warning,
                        crate::install_state::ReviewResult::Failed { .. } => DiagStatus::Error,
                    };

                    let content = vec![
                        format!("Last review: {}", review.timestamp.format("%Y-%m-%d %H:%M")),
                        format!("Result: {:?}", review.result),
                        format!("Duration: {}ms", review.duration_ms),
                    ];

                    DiagnosticsSection {
                        title: "Install Review".to_string(),
                        evidence_id,
                        content,
                        status,
                    }
                }
                None => DiagnosticsSection {
                    title: "Install Review".to_string(),
                    evidence_id,
                    content: vec!["No review performed yet".to_string()],
                    status: DiagStatus::Warning,
                },
            }
        }
        None => DiagnosticsSection {
            title: "Install Review".to_string(),
            evidence_id,
            content: vec!["No install state found".to_string()],
            status: DiagStatus::Warning,
        },
    }
}

fn gather_update_state_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let state = crate::config::UpdateState::load();

    let status = if state.is_update_in_progress() {
        DiagStatus::Warning
    } else {
        match state.last_result {
            crate::config::UpdateResult::Ok | crate::config::UpdateResult::UpdatedTo => DiagStatus::Ok,
            crate::config::UpdateResult::Failed | crate::config::UpdateResult::RolledBack => DiagStatus::Error,
            _ => DiagStatus::Ok,
        }
    };

    let mut content = vec![
        format!("Mode: {}", state.format_mode()),
        format!("Channel: {}", state.format_channel()),
        format!("Last check: {}", state.format_last_check()),
        format!("Last result: {}", state.format_last_result()),
    ];

    if state.is_update_in_progress() {
        content.push(format!("Update in progress: {:?}", state.update_phase));
    }

    if let Some(ref error) = state.last_error {
        content.push(format!("Last error: {}", error));
    }

    DiagnosticsSection {
        title: "Update State".to_string(),
        evidence_id,
        content,
        status,
    }
}

fn gather_model_readiness_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let junior_state = crate::config::JuniorState::load();
    let config = crate::config::AnnaConfig::load();

    let status = if !config.junior.enabled {
        DiagStatus::Ok // Disabled is OK
    } else if !junior_state.ollama_available {
        DiagStatus::Warning
    } else if !junior_state.model_ready {
        DiagStatus::Warning
    } else {
        DiagStatus::Ok
    };

    let mut content = vec![
        format!("Junior enabled: {}", config.junior.enabled),
        format!("Ollama available: {}", junior_state.ollama_available),
    ];

    if let Some(ref model) = junior_state.selected_model {
        content.push(format!("Model: {} (ready: {})", model, junior_state.model_ready));
    } else {
        content.push("Model: (none selected)".to_string());
    }

    if junior_state.last_check > 0 {
        let ago = now_epoch().saturating_sub(junior_state.last_check);
        content.push(format!("Last check: {}s ago", ago));
    }

    DiagnosticsSection {
        title: "Model Readiness".to_string(),
        evidence_id,
        content,
        status,
    }
}

fn gather_policy_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let policy_dir = Path::new("/etc/anna/policy");

    if !policy_dir.exists() {
        return DiagnosticsSection {
            title: "Policy".to_string(),
            evidence_id,
            content: vec!["Policy directory not found".to_string()],
            status: DiagStatus::Warning,
        };
    }

    let mut content = Vec::new();
    let mut status = DiagStatus::Ok;

    // Check each policy file
    for name in &["capabilities.toml", "blocked.toml", "risk.toml"] {
        let path = policy_dir.join(name);
        if path.exists() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let epoch = modified.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                    content.push(format!("{}: OK (modified: {})", name, format_epoch(epoch)));
                }
            }
        } else {
            content.push(format!("{}: MISSING", name));
            status = DiagStatus::Warning;
        }
    }

    // Try to parse policy
    let policy = crate::policy::get_policy();
    content.push(format!("Schema version: {}", policy.capabilities.schema_version));

    DiagnosticsSection {
        title: "Policy".to_string(),
        evidence_id,
        content,
        status,
    }
}

fn gather_storage_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let data_dir = "/var/lib/anna";

    let content = if Path::new(data_dir).exists() {
        let size = get_dir_size(data_dir);
        vec![
            format!("Data directory: {}", data_dir),
            format!("Total size: {}", format_bytes(size)),
        ]
    } else {
        vec!["Data directory not found".to_string()]
    };

    DiagnosticsSection {
        title: "Storage".to_string(),
        evidence_id,
        content,
        status: DiagStatus::Ok,
    }
}

fn gather_budget_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let metrics = MetricsStore::load();
    let budgets = ErrorBudgets::default();

    let today = today_string();
    let today_metrics = match metrics.for_date(&today) {
        Some(m) => m.clone(),
        None => DailyMetrics::default(),
    };

    let statuses = calculate_budget_status(&today_metrics, &budgets);

    let mut content = Vec::new();
    let mut worst_status = DiagStatus::Ok;

    if statuses.is_empty() {
        content.push("No budget data today".to_string());
    } else {
        for status in &statuses {
            content.push(format!(
                "{}: {:.1}% / {:.1}% ({})",
                status.category, status.current_percent, status.budget_percent, status.status.as_str()
            ));

            match status.status {
                BudgetState::Critical | BudgetState::Exhausted => {
                    worst_status = DiagStatus::Error;
                }
                BudgetState::Warning if worst_status != DiagStatus::Error => {
                    worst_status = DiagStatus::Warning;
                }
                _ => {}
            }
        }
    }

    DiagnosticsSection {
        title: "Error Budgets".to_string(),
        evidence_id,
        content,
        status: worst_status,
    }
}

fn gather_errors_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let errors = load_recent_errors(20);

    let status = if errors.is_empty() {
        DiagStatus::Ok
    } else if errors.len() >= 10 {
        DiagStatus::Warning
    } else {
        DiagStatus::Ok
    };

    let content = if errors.is_empty() {
        vec!["No recent errors".to_string()]
    } else {
        errors.iter().take(10).map(|e| {
            let error_msg = e.error.as_deref().unwrap_or("unknown");
            // Redact potential secrets
            let redacted = crate::redaction::redact_transcript(error_msg);
            format!(
                "[{}] {}: {} ({})",
                format_epoch(e.timestamp),
                e.op_type,
                redacted,
                e.tool_name.as_deref().unwrap_or("-")
            )
        }).collect()
    };

    DiagnosticsSection {
        title: "Recent Errors".to_string(),
        evidence_id,
        content,
        status,
    }
}

fn gather_alerts_section(counter: &mut u32) -> DiagnosticsSection {
    let evidence_id = format!("DIAG{:05}", *counter);
    *counter += 1;

    let queue = crate::anomaly_engine::AlertQueue::load();
    let active = queue.get_active();
    let (critical, warning, _) = queue.count_by_severity();

    let status = if critical > 0 {
        DiagStatus::Error
    } else if warning > 0 {
        DiagStatus::Warning
    } else {
        DiagStatus::Ok
    };

    let content = if active.is_empty() {
        vec!["No active alerts".to_string()]
    } else {
        let mut lines = vec![format!("{} critical, {} warning", critical, warning)];
        for alert in active.iter().take(10) {
            lines.push(format!("[{}] {}: {}", alert.evidence_id, alert.severity, alert.title));
        }
        lines
    };

    DiagnosticsSection {
        title: "Active Alerts".to_string(),
        evidence_id,
        content,
        status,
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn today_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn days_ago_string(days: i64) -> String {
    let date = chrono::Local::now() - chrono::Duration::days(days);
    date.format("%Y-%m-%d").to_string()
}

fn format_epoch(epoch: u64) -> String {
    use chrono::{TimeZone, Local};
    match Local.timestamp_opt(epoch as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        _ => "unknown".to_string(),
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn get_dir_size(path: &str) -> u64 {
    let mut total: u64 = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += get_dir_size(&entry.path().to_string_lossy());
                }
            }
        }
    }
    total
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_type_as_str() {
        assert_eq!(MetricType::RequestSuccess.as_str(), "request_success");
        assert_eq!(MetricType::ToolFailure.as_str(), "tool_failure");
        assert_eq!(MetricType::MutationRollback.as_str(), "mutation_rollback");
    }

    #[test]
    fn test_daily_metrics_increment() {
        let mut metrics = DailyMetrics::default();
        metrics.increment(MetricType::RequestSuccess);
        metrics.increment(MetricType::RequestSuccess);
        metrics.increment(MetricType::RequestFailure);

        assert_eq!(metrics.get_count(MetricType::RequestSuccess), 2);
        assert_eq!(metrics.get_count(MetricType::RequestFailure), 1);
        assert_eq!(metrics.get_count(MetricType::ToolSuccess), 0);
    }

    #[test]
    fn test_latency_percentile() {
        let mut metrics = DailyMetrics::default();
        for i in 1..=100 {
            metrics.add_latency("e2e", i);
        }

        // P50 should be around 50
        let p50 = metrics.percentile_latency("e2e", 50.0).unwrap();
        assert!(p50 >= 49 && p50 <= 51);

        // P95 should be around 95
        let p95 = metrics.percentile_latency("e2e", 95.0).unwrap();
        assert!(p95 >= 94 && p95 <= 96);
    }

    #[test]
    fn test_budget_state_calculation() {
        // Under budget (0% failure rate)
        assert_eq!(compute_budget_state(0.0, 1.0), BudgetState::Ok);

        // Warning threshold (50%)
        assert_eq!(compute_budget_state(0.5, 1.0), BudgetState::Warning);

        // Critical threshold (80%)
        assert_eq!(compute_budget_state(0.8, 1.0), BudgetState::Critical);

        // Exhausted (100%+)
        assert_eq!(compute_budget_state(1.0, 1.0), BudgetState::Exhausted);
        assert_eq!(compute_budget_state(1.5, 1.0), BudgetState::Exhausted);
    }

    #[test]
    fn test_budget_status_calculation() {
        let mut metrics = DailyMetrics::default();

        // 10 successes, 0 failures = 0% failure rate
        for _ in 0..10 {
            metrics.increment(MetricType::RequestSuccess);
        }

        let budgets = ErrorBudgets::default();
        let statuses = calculate_budget_status(&metrics, &budgets);

        assert!(!statuses.is_empty());
        let request_status = statuses.iter().find(|s| s.category == "request_failures").unwrap();
        assert_eq!(request_status.status, BudgetState::Ok);
        assert_eq!(request_status.current_percent, 0.0);
    }

    #[test]
    fn test_budget_alert_on_failure() {
        let mut metrics = DailyMetrics::default();

        // 90 successes, 10 failures = 10% failure rate (exceeds 1% budget)
        for _ in 0..90 {
            metrics.increment(MetricType::RequestSuccess);
        }
        for _ in 0..10 {
            metrics.increment(MetricType::RequestFailure);
        }

        let budgets = ErrorBudgets::default();
        let statuses = calculate_budget_status(&metrics, &budgets);
        let alerts = check_budget_alerts(&statuses);

        assert!(!alerts.is_empty());
        let alert = &alerts[0];
        assert_eq!(alert.category, "request_failures");
        assert_eq!(alert.severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_budget_warning_threshold() {
        let mut metrics = DailyMetrics::default();

        // 199 successes, 1 failure = 0.5% failure rate (50% of 1% budget = warning)
        for _ in 0..199 {
            metrics.increment(MetricType::RequestSuccess);
        }
        metrics.increment(MetricType::RequestFailure);

        let budgets = ErrorBudgets::default();
        let statuses = calculate_budget_status(&metrics, &budgets);
        let request_status = statuses.iter().find(|s| s.category == "request_failures").unwrap();

        // 0.5% failure rate with 1% budget = 50% burn = warning
        assert_eq!(request_status.status, BudgetState::Warning);
    }

    #[test]
    fn test_error_budgets_default() {
        let budgets = ErrorBudgets::default();
        assert_eq!(budgets.request_failure_percent, 1.0);
        assert_eq!(budgets.tool_failure_percent, 2.0);
        assert_eq!(budgets.mutation_rollback_percent, 0.5);
        assert_eq!(budgets.llm_timeout_percent, 3.0);
    }

    #[test]
    fn test_ops_log_entry() {
        let entry = OpsLogEntry::new("test_op")
            .with_error("test error");

        assert_eq!(entry.op_type, "test_op");
        assert!(!entry.success);
        assert_eq!(entry.error, Some("test error".to_string()));
    }

    #[test]
    fn test_diag_status_as_str() {
        assert_eq!(DiagStatus::Ok.as_str(), "OK");
        assert_eq!(DiagStatus::Warning.as_str(), "WARNING");
        assert_eq!(DiagStatus::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_metrics_store_default() {
        let store = MetricsStore::default();
        assert_eq!(store.schema_version, 1);
        assert_eq!(store.retention_days, 7);
        assert!(store.daily.is_empty());
    }

    #[test]
    fn test_avg_latency() {
        let mut metrics = DailyMetrics::default();
        metrics.add_latency("test", 100);
        metrics.add_latency("test", 200);
        metrics.add_latency("test", 300);

        let avg = metrics.avg_latency("test").unwrap();
        assert_eq!(avg, 200);
    }
}
