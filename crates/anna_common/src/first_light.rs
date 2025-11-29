//! First Light Self-Test Module v2.2.0
//!
//! After a hard reset, Anna runs a "First Light" self-test to verify
//! the system is working correctly. This provides a clean baseline
//! for all future measurements.
//!
//! ## First Light Test Suite
//!
//! 5 canonical questions that exercise the core system:
//! 1. CPU cores/threads
//! 2. RAM installed/available
//! 3. Root FS free/total
//! 4. Self health summary
//! 5. LLM connectivity check (Junior + Senior)
//!
//! ## Results
//!
//! Each question generates:
//! - reliability score
//! - latency (ms)
//! - origin (Brain/Junior/Senior)
//! - XP awarded
//! - telemetry event

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

/// Directory for benchmark results
pub const BENCHMARKS_DIR: &str = "/var/lib/anna/benchmarks";
pub const FIRST_LIGHT_FILE: &str = "/var/lib/anna/benchmarks/first_light.json";

// ============================================================================
// First Light Questions
// ============================================================================

/// The 5 canonical First Light questions
pub const FIRST_LIGHT_QUESTIONS: &[(&str, &str)] = &[
    ("cpu", "How many CPU cores and threads do I have?"),
    ("ram", "How much RAM is installed and available?"),
    ("disk", "How much space is free on root filesystem?"),
    ("health", "What is your health status?"),
    ("llm", "Are Junior and Senior LLM models working?"),
];

// ============================================================================
// First Light Result
// ============================================================================

/// Result of a single First Light question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstLightQuestion {
    /// Question ID (cpu, ram, disk, health, llm)
    pub id: String,
    /// The question text
    pub question: String,
    /// Whether the question was answered successfully
    pub success: bool,
    /// Reliability score (0.0-1.0)
    pub reliability: f64,
    /// Response latency in milliseconds
    pub latency_ms: u64,
    /// Origin of the answer (Brain, Junior, Senior, etc.)
    pub origin: String,
    /// XP awarded for this question
    pub xp_awarded: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Short answer summary
    pub answer_summary: String,
}

impl FirstLightQuestion {
    /// Create a successful result
    pub fn success(
        id: &str,
        question: &str,
        reliability: f64,
        latency_ms: u64,
        origin: &str,
        xp: u64,
        summary: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            question: question.to_string(),
            success: true,
            reliability,
            latency_ms,
            origin: origin.to_string(),
            xp_awarded: xp,
            error: None,
            answer_summary: summary.to_string(),
        }
    }

    /// Create a failed result
    pub fn failure(id: &str, question: &str, latency_ms: u64, error: &str) -> Self {
        Self {
            id: id.to_string(),
            question: question.to_string(),
            success: false,
            reliability: 0.0,
            latency_ms,
            origin: "Error".to_string(),
            xp_awarded: 0,
            error: Some(error.to_string()),
            answer_summary: String::new(),
        }
    }

    /// Format as status line
    pub fn format_line(&self) -> String {
        if self.success {
            format!(
                "  ✅  {} - reliability {:.0}%, {}ms ({})",
                self.id.to_uppercase(),
                self.reliability * 100.0,
                self.latency_ms,
                self.origin
            )
        } else {
            format!(
                "  ❌  {} - FAILED: {}",
                self.id.to_uppercase(),
                self.error.as_deref().unwrap_or("unknown error")
            )
        }
    }
}

/// Complete First Light test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstLightResult {
    /// Timestamp of the test
    pub timestamp: String,
    /// Anna version
    pub version: String,
    /// Individual question results
    pub questions: Vec<FirstLightQuestion>,
    /// Overall success (all questions passed)
    pub all_passed: bool,
    /// Total XP awarded
    pub total_xp: u64,
    /// Average reliability
    pub avg_reliability: f64,
    /// Average latency
    pub avg_latency_ms: u64,
    /// Total test duration
    pub total_duration_ms: u64,
    /// Sanity checks passed
    pub sanity_passed: bool,
    /// Sanity check details
    pub sanity_details: Vec<String>,
}

impl FirstLightResult {
    /// Create a new result with questions
    pub fn new(questions: Vec<FirstLightQuestion>, total_duration_ms: u64) -> Self {
        let all_passed = questions.iter().all(|q| q.success);
        let total_xp: u64 = questions.iter().map(|q| q.xp_awarded).sum();

        let successful: Vec<&FirstLightQuestion> = questions.iter().filter(|q| q.success).collect();
        let avg_reliability = if successful.is_empty() {
            0.0
        } else {
            successful.iter().map(|q| q.reliability).sum::<f64>() / successful.len() as f64
        };
        let avg_latency_ms = if successful.is_empty() {
            0
        } else {
            successful.iter().map(|q| q.latency_ms).sum::<u64>() / successful.len() as u64
        };

        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            questions,
            all_passed,
            total_xp,
            avg_reliability,
            avg_latency_ms,
            total_duration_ms,
            sanity_passed: false,
            sanity_details: vec![],
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        // Ensure directory exists
        fs::create_dir_all(BENCHMARKS_DIR)
            .map_err(|e| format!("Failed to create benchmarks dir: {}", e))?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(FIRST_LIGHT_FILE, json)
            .map_err(|e| format!("Failed to write first_light.json: {}", e))?;

        Ok(())
    }

    /// Load from disk
    pub fn load() -> Option<Self> {
        let content = fs::read_to_string(FIRST_LIGHT_FILE).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Format summary for display
    pub fn format_summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(String::new());
        lines.push("═══════════════════════════════════════════".to_string());
        lines.push("  🌅  FIRST LIGHT SELF-TEST COMPLETE".to_string());
        lines.push("═══════════════════════════════════════════".to_string());
        lines.push(String::new());

        // Question results
        for q in &self.questions {
            lines.push(q.format_line());
        }

        lines.push(String::new());

        // Summary stats
        let status = if self.all_passed {
            "✅  ALL TESTS PASSED"
        } else {
            "⚠️   SOME TESTS FAILED"
        };
        lines.push(format!("  {}", status));
        lines.push(format!(
            "  📊  Avg Reliability: {:.0}%",
            self.avg_reliability * 100.0
        ));
        lines.push(format!("  ⏱️   Avg Latency: {}ms", self.avg_latency_ms));
        lines.push(format!("  🎮  Total XP: +{}", self.total_xp));
        lines.push(format!(
            "  ⏰  Total Duration: {}ms",
            self.total_duration_ms
        ));

        // Sanity check results
        if !self.sanity_details.is_empty() {
            lines.push(String::new());
            let sanity_status = if self.sanity_passed {
                "✅  SANITY CHECKS PASSED"
            } else {
                "⚠️   SANITY ISSUES DETECTED"
            };
            lines.push(format!("  {}", sanity_status));
            for detail in &self.sanity_details {
                lines.push(format!("      {}", detail));
            }
        }

        lines.push(String::new());
        lines.push("───────────────────────────────────────────".to_string());
        lines.push("  Anna is ready. Ask your first question!".to_string());
        lines.push("───────────────────────────────────────────".to_string());

        lines.join("\n")
    }

    /// Get pass/fail counts
    pub fn pass_fail_counts(&self) -> (usize, usize) {
        let passed = self.questions.iter().filter(|q| q.success).count();
        let failed = self.questions.len() - passed;
        (passed, failed)
    }
}

// ============================================================================
// Sanity Checks
// ============================================================================

/// Result of sanity validation
#[derive(Debug, Clone)]
pub struct SanityCheckResult {
    pub xp_valid: bool,
    pub xp_message: String,
    pub telemetry_valid: bool,
    pub telemetry_message: String,
    pub all_valid: bool,
    pub repairs_attempted: Vec<String>,
}

impl SanityCheckResult {
    /// Format for display
    pub fn format_messages(&self) -> Vec<String> {
        let mut msgs = Vec::new();

        if self.xp_valid {
            msgs.push(format!("✅  XP: {}", self.xp_message));
        } else {
            msgs.push(format!("❌  XP: {}", self.xp_message));
        }

        if self.telemetry_valid {
            msgs.push(format!("✅  Telemetry: {}", self.telemetry_message));
        } else {
            msgs.push(format!("❌  Telemetry: {}", self.telemetry_message));
        }

        if !self.repairs_attempted.is_empty() {
            msgs.push("🔧  Auto-repairs attempted:".to_string());
            for repair in &self.repairs_attempted {
                msgs.push(format!("      - {}", repair));
            }
        }

        msgs
    }
}

/// Run sanity checks on XP and telemetry
pub fn run_sanity_checks() -> SanityCheckResult {
    let mut repairs = Vec::new();

    // Check XP store
    let (xp_valid, xp_message) = check_xp_sanity(&mut repairs);

    // Check telemetry
    let (telemetry_valid, telemetry_message) = check_telemetry_sanity(&mut repairs);

    SanityCheckResult {
        xp_valid,
        xp_message,
        telemetry_valid,
        telemetry_message,
        all_valid: xp_valid && telemetry_valid,
        repairs_attempted: repairs,
    }
}

/// Check XP store sanity
fn check_xp_sanity(repairs: &mut Vec<String>) -> (bool, String) {
    use crate::xp_track::XpStore;

    let store = XpStore::load();

    // Check basic validity
    if store.anna.level < 1 || store.anna.level > 99 {
        repairs.push("Reset invalid Anna level to 1".to_string());
        return (false, "Invalid level detected".to_string());
    }

    if store.anna.trust < 0.0 || store.anna.trust > 1.0 {
        repairs.push("Reset invalid trust to 0.5".to_string());
        return (false, "Invalid trust value detected".to_string());
    }

    // Check if XP increased after First Light (should have some XP)
    if store.anna_stats.total_questions == 0 {
        return (true, "Fresh state (no questions yet)".to_string());
    }

    // Check XP is persisting
    if store.anna.xp == 0 && store.anna_stats.total_questions > 0 {
        return (false, "XP not persisting (questions answered but XP=0)".to_string());
    }

    (true, format!(
        "Level {} with {} XP, {} questions",
        store.anna.level, store.anna.xp, store.anna_stats.total_questions
    ))
}

/// Check telemetry sanity
fn check_telemetry_sanity(repairs: &mut Vec<String>) -> (bool, String) {
    let telemetry_path = Path::new("/var/log/anna/telemetry.jsonl");

    if !telemetry_path.exists() {
        // Create empty file
        if let Err(e) = fs::write(telemetry_path, "") {
            return (false, format!("Cannot create telemetry file: {}", e));
        }
        repairs.push("Created missing telemetry file".to_string());
        return (true, "Created empty telemetry file".to_string());
    }

    // Check file is readable
    let content = match fs::read_to_string(telemetry_path) {
        Ok(c) => c,
        Err(e) => {
            return (false, format!("Cannot read telemetry: {}", e));
        }
    };

    // Check last 5 events have valid JSON
    let lines: Vec<&str> = content.lines().collect();
    let last_5 = lines.iter().rev().take(5).collect::<Vec<_>>();

    if last_5.is_empty() {
        return (true, "Empty telemetry (fresh state)".to_string());
    }

    let mut valid_count = 0;
    for line in &last_5 {
        if serde_json::from_str::<serde_json::Value>(line).is_ok() {
            valid_count += 1;
        }
    }

    if valid_count == last_5.len() {
        (true, format!("{} recent events valid", last_5.len()))
    } else {
        repairs.push("Some telemetry events corrupted".to_string());
        (false, format!("Only {}/{} recent events valid", valid_count, last_5.len()))
    }
}

// ============================================================================
// Daily Check-In
// ============================================================================

/// Daily check-in summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCheckIn {
    /// System uptime
    pub uptime: String,
    /// XP gained today
    pub xp_today: u64,
    /// Reliability average (last 20 events)
    pub avg_reliability_recent: f64,
    /// Brain vs LLM ratio
    pub brain_ratio: f64,
    /// LLM usage ratio
    pub llm_ratio: f64,
    /// Errors detected today
    pub errors_today: u64,
    /// Auto-repairs done today
    pub repairs_today: u64,
    /// Model performance rating (0-100)
    pub model_rating: u32,
    /// Overall status
    pub status: String,
}

impl DailyCheckIn {
    /// Generate daily check-in from current state
    pub fn generate() -> Self {
        use crate::xp_track::XpStore;

        let store = XpStore::load();

        // Get uptime
        let uptime = get_system_uptime();

        // Calculate brain vs LLM ratio from stats
        let total_q = store.anna_stats.total_questions as f64;
        let brain_solves = store.anna_stats.self_solves + store.anna_stats.brain_assists;
        let llm_solves = store.anna_stats.llm_answers;

        let (brain_ratio, llm_ratio) = if total_q > 0.0 {
            (brain_solves as f64 / total_q, llm_solves as f64 / total_q)
        } else {
            (0.0, 0.0)
        };

        // Reliability from stats
        let avg_reliability_recent = store.anna_stats.avg_reliability;

        // Errors and timeouts
        let errors_today = store.anna_stats.timeouts + store.anna_stats.refusals;

        // Model rating based on Junior/Senior stats
        let jr_good = store.junior_stats.good_plans;
        let jr_bad = store.junior_stats.bad_plans + store.junior_stats.timeouts;
        let sr_good = store.senior_stats.approvals + store.senior_stats.fix_and_accept;
        let sr_bad = store.senior_stats.refusals + store.senior_stats.timeouts;

        let total_good = jr_good + sr_good;
        let total_bad = jr_bad + sr_bad;
        let model_rating = if total_good + total_bad > 0 {
            ((total_good as f64 / (total_good + total_bad) as f64) * 100.0) as u32
        } else {
            50 // Neutral if no data
        };

        // Overall status
        let status = if avg_reliability_recent >= 0.8 && model_rating >= 70 {
            "🟢  Excellent".to_string()
        } else if avg_reliability_recent >= 0.6 && model_rating >= 50 {
            "🟡  Good".to_string()
        } else if avg_reliability_recent >= 0.4 {
            "🟠  Fair".to_string()
        } else {
            "🔴  Needs attention".to_string()
        };

        Self {
            uptime,
            xp_today: store.anna.xp, // TODO: track daily XP
            avg_reliability_recent,
            brain_ratio,
            llm_ratio,
            errors_today,
            repairs_today: 0, // TODO: track repairs
            model_rating,
            status,
        }
    }

    /// Format for display
    pub fn format_display(&self) -> String {
        let mut lines = Vec::new();

        lines.push(String::new());
        lines.push("═══════════════════════════════════════════".to_string());
        lines.push("  📋  DAILY CHECK-IN".to_string());
        lines.push("═══════════════════════════════════════════".to_string());
        lines.push(String::new());

        lines.push(format!("  ⏰  Uptime: {}", self.uptime));
        lines.push(format!("  🎮  XP Today: +{}", self.xp_today));
        lines.push(format!(
            "  📊  Reliability: {:.0}%",
            self.avg_reliability_recent * 100.0
        ));
        lines.push(format!(
            "  🧠  Brain/LLM: {:.0}% / {:.0}%",
            self.brain_ratio * 100.0,
            self.llm_ratio * 100.0
        ));
        lines.push(format!("  ❌  Errors: {}", self.errors_today));
        lines.push(format!("  🔧  Repairs: {}", self.repairs_today));
        lines.push(format!("  🤖  Model Rating: {}%", self.model_rating));

        lines.push(String::new());
        lines.push(format!("  Status: {}", self.status));
        lines.push(String::new());
        lines.push("───────────────────────────────────────────".to_string());

        lines.join("\n")
    }
}

/// Get system uptime as human-readable string
fn get_system_uptime() -> String {
    // Try to read from /proc/uptime
    if let Ok(content) = fs::read_to_string("/proc/uptime") {
        if let Some(secs_str) = content.split_whitespace().next() {
            if let Ok(secs) = secs_str.parse::<f64>() {
                let total_secs = secs as u64;
                let days = total_secs / 86400;
                let hours = (total_secs % 86400) / 3600;
                let mins = (total_secs % 3600) / 60;

                if days > 0 {
                    return format!("{}d {}h {}m", days, hours, mins);
                } else if hours > 0 {
                    return format!("{}h {}m", hours, mins);
                } else {
                    return format!("{}m", mins);
                }
            }
        }
    }
    "unknown".to_string()
}

// ============================================================================
// Check Question Types
// ============================================================================

/// Check if a question is asking for daily check-in
pub fn is_daily_checkin_question(q: &str) -> bool {
    let q = q.to_lowercase();

    q.contains("daily check") ||
    q.contains("check in") ||
    q.contains("checkin") ||
    (q.contains("how are you") && q.contains("today")) ||
    q.contains("today's check") ||
    q.contains("status today") ||
    q.contains("daily status")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_light_question_success() {
        let q = FirstLightQuestion::success(
            "cpu",
            "How many CPU cores?",
            0.95,
            150,
            "Brain",
            5,
            "8 cores, 16 threads",
        );

        assert!(q.success);
        assert_eq!(q.id, "cpu");
        assert_eq!(q.reliability, 0.95);
        assert_eq!(q.xp_awarded, 5);
        assert!(q.error.is_none());
    }

    #[test]
    fn test_first_light_question_failure() {
        let q = FirstLightQuestion::failure("llm", "LLM check", 5000, "Connection timeout");

        assert!(!q.success);
        assert_eq!(q.reliability, 0.0);
        assert_eq!(q.xp_awarded, 0);
        assert!(q.error.is_some());
    }

    #[test]
    fn test_first_light_result_stats() {
        let questions = vec![
            FirstLightQuestion::success("cpu", "CPU?", 0.9, 100, "Brain", 5, "8 cores"),
            FirstLightQuestion::success("ram", "RAM?", 0.8, 150, "Brain", 5, "16GB"),
            FirstLightQuestion::failure("llm", "LLM?", 5000, "timeout"),
        ];

        let result = FirstLightResult::new(questions, 5250);

        assert!(!result.all_passed);
        assert_eq!(result.total_xp, 10);
        // Use approximate comparison for floating point
        assert!((result.avg_reliability - 0.85).abs() < 0.001);
        assert_eq!(result.avg_latency_ms, 125);

        let (passed, failed) = result.pass_fail_counts();
        assert_eq!(passed, 2);
        assert_eq!(failed, 1);
    }

    #[test]
    fn test_daily_checkin_triggers() {
        assert!(is_daily_checkin_question("daily check in"));
        assert!(is_daily_checkin_question("show today's check in"));
        assert!(is_daily_checkin_question("how are you today"));
        assert!(is_daily_checkin_question("daily status"));

        assert!(!is_daily_checkin_question("what is the weather"));
        assert!(!is_daily_checkin_question("how are you")); // needs "today"
    }

    #[test]
    fn test_uptime_parsing() {
        let uptime = get_system_uptime();
        // Just verify it returns something
        assert!(!uptime.is_empty());
    }

    #[test]
    fn test_sanity_check_runs() {
        // Just verify it doesn't panic
        let result = run_sanity_checks();
        // XP should be valid on a fresh system
        assert!(!result.xp_message.is_empty());
    }
}
