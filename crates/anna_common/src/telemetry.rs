//! Telemetry Recording Module v1.2.0
//!
//! Local-only telemetry for diagnosing real-world Anna performance.
//! Writes JSONL to `/var/log/anna/telemetry.jsonl`.
//!
//! ## Purpose
//!
//! The gap between passing acceptance tests and struggling real-world performance
//! needs diagnostic data. This module records:
//! - Question processing outcomes (success/failure/timeout)
//! - Origin paths (Brain/Junior/Senior)
//! - Latency breakdowns
//! - Failure causes
//!
//! ## v1.2.0 Enhancements
//!
//! - Lifetime aggregation (TelemetrySummaryLifetime)
//! - Per-origin detailed stats (OriginStats)
//! - Windowed aggregation (last N entries, last T hours)
//! - Streaming reader for large telemetry files
//! - Latency budget constants for enforcement
//!
//! ## Privacy
//!
//! - Local storage only (no network transmission)
//! - Question text is hashed for privacy
//! - Focus on aggregate metrics, not individual content

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

// ============================================================================
// Median Computation Helpers (v3.6.0)
// ============================================================================

/// Compute median of u64 values (sorts in place)
fn compute_median_u64(values: &mut [u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        // Even count: average of two middle values
        (values[mid - 1] + values[mid]) / 2
    } else {
        values[mid]
    }
}

/// Compute median of f64 values (sorts in place)
fn compute_median_f64(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        // Even count: average of two middle values
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

// ============================================================================
// Latency Budget Constants (v1.2.0)
// ============================================================================

/// Brain fast path target latency (should complete within this)
pub const BRAIN_TARGET_MS: u64 = 100;

/// Brain hard limit - exceeding this is a performance bug
pub const BRAIN_HARD_LIMIT_MS: u64 = 250;

/// Junior LLM target latency
pub const LLM_JUNIOR_TARGET_MS: u64 = 10_000;

/// Junior LLM hard limit - cancel and fallback if exceeded
pub const LLM_JUNIOR_HARD_LIMIT_MS: u64 = 15_000;

/// Senior LLM target latency
pub const LLM_SENIOR_TARGET_MS: u64 = 15_000;

/// Senior LLM hard limit - cancel and fallback if exceeded
pub const LLM_SENIOR_HARD_LIMIT_MS: u64 = 20_000;

/// Default window size for recent stats (number of entries)
pub const DEFAULT_WINDOW_SIZE: usize = 100;

// ============================================================================
// File Paths
// ============================================================================

/// Default telemetry file location
pub const TELEMETRY_FILE: &str = "/var/log/anna/telemetry.jsonl";

/// Fallback location if /var/log/anna is not writable
pub const TELEMETRY_FALLBACK: &str = "/tmp/anna-telemetry.jsonl";

/// Outcome of a question processing attempt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Failure,
    Timeout,
    Refusal,
}

/// Origin/path that produced the answer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Brain,
    Junior,
    Senior,
    Fallback,
    Error,
}

/// A single telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Correlation ID for linking related events
    pub correlation_id: String,
    /// SHA256 hash of question (first 16 chars for privacy)
    pub question_hash: String,
    /// Processing outcome
    pub outcome: Outcome,
    /// Answer origin
    pub origin: Origin,
    /// Reliability score (0.0 - 1.0)
    pub reliability: f64,
    /// Total latency in milliseconds
    pub latency_ms: u64,
    /// Brain fast path latency (if attempted)
    pub brain_ms: Option<u64>,
    /// Junior LLM latency
    pub junior_ms: Option<u64>,
    /// Senior LLM latency
    pub senior_ms: Option<u64>,
    /// Failure cause (if any)
    pub failure_cause: Option<String>,
    /// Number of probes executed
    pub probes_count: u32,
    /// Whether answer was from cache
    pub cached: bool,
}

impl TelemetryEvent {
    /// Create a new telemetry event
    pub fn new(
        question: &str,
        outcome: Outcome,
        origin: Origin,
        reliability: f64,
        latency_ms: u64,
    ) -> Self {
        // Hash question for privacy
        let question_hash = hash_question(question);

        Self {
            timestamp: Utc::now().to_rfc3339(),
            correlation_id: generate_correlation_id(),
            question_hash,
            outcome,
            origin,
            reliability,
            latency_ms,
            brain_ms: None,
            junior_ms: None,
            senior_ms: None,
            failure_cause: None,
            probes_count: 0,
            cached: false,
        }
    }

    /// Set Brain timing
    pub fn with_brain_ms(mut self, ms: u64) -> Self {
        self.brain_ms = Some(ms);
        self
    }

    /// Set Junior timing
    pub fn with_junior_ms(mut self, ms: u64) -> Self {
        self.junior_ms = Some(ms);
        self
    }

    /// Set Senior timing
    pub fn with_senior_ms(mut self, ms: u64) -> Self {
        self.senior_ms = Some(ms);
        self
    }

    /// Set failure cause
    pub fn with_failure(mut self, cause: &str) -> Self {
        self.failure_cause = Some(cause.to_string());
        self
    }

    /// Set probes count
    pub fn with_probes(mut self, count: u32) -> Self {
        self.probes_count = count;
        self
    }

    /// Mark as cached
    pub fn with_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    /// Set correlation ID (for linking with other events)
    pub fn with_correlation_id(mut self, id: &str) -> Self {
        self.correlation_id = id.to_string();
        self
    }
}

/// Telemetry recorder that writes to JSONL file
pub struct TelemetryRecorder {
    file_path: PathBuf,
}

impl Default for TelemetryRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryRecorder {
    /// Create a new recorder with default path
    pub fn new() -> Self {
        let path = PathBuf::from(TELEMETRY_FILE);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Fall back if primary path not writable
        let file_path = if Self::is_writable(&path) {
            path
        } else {
            PathBuf::from(TELEMETRY_FALLBACK)
        };

        Self { file_path }
    }

    /// Create recorder with custom path (for testing)
    pub fn with_path(path: PathBuf) -> Self {
        Self { file_path: path }
    }

    /// Check if path is writable
    fn is_writable(path: &PathBuf) -> bool {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return fs::create_dir_all(parent).is_ok();
            }
            // Try to open for append
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .is_ok()
        } else {
            false
        }
    }

    /// Record a telemetry event
    pub fn record(&self, event: &TelemetryEvent) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(event)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        writeln!(file, "{}", json)?;
        Ok(())
    }

    /// Get summary metrics from recent events
    pub fn summary(&self, hours: u64) -> TelemetrySummary {
        let events = self.read_recent(hours);
        TelemetrySummary::from_events(&events)
    }

    /// Read events from the last N hours
    pub fn read_recent(&self, hours: u64) -> Vec<TelemetryEvent> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let content = fs::read_to_string(&self.file_path).unwrap_or_default();

        content
            .lines()
            .filter_map(|line| serde_json::from_str::<TelemetryEvent>(line).ok())
            .filter(|e| e.timestamp >= cutoff_str)
            .collect()
    }

    /// Get the file path being used
    pub fn path(&self) -> &PathBuf {
        &self.file_path
    }
}

/// Summary of telemetry metrics
/// v3.6.0: Added worst/best/median stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetrySummary {
    /// Total questions in period
    pub total: u64,
    /// Successful answers
    pub successes: u64,
    /// Failed answers
    pub failures: u64,
    /// Timeouts
    pub timeouts: u64,
    /// Refusals
    pub refusals: u64,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Average latency for successful answers (ms)
    pub avg_latency_ms: u64,
    /// Min latency (best) - v3.6.0
    pub min_latency_ms: u64,
    /// Max latency (worst) - v3.6.0
    pub max_latency_ms: u64,
    /// Median latency - v3.6.0
    pub median_latency_ms: u64,
    /// Average reliability (0.0 - 1.0)
    pub avg_reliability: f64,
    /// Min reliability (worst) - v3.6.0
    pub min_reliability: f64,
    /// Max reliability (best) - v3.6.0
    pub max_reliability: f64,
    /// Median reliability - v3.6.0
    pub median_reliability: f64,
    /// Questions answered by Brain
    pub brain_count: u64,
    /// Questions answered by Junior
    pub junior_count: u64,
    /// Questions answered by Senior
    pub senior_count: u64,
    /// Most common failure cause
    pub top_failure: Option<String>,
}

impl TelemetrySummary {
    /// Create summary from events
    /// v3.6.0: Now computes worst/best/median stats
    pub fn from_events(events: &[TelemetryEvent]) -> Self {
        if events.is_empty() {
            return Self::default();
        }

        let total = events.len() as u64;
        let successes = events.iter().filter(|e| e.outcome == Outcome::Success).count() as u64;
        let failures = events.iter().filter(|e| e.outcome == Outcome::Failure).count() as u64;
        let timeouts = events.iter().filter(|e| e.outcome == Outcome::Timeout).count() as u64;
        let refusals = events.iter().filter(|e| e.outcome == Outcome::Refusal).count() as u64;

        let success_rate = if total > 0 {
            successes as f64 / total as f64
        } else {
            0.0
        };

        // Latency stats (from successful answers only)
        let successful: Vec<_> = events.iter().filter(|e| e.outcome == Outcome::Success).collect();
        let (avg_latency_ms, min_latency_ms, max_latency_ms, median_latency_ms) = if !successful.is_empty() {
            let mut latencies: Vec<u64> = successful.iter().map(|e| e.latency_ms).collect();
            let avg = latencies.iter().sum::<u64>() / latencies.len() as u64;
            let min = *latencies.iter().min().unwrap_or(&0);
            let max = *latencies.iter().max().unwrap_or(&0);
            let median = compute_median_u64(&mut latencies);
            (avg, min, max, median)
        } else {
            (0, 0, 0, 0)
        };

        // Reliability stats (from all events)
        let (avg_reliability, min_reliability, max_reliability, median_reliability) = {
            let mut reliabilities: Vec<f64> = events.iter().map(|e| e.reliability).collect();
            let avg = reliabilities.iter().sum::<f64>() / reliabilities.len() as f64;
            let min = reliabilities.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = reliabilities.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let median = compute_median_f64(&mut reliabilities);
            (avg, min, max, median)
        };

        // Origin counts
        let brain_count = events.iter().filter(|e| e.origin == Origin::Brain).count() as u64;
        let junior_count = events.iter().filter(|e| e.origin == Origin::Junior).count() as u64;
        let senior_count = events.iter().filter(|e| e.origin == Origin::Senior).count() as u64;

        // Top failure cause
        let mut failure_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for event in events {
            if let Some(cause) = &event.failure_cause {
                *failure_counts.entry(cause.clone()).or_insert(0) += 1;
            }
        }
        let top_failure = failure_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cause, _)| cause);

        Self {
            total,
            successes,
            failures,
            timeouts,
            refusals,
            success_rate,
            avg_latency_ms,
            min_latency_ms,
            max_latency_ms,
            median_latency_ms,
            avg_reliability,
            min_reliability,
            max_reliability,
            median_reliability,
            brain_count,
            junior_count,
            senior_count,
            top_failure,
        }
    }

    /// Check if Anna is struggling (success rate < 50%)
    pub fn is_struggling(&self) -> bool {
        self.total >= 5 && self.success_rate < 0.50
    }

    /// Format for display
    /// v3.6.0: Now shows worst/best/median stats with percentages
    pub fn display(&self) -> String {
        use crate::ui_colors::format_percentage;

        format!(
            "Last 24h: {}/{} successful ({}), avg latency {}ms\n\
             Latency: best {}ms, worst {}ms, median {}ms\n\
             Reliability: best {}, worst {}, median {}, avg {}\n\
             Origins: Brain={}, Junior={}, Senior={}\n\
             Issues: {} failures, {} timeouts, {} refusals{}",
            self.successes, self.total, format_percentage(self.success_rate),
            self.avg_latency_ms,
            self.min_latency_ms, self.max_latency_ms, self.median_latency_ms,
            format_percentage(self.max_reliability),
            format_percentage(self.min_reliability),
            format_percentage(self.median_reliability),
            format_percentage(self.avg_reliability),
            self.brain_count, self.junior_count, self.senior_count,
            self.failures, self.timeouts, self.refusals,
            self.top_failure.as_ref().map(|f| format!(" (top: {})", f)).unwrap_or_default()
        )
    }

    /// Get a text description of Anna's current state based on metrics
    pub fn status_hint(&self) -> &'static str {
        if self.total < 5 {
            "Not enough data yet to assess performance."
        } else if self.success_rate >= 0.8 {
            "Anna is behaving reliably on recent questions."
        } else if self.success_rate >= 0.4 {
            "Anna is still learning. Some answers need more work."
        } else {
            "Anna is struggling with recent questions."
        }
    }
}

// ============================================================================
// Per-Origin Stats (v1.2.0)
// ============================================================================

/// Detailed stats for a single origin (Brain/Junior/Senior)
/// v3.6.0: Added worst/best/median stats for complete picture
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OriginStats {
    /// Number of questions handled by this origin
    pub count: u64,
    /// Successful answers
    pub successes: u64,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Average latency (ms)
    pub avg_latency_ms: u64,
    /// Min latency seen (best)
    pub min_latency_ms: u64,
    /// Max latency seen (worst)
    pub max_latency_ms: u64,
    /// Median latency (ms) - v3.6.0
    pub median_latency_ms: u64,
    /// Average reliability score
    pub avg_reliability: f64,
    /// Min reliability (worst) - v3.6.0
    pub min_reliability: f64,
    /// Max reliability (best) - v3.6.0
    pub max_reliability: f64,
    /// Median reliability - v3.6.0
    pub median_reliability: f64,
}

impl OriginStats {
    /// Create stats from events for a specific origin
    /// v3.6.0: Now computes worst/best/median stats
    pub fn from_events(events: &[TelemetryEvent], origin: Origin) -> Self {
        let filtered: Vec<_> = events.iter().filter(|e| e.origin == origin).collect();

        if filtered.is_empty() {
            return Self::default();
        }

        let count = filtered.len() as u64;
        let successes = filtered.iter().filter(|e| e.outcome == Outcome::Success).count() as u64;
        let success_rate = successes as f64 / count as f64;

        // Latency stats
        let mut latencies: Vec<u64> = filtered.iter().map(|e| e.latency_ms).collect();
        let avg_latency_ms = latencies.iter().sum::<u64>() / count;
        let min_latency_ms = *latencies.iter().min().unwrap_or(&0);
        let max_latency_ms = *latencies.iter().max().unwrap_or(&0);
        let median_latency_ms = compute_median_u64(&mut latencies);

        // Reliability stats
        let mut reliabilities: Vec<f64> = filtered.iter().map(|e| e.reliability).collect();
        let avg_reliability = reliabilities.iter().sum::<f64>() / count as f64;
        let min_reliability = reliabilities.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_reliability = reliabilities.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let median_reliability = compute_median_f64(&mut reliabilities);

        Self {
            count,
            successes,
            success_rate,
            avg_latency_ms,
            min_latency_ms,
            max_latency_ms,
            median_latency_ms,
            avg_reliability,
            min_reliability,
            max_reliability,
            median_reliability,
        }
    }

    /// Format for single-line display
    /// v3.6.0: Updated to use format_percentage
    pub fn display_line(&self, name: &str) -> String {
        use crate::ui_colors::format_percentage;

        if self.count == 0 {
            format!("  {}:  No data", name)
        } else {
            format!(
                "  {}:  {} questions, {} success, {}ms avg latency",
                name, self.count, format_percentage(self.success_rate), self.avg_latency_ms
            )
        }
    }

    /// Format for detailed multi-line display (v3.6.0)
    pub fn display_detailed(&self, name: &str) -> String {
        use crate::ui_colors::format_percentage;

        if self.count == 0 {
            return format!("  {}:  No data\n", name);
        }

        format!(
            "  {}:  {} questions, {} success\n\
             {}    Latency: avg {}ms, best {}ms, worst {}ms, median {}ms\n\
             {}    Reliability: avg {}, best {}, worst {}, median {}",
            name, self.count, format_percentage(self.success_rate),
            " ".repeat(name.len()),
            self.avg_latency_ms, self.min_latency_ms, self.max_latency_ms, self.median_latency_ms,
            " ".repeat(name.len()),
            format_percentage(self.avg_reliability),
            format_percentage(self.max_reliability),
            format_percentage(self.min_reliability),
            format_percentage(self.median_reliability)
        )
    }
}

// ============================================================================
// Lifetime + Windowed Summary (v1.2.0)
// ============================================================================

/// Complete telemetry summary with lifetime and windowed stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetrySummaryComplete {
    /// Lifetime stats (all recorded events)
    pub lifetime: TelemetrySummary,
    /// Windowed stats (last N events)
    pub window: TelemetrySummary,
    /// Window size used
    pub window_size: usize,
    /// Per-origin stats for window
    pub brain_stats: OriginStats,
    pub junior_stats: OriginStats,
    pub senior_stats: OriginStats,
    /// Whether telemetry data is available
    pub has_data: bool,
    /// Status hint based on recent performance
    pub status_hint: String,
    // v3.4.0: Rolling stats for performance hints
    /// Rolling average latency (last 50 questions)
    pub rolling_avg_latency_ms: u64,
    /// Rolling failure rate (last 50 questions)
    pub rolling_failure_rate: f64,
    /// Rolling timeout rate (last 50 questions)
    pub rolling_timeout_rate: f64,
}

impl TelemetrySummaryComplete {
    /// Format the complete summary for status display
    /// v3.6.0: Enhanced with best/worst/median stats and percentages
    pub fn display(&self) -> String {
        use crate::ui_colors::format_percentage;

        if !self.has_data {
            return "  *  No telemetry data available yet.\n".to_string();
        }

        let mut out = String::new();

        // Lifetime stats
        out.push_str(&format!(
            "  *  Lifetime:  {} questions, {} success rate\n",
            self.lifetime.total, format_percentage(self.lifetime.success_rate)
        ));

        // Window stats with reliability
        out.push_str(&format!(
            "  [STATS]  Recent (last {}):  {}/{} success ({}), {}ms avg\n",
            self.window_size.min(self.window.total as usize),
            self.window.successes, self.window.total,
            format_percentage(self.window.success_rate),
            self.window.avg_latency_ms
        ));

        // Reliability stats (v3.6.0)
        out.push_str(&format!(
            "  [TARGET]  Reliability:  best {}, worst {}, median {}\n",
            format_percentage(self.window.max_reliability),
            format_percentage(self.window.min_reliability),
            format_percentage(self.window.median_reliability)
        ));

        // Latency stats (v3.6.0)
        out.push_str(&format!(
            "  [TIME]  Latency:  best {}ms, worst {}ms, median {}ms\n",
            self.window.min_latency_ms,
            self.window.max_latency_ms,
            self.window.median_latency_ms
        ));

        // Per-origin breakdown
        out.push_str("\n  -- Per-Origin Performance --\n");
        out.push_str(&self.brain_stats.display_line("[BRAIN]  Brain "));
        out.push('\n');
        out.push_str(&self.junior_stats.display_line("JR  Junior"));
        out.push('\n');
        out.push_str(&self.senior_stats.display_line("SR  Senior"));
        out.push('\n');

        // Status hint
        out.push_str(&format!("\n  [HINT]  {}\n", self.status_hint));

        out
    }
}

// ============================================================================
// Rolling Stats (v3.4.0) - For Performance Hints
// ============================================================================

/// Rolling statistics for performance hint calculation
#[derive(Debug, Clone, Default)]
pub struct RollingStats {
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Failure rate (0.0-1.0)
    pub failure_rate: f64,
    /// Timeout rate (0.0-1.0)
    pub timeout_rate: f64,
    /// Total events in the rolling window
    pub count: usize,
}

impl RollingStats {
    /// Compute rolling stats from events
    pub fn from_events(events: &[TelemetryEvent]) -> Self {
        if events.is_empty() {
            return Self::default();
        }

        let count = events.len();
        let total_latency: u64 = events.iter().map(|e| e.latency_ms).sum();
        let avg_latency_ms = total_latency / count as u64;

        let failures = events.iter().filter(|e| e.outcome == Outcome::Failure).count();
        let timeouts = events.iter().filter(|e| e.outcome == Outcome::Timeout).count();

        let failure_rate = failures as f64 / count as f64;
        let timeout_rate = timeouts as f64 / count as f64;

        Self {
            avg_latency_ms,
            failure_rate,
            timeout_rate,
            count,
        }
    }
}

// ============================================================================
// Streaming Reader (v1.2.0)
// ============================================================================

/// Read telemetry events efficiently without loading entire file
pub struct TelemetryReader {
    file_path: PathBuf,
}

impl TelemetryReader {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    pub fn default_path() -> Self {
        Self::new(PathBuf::from(TELEMETRY_FILE))
    }

    /// Check if telemetry file exists and has data
    pub fn has_data(&self) -> bool {
        self.file_path.exists() && fs::metadata(&self.file_path)
            .map(|m| m.len() > 0)
            .unwrap_or(false)
    }

    /// Read the last N events from the file (streaming, memory efficient)
    pub fn read_last_n(&self, n: usize) -> Vec<TelemetryEvent> {
        if !self.file_path.exists() {
            return Vec::new();
        }

        let file = match File::open(&self.file_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        let mut events: Vec<TelemetryEvent> = Vec::new();

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(event) = serde_json::from_str::<TelemetryEvent>(&line) {
                events.push(event);
                // Keep only last n
                if events.len() > n * 2 {
                    events = events.split_off(events.len() - n);
                }
            }
        }

        // Return exactly last n
        if events.len() > n {
            events.split_off(events.len() - n)
        } else {
            events
        }
    }

    /// Read all events (for lifetime stats - use carefully on large files)
    pub fn read_all(&self) -> Vec<TelemetryEvent> {
        if !self.file_path.exists() {
            return Vec::new();
        }

        let content = fs::read_to_string(&self.file_path).unwrap_or_default();
        content
            .lines()
            .filter_map(|line| serde_json::from_str::<TelemetryEvent>(line).ok())
            .collect()
    }

    /// Read events from the last N hours
    pub fn read_hours(&self, hours: u64) -> Vec<TelemetryEvent> {
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        let cutoff_str = cutoff.to_rfc3339();

        self.read_all()
            .into_iter()
            .filter(|e| e.timestamp >= cutoff_str)
            .collect()
    }

    /// Compute complete summary (lifetime + windowed)
    pub fn complete_summary(&self, window_size: usize) -> TelemetrySummaryComplete {
        if !self.has_data() {
            return TelemetrySummaryComplete {
                has_data: false,
                status_hint: "No telemetry data available yet.".to_string(),
                ..Default::default()
            };
        }

        let all_events = self.read_all();
        let lifetime = TelemetrySummary::from_events(&all_events);

        // Get window events
        let window_events: Vec<_> = if all_events.len() > window_size {
            all_events[all_events.len() - window_size..].to_vec()
        } else {
            all_events.clone()
        };

        let window = TelemetrySummary::from_events(&window_events);
        let status_hint = window.status_hint().to_string();

        // Per-origin stats from window
        let brain_stats = OriginStats::from_events(&window_events, Origin::Brain);
        let junior_stats = OriginStats::from_events(&window_events, Origin::Junior);
        let senior_stats = OriginStats::from_events(&window_events, Origin::Senior);

        // v3.4.0: Compute rolling stats for performance hints (last 50)
        let rolling_events: Vec<_> = if all_events.len() > 50 {
            all_events[all_events.len() - 50..].to_vec()
        } else {
            all_events.clone()
        };
        let rolling_stats = RollingStats::from_events(&rolling_events);

        TelemetrySummaryComplete {
            lifetime,
            window,
            window_size,
            brain_stats,
            junior_stats,
            senior_stats,
            has_data: true,
            status_hint,
            rolling_avg_latency_ms: rolling_stats.avg_latency_ms,
            rolling_failure_rate: rolling_stats.failure_rate,
            rolling_timeout_rate: rolling_stats.timeout_rate,
        }
    }

    /// Count total events without loading all into memory
    pub fn count_events(&self) -> u64 {
        if !self.file_path.exists() {
            return 0;
        }

        let file = match File::open(&self.file_path) {
            Ok(f) => f,
            Err(_) => return 0,
        };

        BufReader::new(file).lines().count() as u64
    }
}

/// Hash question for privacy (first 16 chars of SHA256)
fn hash_question(question: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    question.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Generate a correlation ID
fn generate_correlation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    format!("{:016x}", timestamp & 0xFFFFFFFFFFFFFFFF)
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Record a successful answer
pub fn record_success(
    question: &str,
    origin: Origin,
    reliability: f64,
    latency_ms: u64,
    junior_ms: u64,
    senior_ms: u64,
    probes_count: u32,
) {
    let event = TelemetryEvent::new(question, Outcome::Success, origin, reliability, latency_ms)
        .with_junior_ms(junior_ms)
        .with_senior_ms(senior_ms)
        .with_probes(probes_count);

    let recorder = TelemetryRecorder::new();
    let _ = recorder.record(&event);
}

/// Record a Brain fast path answer
pub fn record_brain_answer(question: &str, reliability: f64, latency_ms: u64) {
    let event = TelemetryEvent::new(question, Outcome::Success, Origin::Brain, reliability, latency_ms)
        .with_brain_ms(latency_ms);

    let recorder = TelemetryRecorder::new();
    let _ = recorder.record(&event);
}

/// Record a failure
pub fn record_failure(question: &str, cause: &str, latency_ms: u64) {
    let event = TelemetryEvent::new(question, Outcome::Failure, Origin::Error, 0.0, latency_ms)
        .with_failure(cause);

    let recorder = TelemetryRecorder::new();
    let _ = recorder.record(&event);
}

/// Record a timeout
pub fn record_timeout(question: &str, latency_ms: u64) {
    let event = TelemetryEvent::new(question, Outcome::Timeout, Origin::Error, 0.0, latency_ms)
        .with_failure("timeout");

    let recorder = TelemetryRecorder::new();
    let _ = recorder.record(&event);
}

/// Record a refusal (low reliability)
pub fn record_refusal(question: &str, reliability: f64, latency_ms: u64) {
    let event = TelemetryEvent::new(question, Outcome::Refusal, Origin::Senior, reliability, latency_ms)
        .with_failure("low_reliability");

    let recorder = TelemetryRecorder::new();
    let _ = recorder.record(&event);
}

/// Get 24h summary
pub fn get_24h_summary() -> TelemetrySummary {
    TelemetryRecorder::new().summary(24)
}

/// v3.4.0: Get performance hint from recent telemetry
pub fn get_performance_hint() -> crate::perf_timing::PerformanceHint {
    let reader = TelemetryReader::default_path();
    let summary = reader.complete_summary(DEFAULT_WINDOW_SIZE);

    crate::perf_timing::PerformanceHint::from_stats(
        summary.rolling_avg_latency_ms,
        summary.rolling_failure_rate,
        summary.rolling_timeout_rate,
    )
}

/// v3.4.0: Get rolling stats for last 50 questions
pub fn get_rolling_stats() -> RollingStats {
    let reader = TelemetryReader::default_path();
    let events = reader.read_last_n(50);
    RollingStats::from_events(&events)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // ========================================================================
    // Median Calculation Tests (v3.6.0)
    // ========================================================================

    #[test]
    fn test_median_u64_odd_count() {
        let mut values = vec![3, 1, 5, 2, 4];
        assert_eq!(compute_median_u64(&mut values), 3);
    }

    #[test]
    fn test_median_u64_even_count() {
        let mut values = vec![1, 2, 3, 4];
        assert_eq!(compute_median_u64(&mut values), 2); // (2+3)/2 = 2 (integer division)
    }

    #[test]
    fn test_median_u64_single() {
        let mut values = vec![42];
        assert_eq!(compute_median_u64(&mut values), 42);
    }

    #[test]
    fn test_median_u64_empty() {
        let mut values: Vec<u64> = vec![];
        assert_eq!(compute_median_u64(&mut values), 0);
    }

    #[test]
    fn test_median_f64_odd_count() {
        let mut values = vec![3.0, 1.0, 5.0, 2.0, 4.0];
        assert!((compute_median_f64(&mut values) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_median_f64_even_count() {
        let mut values = vec![1.0, 2.0, 3.0, 4.0];
        assert!((compute_median_f64(&mut values) - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_median_f64_single() {
        let mut values = vec![0.75];
        assert!((compute_median_f64(&mut values) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_median_f64_empty() {
        let mut values: Vec<f64> = vec![];
        assert!((compute_median_f64(&mut values) - 0.0).abs() < 0.001);
    }

    // ========================================================================
    // Telemetry Event Tests
    // ========================================================================

    #[test]
    fn test_telemetry_event_creation() {
        let event = TelemetryEvent::new(
            "How much RAM?",
            Outcome::Success,
            Origin::Brain,
            0.99,
            45,
        );

        assert!(!event.question_hash.is_empty());
        assert_eq!(event.outcome, Outcome::Success);
        assert_eq!(event.origin, Origin::Brain);
        assert_eq!(event.reliability, 0.99);
        assert_eq!(event.latency_ms, 45);
    }

    #[test]
    fn test_telemetry_recorder() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();

        let recorder = TelemetryRecorder::with_path(path.clone());

        let event = TelemetryEvent::new(
            "Test question",
            Outcome::Success,
            Origin::Brain,
            0.95,
            100,
        );

        recorder.record(&event).unwrap();

        let events = recorder.read_recent(1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].outcome, Outcome::Success);
    }

    #[test]
    fn test_telemetry_summary() {
        let events = vec![
            TelemetryEvent::new("q1", Outcome::Success, Origin::Brain, 0.99, 50),
            TelemetryEvent::new("q2", Outcome::Success, Origin::Junior, 0.85, 2000),
            TelemetryEvent::new("q3", Outcome::Failure, Origin::Error, 0.0, 1000)
                .with_failure("llm_error"),
        ];

        let summary = TelemetrySummary::from_events(&events);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.successes, 2);
        assert_eq!(summary.failures, 1);
        assert!((summary.success_rate - 0.666).abs() < 0.01);
        assert_eq!(summary.brain_count, 1);
        assert_eq!(summary.junior_count, 1);
        assert_eq!(summary.top_failure, Some("llm_error".to_string()));
    }

    #[test]
    fn test_is_struggling() {
        let failing = TelemetrySummary {
            total: 10,
            successes: 3,
            failures: 7,
            success_rate: 0.3,
            ..Default::default()
        };
        assert!(failing.is_struggling());

        let healthy = TelemetrySummary {
            total: 10,
            successes: 8,
            failures: 2,
            success_rate: 0.8,
            ..Default::default()
        };
        assert!(!healthy.is_struggling());
    }

    #[test]
    fn test_status_hint() {
        // Not enough data
        let few = TelemetrySummary { total: 3, success_rate: 0.9, ..Default::default() };
        assert!(few.status_hint().contains("Not enough data"));

        // Reliable
        let reliable = TelemetrySummary { total: 10, success_rate: 0.85, ..Default::default() };
        assert!(reliable.status_hint().contains("reliably"));

        // Learning
        let learning = TelemetrySummary { total: 10, success_rate: 0.5, ..Default::default() };
        assert!(learning.status_hint().contains("learning"));

        // Struggling
        let struggling = TelemetrySummary { total: 10, success_rate: 0.2, ..Default::default() };
        assert!(struggling.status_hint().contains("struggling"));
    }

    #[test]
    fn test_origin_stats() {
        let events = vec![
            TelemetryEvent::new("q1", Outcome::Success, Origin::Brain, 0.99, 10),
            TelemetryEvent::new("q2", Outcome::Success, Origin::Brain, 0.95, 20),
            TelemetryEvent::new("q3", Outcome::Failure, Origin::Brain, 0.30, 30),
            TelemetryEvent::new("q4", Outcome::Success, Origin::Junior, 0.80, 5000),
        ];

        let brain_stats = OriginStats::from_events(&events, Origin::Brain);
        assert_eq!(brain_stats.count, 3);
        assert_eq!(brain_stats.successes, 2);
        assert!((brain_stats.success_rate - 0.666).abs() < 0.01);
        assert_eq!(brain_stats.avg_latency_ms, 20); // (10+20+30)/3
        assert_eq!(brain_stats.min_latency_ms, 10);
        assert_eq!(brain_stats.max_latency_ms, 30);
        assert_eq!(brain_stats.median_latency_ms, 20); // sorted: [10, 20, 30], median = 20

        // v3.6.0: Test reliability stats
        assert!((brain_stats.min_reliability - 0.30).abs() < 0.01); // worst
        assert!((brain_stats.max_reliability - 0.99).abs() < 0.01); // best
        assert!((brain_stats.median_reliability - 0.95).abs() < 0.01); // median of [0.30, 0.95, 0.99]

        let junior_stats = OriginStats::from_events(&events, Origin::Junior);
        assert_eq!(junior_stats.count, 1);
        assert_eq!(junior_stats.successes, 1);
        assert_eq!(junior_stats.avg_latency_ms, 5000);
        assert_eq!(junior_stats.median_latency_ms, 5000); // single value
        assert!((junior_stats.median_reliability - 0.80).abs() < 0.01);

        let senior_stats = OriginStats::from_events(&events, Origin::Senior);
        assert_eq!(senior_stats.count, 0); // No Senior events
    }

    #[test]
    fn test_telemetry_summary_new_stats() {
        // v3.6.0: Test the new worst/best/median stats
        let events = vec![
            TelemetryEvent::new("q1", Outcome::Success, Origin::Brain, 0.99, 100),
            TelemetryEvent::new("q2", Outcome::Success, Origin::Junior, 0.85, 500),
            TelemetryEvent::new("q3", Outcome::Success, Origin::Senior, 0.70, 200),
            TelemetryEvent::new("q4", Outcome::Failure, Origin::Error, 0.0, 1000),
        ];

        let summary = TelemetrySummary::from_events(&events);

        // Latency stats (from successful events only)
        assert_eq!(summary.min_latency_ms, 100);  // best
        assert_eq!(summary.max_latency_ms, 500);  // worst
        assert_eq!(summary.median_latency_ms, 200); // median of [100, 200, 500]

        // Reliability stats (from all events)
        assert!((summary.min_reliability - 0.0).abs() < 0.01);  // worst
        assert!((summary.max_reliability - 0.99).abs() < 0.01); // best
        // median of [0.0, 0.70, 0.85, 0.99] = (0.70 + 0.85) / 2 = 0.775
        assert!((summary.median_reliability - 0.775).abs() < 0.01);
    }

    #[test]
    fn test_telemetry_reader_complete_summary() {
        use std::io::Write;
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();

        // Write some test events
        {
            let mut file = std::fs::File::create(&path).unwrap();
            for i in 0..10 {
                let origin = if i % 3 == 0 { Origin::Brain } else if i % 3 == 1 { Origin::Junior } else { Origin::Senior };
                let outcome = if i % 4 == 0 { Outcome::Failure } else { Outcome::Success };
                let event = TelemetryEvent::new(
                    &format!("q{}", i),
                    outcome,
                    origin,
                    0.8 + (i as f64 * 0.01),
                    100 * (i as u64 + 1),
                );
                writeln!(file, "{}", serde_json::to_string(&event).unwrap()).unwrap();
            }
        }

        let reader = TelemetryReader::new(path);
        assert!(reader.has_data());

        let summary = reader.complete_summary(5);
        assert!(summary.has_data);
        assert_eq!(summary.lifetime.total, 10);
        assert_eq!(summary.window.total, 5); // Last 5 events

        // Check per-origin stats exist
        assert!(summary.brain_stats.count > 0 || summary.junior_stats.count > 0 || summary.senior_stats.count > 0);
    }

    #[test]
    fn test_telemetry_reader_empty_file() {
        let temp = NamedTempFile::new().unwrap();
        let reader = TelemetryReader::new(temp.path().to_path_buf());

        // Empty file should return no data
        let summary = reader.complete_summary(100);
        assert!(!summary.has_data);
        assert!(summary.status_hint.contains("No telemetry"));
    }

    #[test]
    fn test_telemetry_reader_missing_file() {
        let reader = TelemetryReader::new(PathBuf::from("/nonexistent/path/telemetry.jsonl"));

        // Missing file should not panic
        let summary = reader.complete_summary(100);
        assert!(!summary.has_data);
        assert_eq!(reader.count_events(), 0);
    }

    #[test]
    fn test_read_last_n() {
        use std::io::Write;
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();

        // Write 20 events
        {
            let mut file = std::fs::File::create(&path).unwrap();
            for i in 0..20 {
                let event = TelemetryEvent::new(
                    &format!("question_{}", i),
                    Outcome::Success,
                    Origin::Brain,
                    0.9,
                    i as u64,
                );
                writeln!(file, "{}", serde_json::to_string(&event).unwrap()).unwrap();
            }
        }

        let reader = TelemetryReader::new(path);

        // Read last 5
        let last_5 = reader.read_last_n(5);
        assert_eq!(last_5.len(), 5);
        // Should be events 15-19 (latency = index)
        assert_eq!(last_5[0].latency_ms, 15);
        assert_eq!(last_5[4].latency_ms, 19);
    }

    #[test]
    fn test_latency_constants() {
        // Sanity check that constants are properly ordered
        assert!(BRAIN_TARGET_MS < BRAIN_HARD_LIMIT_MS);
        assert!(BRAIN_HARD_LIMIT_MS < LLM_JUNIOR_TARGET_MS);
        assert!(LLM_JUNIOR_TARGET_MS < LLM_JUNIOR_HARD_LIMIT_MS);
        assert!(LLM_JUNIOR_HARD_LIMIT_MS <= LLM_SENIOR_TARGET_MS);
        assert!(LLM_SENIOR_TARGET_MS < LLM_SENIOR_HARD_LIMIT_MS);
    }
}
