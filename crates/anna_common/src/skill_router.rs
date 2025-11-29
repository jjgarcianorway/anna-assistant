//! Skill Router v1.6.0 - Generic skill-based routing
//!
//! This module provides the central routing primitive for Anna.
//! All question handling flows through skill classification and dispatch.
//!
//! ## Design Principles
//!
//! 1. **No hardcoded phrases**: Only pattern sets and keywords, never exact sentence matches
//! 2. **Fail fast**: Unsupported skills return quickly with clear messages
//! 3. **Invariants enforced**: Every answer has text, origin, and reliability
//! 4. **Time budgets**: Brain skills <250ms, LLM skills <15s total
//!
//! ## Usage
//!
//! ```ignore
//! let skill = classify_skill(question);
//! let result = handle_skill(skill, &ctx, question).await;
//! // result is guaranteed to have non-empty text and valid reliability
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// SKILL ENUM - The central routing primitive
// ============================================================================

/// High-level capabilities that Anna supports.
///
/// This enum is the ONLY place where supported operations are defined.
/// The classifier maps natural language to skills; handlers execute them.
///
/// IMPORTANT: This enum must NOT contain any natural language phrases.
/// Only the classifier and tests may refer to user phrasings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    // ========================================================================
    // Benchmark Skills
    // ========================================================================
    /// Run the full Snow Leopard benchmark suite
    BenchmarkRunFull,
    /// Run the quick Snow Leopard benchmark
    BenchmarkRunQuick,
    /// Show benchmark history
    BenchmarkHistory,
    /// Compare the last two benchmark runs
    BenchmarkCompareLastTwo,

    // ========================================================================
    // System Information Skills
    // ========================================================================
    /// Get CPU information (model, cores, threads)
    CpuInfo,
    /// Get RAM/memory information
    RamInfo,
    /// Get root disk space information
    RootDiskInfo,
    /// Get system uptime
    UptimeInfo,
    /// Get network summary (interfaces, IPs)
    NetworkSummary,
    /// Get GPU information
    GpuInfo,
    /// Get OS/distro information
    OsInfo,

    // ========================================================================
    // Service & Health Skills
    // ========================================================================
    /// Check Anna's self-health status
    SelfHealth,
    /// Summarize annad service logs
    LogsAnnadSummary,
    /// Check for available updates
    UpdatesPlan,

    // ========================================================================
    // Debug & Control Skills
    // ========================================================================
    /// Enable debug mode
    DebugEnable,
    /// Disable debug mode
    DebugDisable,
    /// Show debug status
    DebugStatus,

    // ========================================================================
    // Experience Management Skills
    // ========================================================================
    /// Reset experience/learning (soft reset)
    ResetExperience,
    /// Factory reset (hard reset)
    ResetFactory,

    // ========================================================================
    // Fallback
    // ========================================================================
    /// Question does not match any known skill
    Unsupported,
}

impl SkillType {
    /// Returns true if this skill can be handled by Brain only (no LLM needed)
    ///
    /// These deterministic categories ALWAYS skip LLM for fast, reliable responses:
    /// - System info: CPU, RAM, disk, uptime, GPU, OS, network
    /// - Service info: health, logs, updates
    /// - Control: debug enable/disable/status
    /// - Meta: benchmark history/compare, reset
    pub fn is_brain_only(&self) -> bool {
        matches!(
            self,
            // System information (deterministic probes)
            SkillType::CpuInfo
                | SkillType::RamInfo
                | SkillType::RootDiskInfo
                | SkillType::UptimeInfo
                | SkillType::NetworkSummary
                | SkillType::GpuInfo
                | SkillType::OsInfo
                // Service & health (deterministic probes)
                | SkillType::SelfHealth
                | SkillType::LogsAnnadSummary
                | SkillType::UpdatesPlan
                // Control commands
                | SkillType::DebugEnable
                | SkillType::DebugDisable
                | SkillType::DebugStatus
                // Meta operations
                | SkillType::BenchmarkHistory
                | SkillType::BenchmarkCompareLastTwo
                | SkillType::ResetExperience
                | SkillType::ResetFactory
                | SkillType::Unsupported
        )
    }

    /// Returns true if this skill requires LLM calls
    ///
    /// NOTE: As of v1.7.0, NO skills require LLM - all are deterministic.
    /// This method exists for forward compatibility if we add LLM skills later.
    pub fn requires_llm(&self) -> bool {
        // All skills are now brain-only (deterministic)
        false
    }

    /// Returns true if this skill runs a long operation (benchmark)
    pub fn is_long_running(&self) -> bool {
        matches!(self, SkillType::BenchmarkRunFull | SkillType::BenchmarkRunQuick)
    }

    /// Get the time budget for this skill
    pub fn time_budget(&self) -> Duration {
        if self.is_brain_only() {
            Duration::from_millis(250)
        } else if self.is_long_running() {
            Duration::from_secs(300) // 5 minutes for benchmarks
        } else {
            Duration::from_secs(15) // LLM budget
        }
    }

    /// Human-readable name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            SkillType::BenchmarkRunFull => "Full Benchmark",
            SkillType::BenchmarkRunQuick => "Quick Benchmark",
            SkillType::BenchmarkHistory => "Benchmark History",
            SkillType::BenchmarkCompareLastTwo => "Benchmark Comparison",
            SkillType::CpuInfo => "CPU Information",
            SkillType::RamInfo => "Memory Information",
            SkillType::RootDiskInfo => "Disk Information",
            SkillType::UptimeInfo => "System Uptime",
            SkillType::NetworkSummary => "Network Summary",
            SkillType::GpuInfo => "GPU Information",
            SkillType::OsInfo => "OS Information",
            SkillType::SelfHealth => "Self Health Check",
            SkillType::LogsAnnadSummary => "Service Logs",
            SkillType::UpdatesPlan => "Update Check",
            SkillType::DebugEnable => "Enable Debug",
            SkillType::DebugDisable => "Disable Debug",
            SkillType::DebugStatus => "Debug Status",
            SkillType::ResetExperience => "Reset Experience",
            SkillType::ResetFactory => "Factory Reset",
            SkillType::Unsupported => "Unsupported",
        }
    }

    /// Get the origin label for answers from this skill
    pub fn origin_label(&self) -> &'static str {
        if self.is_brain_only() || self.is_long_running() {
            "Brain"
        } else {
            "Junior+Senior"
        }
    }
}

// ============================================================================
// ANSWER ORIGIN - Where the answer came from
// ============================================================================

/// Origin of an answer - tracks which component produced it
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerOrigin {
    /// Brain fast path - no LLM needed
    Brain,
    /// Junior LLM only
    Junior,
    /// Junior + Senior LLM
    Senior,
    /// Skill not supported
    Unsupported,
    /// Error/fallback
    Fallback,
}

impl AnswerOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnswerOrigin::Brain => "Brain",
            AnswerOrigin::Junior => "Junior",
            AnswerOrigin::Senior => "Senior",
            AnswerOrigin::Unsupported => "Unsupported",
            AnswerOrigin::Fallback => "Fallback",
        }
    }
}

impl std::fmt::Display for AnswerOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// SKILL ANSWER - Result with enforced invariants
// ============================================================================

/// Result of handling a skill - enforces invariants:
/// - message is never empty
/// - reliability is in [0.0, 1.0]
/// - origin is always set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAnswer {
    /// The skill that was handled
    pub skill: SkillType,
    /// User-visible message (NEVER empty)
    pub message: String,
    /// Where this answer came from
    pub origin: AnswerOrigin,
    /// Reliability score in [0.0, 1.0]
    pub reliability: f64,
    /// How long the skill took to execute
    pub duration_ms: u64,
    /// Whether this was a failure/fallback
    pub is_fallback: bool,
    /// Optional error details (for debugging)
    pub error_details: Option<String>,
}

impl SkillAnswer {
    /// Create a successful answer
    ///
    /// # Panics
    /// Panics if message is empty (invariant violation)
    pub fn success(skill: SkillType, message: impl Into<String>, origin: AnswerOrigin, reliability: f64, duration_ms: u64) -> Self {
        let msg = message.into();
        assert!(!msg.trim().is_empty(), "SkillAnswer message must not be empty");
        Self {
            skill,
            message: msg,
            origin,
            reliability: reliability.clamp(0.0, 1.0),
            duration_ms,
            is_fallback: false,
            error_details: None,
        }
    }

    /// Create a fallback answer (when something went wrong)
    pub fn fallback(skill: SkillType, message: impl Into<String>, origin: AnswerOrigin, reliability: f64, duration_ms: u64, error: impl Into<String>) -> Self {
        let msg = message.into();
        let err = error.into();
        // If message is empty, use the error as message
        let final_message = if msg.trim().is_empty() {
            if err.trim().is_empty() {
                format!("The {} operation could not be completed.", skill.display_name())
            } else {
                err.clone()
            }
        } else {
            msg
        };
        Self {
            skill,
            message: final_message,
            origin,
            // Fallbacks always have at least 0.1 reliability (never completely useless)
            reliability: reliability.clamp(0.1, 1.0),
            duration_ms,
            is_fallback: true,
            error_details: Some(err),
        }
    }

    /// Create an "unsupported" answer
    pub fn unsupported(question: &str, duration_ms: u64) -> Self {
        Self {
            skill: SkillType::Unsupported,
            message: format!(
                "I don't know how to help with that yet.\n\n\
                 Your question: \"{}\"\n\n\
                 I currently support:\n\
                 - System info (CPU, RAM, disk, uptime)\n\
                 - Service health and logs\n\
                 - Snow Leopard benchmarks\n\
                 - Debug mode control\n\n\
                 Try rephrasing or ask about one of these topics.",
                truncate_for_display(question, 80)
            ),
            origin: AnswerOrigin::Unsupported,
            reliability: 0.99, // High confidence that we don't support this
            duration_ms,
            is_fallback: false,
            error_details: None,
        }
    }

    /// Create a timeout answer
    pub fn timeout(skill: SkillType, partial_work: Option<&str>, duration_ms: u64) -> Self {
        let message = if let Some(work) = partial_work {
            format!(
                "I could not safely complete the {} within my time budget.\n\n\
                 What I checked:\n{}\n\n\
                 Please try again or ask a simpler question.",
                skill.display_name(),
                work
            )
        } else {
            format!(
                "I could not complete the {} within my time budget.\n\n\
                 Please try again or ask a simpler question.",
                skill.display_name()
            )
        };
        Self {
            skill,
            message,
            origin: if skill.requires_llm() { AnswerOrigin::Junior } else { AnswerOrigin::Brain },
            reliability: 0.3,
            duration_ms,
            is_fallback: true,
            error_details: Some("timeout".to_string()),
        }
    }
}

// ============================================================================
// SKILL CONTEXT - Runtime context for skill handlers
// ============================================================================

/// Context passed to skill handlers
#[derive(Debug, Clone)]
pub struct SkillContext {
    /// Original question text
    pub question: String,
    /// Time budget for this skill
    pub time_budget: Duration,
    /// Whether debug mode is enabled
    pub debug_enabled: bool,
    /// Optional: time window for log queries (e.g., "3h", "6h", "24h")
    pub time_window: Option<String>,
}

impl SkillContext {
    pub fn new(question: impl Into<String>) -> Self {
        let q = question.into();
        let skill = classify_skill(&q);
        Self {
            question: q,
            time_budget: skill.time_budget(),
            debug_enabled: false,
            time_window: None,
        }
    }

    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug_enabled = enabled;
        self
    }

    pub fn with_time_window(mut self, window: impl Into<String>) -> Self {
        self.time_window = Some(window.into());
        self
    }
}

// ============================================================================
// CLASSIFIER - Pattern-based skill classification
// ============================================================================

/// Classify a question into a Skill using pattern matching.
///
/// This function uses keyword sets and simple patterns, NEVER exact phrase matches.
///
/// # Design
/// - Normalize input (lowercase, trim, collapse whitespace)
/// - Check keyword combinations in priority order
/// - Return Unsupported if no pattern matches
pub fn classify_skill(question: &str) -> SkillType {
    let q = normalize_question(question);

    // Empty or very short questions are unsupported
    if q.len() < 2 {
        return SkillType::Unsupported;
    }

    // ========================================================================
    // Benchmark skills (check first - more specific)
    // ========================================================================
    if is_benchmark_related(&q) {
        // Check compare FIRST (more specific than history)
        if has_compare_intent(&q) {
            return SkillType::BenchmarkCompareLastTwo;
        }
        if has_history_intent(&q) {
            return SkillType::BenchmarkHistory;
        }
        if has_quick_intent(&q) {
            return SkillType::BenchmarkRunQuick;
        }
        // Default: full benchmark if asking to run
        if has_run_intent(&q) {
            return SkillType::BenchmarkRunFull;
        }
        // Just mentioning benchmark without action = history
        return SkillType::BenchmarkHistory;
    }

    // ========================================================================
    // Debug control skills
    // ========================================================================
    if is_debug_related(&q) {
        if has_enable_intent(&q) {
            return SkillType::DebugEnable;
        }
        if has_disable_intent(&q) {
            return SkillType::DebugDisable;
        }
        return SkillType::DebugStatus;
    }

    // ========================================================================
    // Reset/experience skills
    // ========================================================================
    if has_factory_reset_intent(&q) {
        return SkillType::ResetFactory;
    }
    if has_experience_reset_intent(&q) {
        return SkillType::ResetExperience;
    }

    // ========================================================================
    // System info skills (all deterministic, no LLM)
    // ========================================================================
    if is_cpu_related(&q) {
        return SkillType::CpuInfo;
    }
    if is_ram_related(&q) {
        return SkillType::RamInfo;
    }
    if is_disk_related(&q) {
        return SkillType::RootDiskInfo;
    }
    if is_uptime_related(&q) {
        return SkillType::UptimeInfo;
    }
    if is_network_related(&q) {
        return SkillType::NetworkSummary;
    }
    if is_gpu_related(&q) {
        return SkillType::GpuInfo;
    }
    if is_os_related(&q) {
        return SkillType::OsInfo;
    }

    // ========================================================================
    // Service skills (all deterministic, no LLM)
    // ========================================================================
    if is_health_related(&q) {
        return SkillType::SelfHealth;
    }
    if is_logs_related(&q) {
        return SkillType::LogsAnnadSummary;
    }
    if is_updates_related(&q) {
        return SkillType::UpdatesPlan;
    }

    // ========================================================================
    // No match
    // ========================================================================
    SkillType::Unsupported
}

/// Extract time window from question (for log queries)
pub fn extract_time_window(question: &str) -> Option<String> {
    let q = normalize_question(question);

    // Look for patterns like "last 3 hours", "past 6h", "recent 24 hours"
    let patterns = [
        (r"last\s+(\d+)\s*h", "h"),
        (r"past\s+(\d+)\s*h", "h"),
        (r"last\s+(\d+)\s*hour", "h"),
        (r"past\s+(\d+)\s*hour", "h"),
        (r"(\d+)\s*hour", "h"),
        (r"(\d+)\s*h\s", "h"),
    ];

    for (pattern, suffix) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(&q) {
                if let Some(num) = caps.get(1) {
                    return Some(format!("{}{}", num.as_str(), suffix));
                }
            }
        }
    }

    // Default time windows
    if q.contains("recent") {
        return Some("1h".to_string());
    }

    None
}

// ============================================================================
// PATTERN HELPERS - Keyword sets for classification
// ============================================================================

fn normalize_question(q: &str) -> String {
    q.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_benchmark_related(q: &str) -> bool {
    // Must have "benchmark" or "snow leopard"
    (q.contains("benchmark") || q.contains("snow leopard") || q.contains("snowleopard"))
        && !q.contains("how do i") // Exclude help questions
}

fn has_history_intent(q: &str) -> bool {
    q.contains("history") || q.contains("previous")
        || q.contains("list") || q.contains("show all") || q.contains("all runs")
        || (q.contains("have been") && q.contains("run"))
}

fn has_compare_intent(q: &str) -> bool {
    q.contains("compare") || q.contains("diff") || q.contains("delta")
        || q.contains("versus") || q.contains(" vs ") || q.contains("between")
        || (q.contains("last") && q.contains("two"))
        || q.contains("difference")
}

fn has_run_intent(q: &str) -> bool {
    q.contains("run") || q.contains("start") || q.contains("execute")
        || q.contains("perform") || q.contains("do ") || q.contains("begin")
        || q.contains("please") || q.contains("using")  // request patterns
        || q.contains("full")  // "benchmark full" = run full
        || q.contains("now")   // "benchmark now" = run
}

fn has_quick_intent(q: &str) -> bool {
    q.contains("quick") || q.contains("fast") || q.contains("brief")
        || q.contains("short") || q.contains("mini")
}

fn is_debug_related(q: &str) -> bool {
    q.contains("debug") || q.contains("verbose") || q.contains("trace")
}

fn has_enable_intent(q: &str) -> bool {
    q.contains("enable") || q.contains("turn on")
        || (q.contains("activate") && !q.contains("deactivate"))  // avoid substring match
        || q.contains("start")
        || (q.ends_with(" on") || q == "on")  // stricter "on" check
}

fn has_disable_intent(q: &str) -> bool {
    q.contains("disable") || q.contains("turn off") || q.contains("deactivate")
        || (q.contains("stop") && !q.contains("benchmark"))  // "stop debug" but not "stop benchmark"
        || (q.ends_with(" off") || q.ends_with("off"))  // ends with "off"
}

fn has_factory_reset_intent(q: &str) -> bool {
    q.contains("factory") && q.contains("reset")
}

fn has_experience_reset_intent(q: &str) -> bool {
    (q.contains("reset") || q.contains("clear") || q.contains("wipe"))
        && (q.contains("experience") || q.contains("learning") || q.contains("memory") || q.contains("brain"))
        && !q.contains("factory")
}

fn is_cpu_related(q: &str) -> bool {
    q.contains("cpu") || q.contains("processor")
        || (q.contains("core") && (q.contains("how many") || q.contains("count")))
        || q.contains("thread")
}

fn is_ram_related(q: &str) -> bool {
    q.contains("ram") || q.contains("memory")
        || (q.len() <= 5 && q.starts_with("mem")) // short "mem" query
}

fn is_disk_related(q: &str) -> bool {
    q.contains("disk") || q.contains("storage") || q.contains("drive")
        || q.contains("space") || q.contains("partition")
}

fn is_uptime_related(q: &str) -> bool {
    q.contains("uptime") || q.contains("up time")
        || (q.contains("how long") && (q.contains("running") || q.contains("on") || q.contains("up")))
}

fn is_network_related(q: &str) -> bool {
    q.contains("network") || q.contains("ip address") || q.contains("interface")
        || q.contains("ethernet") || q.contains("wifi") || q.contains("connection")
}

fn is_gpu_related(q: &str) -> bool {
    q.contains("gpu") || q.contains("graphics")
        || q.contains("nvidia") || q.contains("amd") || q.contains("radeon")
        || q.contains("intel") && (q.contains("graphics") || q.contains("gpu") || q.contains("video"))
        || q.contains("video card") || q.contains("graphics card")
        || q.contains("cuda") || q.contains("opencl")
}

fn is_os_related(q: &str) -> bool {
    q.contains("os") || q.contains("operating system")
        || q.contains("distro") || q.contains("distribution")
        || q.contains("linux") || q.contains("arch") || q.contains("ubuntu")
        || q.contains("fedora") || q.contains("debian")
        || q.contains("what system") || q.contains("which system")
        || q.contains("version") && (q.contains("system") || q.contains("kernel"))
        || q.contains("uname") || q.contains("kernel")
}

fn is_health_related(q: &str) -> bool {
    // "health check" or "self check" alone is valid
    if q.contains("check") && (q.contains("health") || q.contains("self")) {
        return true;
    }
    // Short "health" query alone is valid
    if q.len() <= 10 && q.contains("health") {
        return true;
    }
    // Otherwise need both condition and target
    (q.contains("health") || q.contains("status") || q.contains("ok") || q.contains("working"))
        && (q.contains("anna") || q.contains("you") || q.contains("self") || q.contains("daemon"))
}

fn is_logs_related(q: &str) -> bool {
    q.contains("log") || q.contains("journal") || q.contains("journalctl")
        || (q.contains("error") && q.contains("show"))
        || (q.contains("what") && q.contains("happen"))
}

fn is_updates_related(q: &str) -> bool {
    q.contains("update") || q.contains("upgrade") || q.contains("patch")
        || q.contains("new version") || q.contains("latest")
}

// ============================================================================
// HELPERS
// ============================================================================

fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Skill enum tests
    // ========================================================================

    #[test]
    fn test_skill_is_brain_only() {
        // All deterministic categories are now brain-only (v1.7.0)
        assert!(SkillType::CpuInfo.is_brain_only());
        assert!(SkillType::RamInfo.is_brain_only());
        assert!(SkillType::GpuInfo.is_brain_only());
        assert!(SkillType::OsInfo.is_brain_only());
        assert!(SkillType::UptimeInfo.is_brain_only());
        assert!(SkillType::NetworkSummary.is_brain_only());
        assert!(SkillType::LogsAnnadSummary.is_brain_only());
        assert!(SkillType::UpdatesPlan.is_brain_only());
        assert!(SkillType::SelfHealth.is_brain_only());
        assert!(SkillType::BenchmarkHistory.is_brain_only());
        // Benchmark runs are NOT brain-only (they're long-running)
        assert!(!SkillType::BenchmarkRunFull.is_brain_only());
    }

    #[test]
    fn test_skill_requires_llm() {
        // As of v1.7.0, NO skills require LLM - all are deterministic
        assert!(!SkillType::LogsAnnadSummary.requires_llm());
        assert!(!SkillType::UpdatesPlan.requires_llm());
        assert!(!SkillType::CpuInfo.requires_llm());
        assert!(!SkillType::BenchmarkRunFull.requires_llm());
        assert!(!SkillType::GpuInfo.requires_llm());
        assert!(!SkillType::OsInfo.requires_llm());
    }

    #[test]
    fn test_skill_time_budgets() {
        // Brain-only skills have short budgets
        assert!(SkillType::CpuInfo.time_budget() < Duration::from_secs(1));
        assert!(SkillType::LogsAnnadSummary.time_budget() < Duration::from_secs(1));
        assert!(SkillType::GpuInfo.time_budget() < Duration::from_secs(1));
        // Long-running skills (benchmarks) have long budgets
        assert!(SkillType::BenchmarkRunFull.time_budget() >= Duration::from_secs(60));
    }

    // ========================================================================
    // Classifier tests - MANY phrasings for each skill
    // ========================================================================

    #[test]
    fn test_classify_benchmark_run_full_many_phrasings() {
        let phrasings = [
            "run the full snow leopard benchmark and show me the summary",
            "please benchmark anna using snow leopard full mode",
            "snow leopard benchmark full run",
            "execute the complete snow leopard benchmark",
            "start a full benchmark run",
            "run snow leopard",
            "do a benchmark",
            "perform the snow leopard test",
            "begin full benchmark",
            "Run the Snow Leopard Benchmark in FULL mode",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::BenchmarkRunFull,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_benchmark_run_quick_many_phrasings() {
        let phrasings = [
            "run the quick snow leopard benchmark",
            "quick benchmark for anna using snow leopard suite",
            "snow leopard quick test",
            "fast benchmark run",
            "do a brief benchmark",
            "run short benchmark",
            "mini snow leopard benchmark",
            "Quick Snow Leopard",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::BenchmarkRunQuick,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_benchmark_history_many_phrasings() {
        let phrasings = [
            "show benchmark history",
            "snow leopard history",
            "list all benchmark runs",
            "show previous benchmarks",
            "past benchmark results",
            "what benchmarks have been run",
            "benchmark list",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::BenchmarkHistory,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_benchmark_compare_many_phrasings() {
        let phrasings = [
            "compare the last two benchmarks",
            "benchmark delta",
            "snow leopard diff",
            "compare benchmark runs",
            "show difference between benchmarks",
            "benchmark versus previous",
            "last two benchmark comparison",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::BenchmarkCompareLastTwo,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_cpu_many_phrasings() {
        let phrasings = [
            "what cpu do i have",
            "show cpu info",
            "how many cores",
            "processor details",
            "what is my cpu",
            "cpu model",
            "how many threads",
            "CPU?",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::CpuInfo,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_ram_many_phrasings() {
        let phrasings = [
            "how much ram",
            "memory info",
            "show memory",
            "ram available",
            "what is my ram",
            "memory usage",
            "RAM?",
            "mem",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::RamInfo,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_disk_many_phrasings() {
        let phrasings = [
            "disk space",
            "storage info",
            "how much space",
            "drive usage",
            "disk free",
            "partition info",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::RootDiskInfo,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_logs_many_phrasings() {
        let phrasings = [
            "show logs",
            "annad logs",
            "journal entries",
            "show errors",
            "what happened in the logs",
            "log summary",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::LogsAnnadSummary,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_health_many_phrasings() {
        let phrasings = [
            "are you ok",
            "health check",
            "anna status",
            "is anna working",
            "self check",
            "daemon health",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::SelfHealth,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_debug_enable_many_phrasings() {
        let phrasings = [
            "enable debug mode",
            "turn on debug",
            "activate debug",
            "start debug",
            "debug on",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::DebugEnable,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_debug_disable_many_phrasings() {
        let phrasings = [
            "disable debug mode",
            "turn off debug",
            "deactivate debug",
            "stop debug",
            "debug off",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::DebugDisable,
                "Failed for: {}",
                phrase
            );
        }
    }

    #[test]
    fn test_classify_unsupported_many_phrasings() {
        let phrasings = [
            "what is the meaning of life",
            "write me a poem",
            "calculate 2+2",
            "hello world",
            "foo bar baz",
            "",
            "x",
        ];
        for phrase in phrasings {
            assert_eq!(
                classify_skill(phrase),
                SkillType::Unsupported,
                "Failed for: {}",
                phrase
            );
        }
    }

    // ========================================================================
    // Time window extraction tests
    // ========================================================================

    #[test]
    fn test_extract_time_window() {
        assert_eq!(extract_time_window("show logs from last 3 hours"), Some("3h".to_string()));
        assert_eq!(extract_time_window("logs past 6 hours"), Some("6h".to_string()));
        assert_eq!(extract_time_window("show recent logs"), Some("1h".to_string()));
        assert_eq!(extract_time_window("show all logs"), None);
    }

    // ========================================================================
    // SkillAnswer invariant tests
    // ========================================================================

    #[test]
    fn test_skill_answer_success() {
        let answer = SkillAnswer::success(
            SkillType::CpuInfo,
            "Your CPU is an AMD Ryzen 7",
            AnswerOrigin::Brain,
            0.95,
            50,
        );
        assert!(!answer.message.is_empty());
        assert!(!answer.is_fallback);
        assert!(answer.reliability >= 0.0 && answer.reliability <= 1.0);
    }

    #[test]
    fn test_skill_answer_fallback_with_error() {
        let answer = SkillAnswer::fallback(
            SkillType::LogsAnnadSummary,
            "",
            AnswerOrigin::Junior,
            0.3,
            5000,
            "LLM timeout",
        );
        // Message should not be empty even if provided empty
        assert!(!answer.message.is_empty());
        assert!(answer.is_fallback);
        assert!(answer.error_details.is_some());
    }

    #[test]
    fn test_skill_answer_unsupported() {
        let answer = SkillAnswer::unsupported("what is love", 10);
        assert!(!answer.message.is_empty());
        assert!(answer.message.contains("what is love"));
        assert_eq!(answer.origin, AnswerOrigin::Unsupported);
    }

    #[test]
    fn test_skill_answer_timeout() {
        let answer = SkillAnswer::timeout(SkillType::LogsAnnadSummary, Some("- Checked journalctl"), 15000);
        assert!(!answer.message.is_empty());
        assert!(answer.message.contains("time budget"));
        assert!(answer.is_fallback);
    }

    #[test]
    #[should_panic(expected = "must not be empty")]
    fn test_skill_answer_success_empty_panics() {
        let _ = SkillAnswer::success(
            SkillType::CpuInfo,
            "",
            AnswerOrigin::Brain,
            0.95,
            50,
        );
    }

    #[test]
    fn test_skill_answer_reliability_clamped() {
        let answer1 = SkillAnswer::success(SkillType::CpuInfo, "test", AnswerOrigin::Brain, 1.5, 10);
        assert_eq!(answer1.reliability, 1.0);

        let answer2 = SkillAnswer::success(SkillType::CpuInfo, "test", AnswerOrigin::Brain, -0.5, 10);
        assert_eq!(answer2.reliability, 0.0);
    }

    // ======================================================================
    // PART 6: NO-HARDCODED-PHRASES REGRESSION TESTS
    // ======================================================================

    /// Test that the classify_skill function uses patterns, not hardcoded full sentences
    /// This is a regression test to verify pattern-based routing
    #[test]
    fn test_no_hardcoded_exact_question_phrases() {
        // Read the classifier source
        let source = include_str!("skill_router.rs");

        // Extract just the classify_skill function body
        let fn_start = source.find("pub fn classify_skill(").unwrap_or(0);
        let fn_end = source[fn_start..]
            .find("pub fn extract_time_window")
            .map(|i| fn_start + i)
            .unwrap_or(source.len());
        let classify_fn = &source[fn_start..fn_end];

        // Full question sentences that should NOT appear in classify_skill
        // The function should use keywords/patterns, not exact matches
        let forbidden_in_classifier = [
            "What CPU do I have",
            "How much RAM is installed",
            "What is my uptime",
            "Show me my disk space",
            "run the snow leopard benchmark",
            "how many cores does my",
            "tell me about my processor",
        ];

        // Check that no exact phrases appear in the classify function
        for phrase in forbidden_in_classifier {
            let lower_fn = classify_fn.to_lowercase();
            let lower_phrase = phrase.to_lowercase();

            assert!(
                !lower_fn.contains(&lower_phrase),
                "Found hardcoded phrase '{}' in classify_skill(). Should use keyword patterns.",
                phrase
            );
        }
    }

    /// Test that multiple very different phrasings all map to the same skill
    /// This proves the classifier is pattern-based, not exact-match
    #[test]
    fn test_phrasing_diversity_cpu() {
        let diverse_phrasings = [
            "cpu",
            "CPU?",
            "what cpu",
            "my processor",
            "processor info",
            "what's my cpu",
            "tell me about my processor",
            "cpuinfo",
            "show cpu details",
            "how many cores",
            "what processor do i have",
            "cpu model",
            "what kind of cpu",
            "cpu specs",
        ];

        for phrasing in diverse_phrasings {
            assert_eq!(
                classify_skill(phrasing),
                SkillType::CpuInfo,
                "Phrasing '{}' should classify as CpuInfo",
                phrasing
            );
        }
    }

    #[test]
    fn test_phrasing_diversity_ram() {
        let diverse_phrasings = [
            "ram",
            "RAM?",
            "memory",
            "how much ram",
            "ram installed",
            "memory info",
            "system memory",
            "how much memory",
            "available memory",
            "ram usage",
            "memory available",
            "mem",
        ];

        for phrasing in diverse_phrasings {
            assert_eq!(
                classify_skill(phrasing),
                SkillType::RamInfo,
                "Phrasing '{}' should classify as RamInfo",
                phrasing
            );
        }
    }

    #[test]
    fn test_phrasing_diversity_benchmark() {
        let full_phrasings = [
            "run benchmark",
            "benchmark full",
            "execute snow leopard",
            "start the benchmark",
            "snow leopard benchmark",
            "full benchmark",
            "benchmark now",
            "run snow leopard full",
        ];

        for phrasing in full_phrasings {
            let skill = classify_skill(phrasing);
            assert!(
                skill == SkillType::BenchmarkRunFull || skill == SkillType::BenchmarkRunQuick,
                "Phrasing '{}' should classify as BenchmarkRunFull/Quick, got {:?}",
                phrasing,
                skill
            );
        }
    }

    /// Test that skills don't rely on question marks or punctuation
    #[test]
    fn test_punctuation_independence() {
        let cases = [
            ("cpu", "cpu?"),
            ("ram", "ram?"),
            ("uptime", "uptime?"),
            ("health check", "health check?"),
            ("what's my cpu", "whats my cpu"),  // apostrophe variation
        ];

        for (a, b) in cases {
            let skill_a = classify_skill(a);
            let skill_b = classify_skill(b);
            assert_eq!(
                skill_a, skill_b,
                "Punctuation should not affect classification: '{}' vs '{}' ({:?} vs {:?})",
                a, b, skill_a, skill_b
            );
        }
    }

    /// Test that classification is case-insensitive
    #[test]
    fn test_case_insensitivity() {
        let cases = [
            "CPU",
            "cpu",
            "Cpu",
            "cPu",
            "RAM",
            "ram",
            "Ram",
            "BENCHMARK",
            "benchmark",
            "Benchmark",
        ];

        // Each pair should match the same skill
        assert_eq!(classify_skill("CPU"), classify_skill("cpu"));
        assert_eq!(classify_skill("RAM"), classify_skill("ram"));
        assert_eq!(classify_skill("BENCHMARK"), classify_skill("benchmark"));
    }

    /// Test that the classifier handles typos gracefully (within reason)
    #[test]
    fn test_typo_tolerance() {
        // These should still classify correctly despite minor variations
        let typo_cases = [
            ("processor", SkillType::CpuInfo),      // alternative for cpu
            ("memory", SkillType::RamInfo),         // alternative for ram
            ("uptime", SkillType::UptimeInfo),
            ("disk", SkillType::RootDiskInfo),
            ("health", SkillType::SelfHealth),
        ];

        for (input, expected) in typo_cases {
            let result = classify_skill(input);
            assert_eq!(
                result, expected,
                "Input '{}' should classify as {:?}, got {:?}",
                input, expected, result
            );
        }
    }

    /// Test that no skill classification relies on word order exclusively
    #[test]
    fn test_word_order_flexibility() {
        // These should all classify the same regardless of word order
        let cpu_variations = [
            "cpu info",
            "info cpu",
            "my cpu",
            "cpu my",
        ];

        for v in cpu_variations {
            assert_eq!(
                classify_skill(v),
                SkillType::CpuInfo,
                "Word order variation '{}' should still be CpuInfo",
                v
            );
        }
    }

    /// Regression test: Ensure common natural language patterns work
    #[test]
    fn test_natural_language_patterns() {
        let natural_patterns = [
            ("what is my cpu", SkillType::CpuInfo),
            ("how much ram do I have", SkillType::RamInfo),
            ("how long has the system been running", SkillType::UptimeInfo),
            ("show me free disk space", SkillType::RootDiskInfo),
            ("is anna healthy", SkillType::SelfHealth),
            ("show benchmark history", SkillType::BenchmarkHistory),
            ("enable debug", SkillType::DebugEnable),
            ("disable debug", SkillType::DebugDisable),
        ];

        for (input, expected) in natural_patterns {
            let result = classify_skill(input);
            assert_eq!(
                result, expected,
                "Natural pattern '{}' should classify as {:?}, got {:?}",
                input, expected, result
            );
        }
    }

    // ========================================================================
    // v1.7.0: GPU and OS classification tests
    // ========================================================================

    #[test]
    fn test_classify_gpu_many_phrasings() {
        let gpu_phrasings = [
            "gpu",
            "GPU?",
            "what gpu",
            "graphics card",
            "video card",
            "nvidia",
            "amd radeon",
            "what graphics",
            "do I have a gpu",
            "gpu info",
            "cuda",
        ];

        for phrasing in gpu_phrasings {
            assert_eq!(
                classify_skill(phrasing),
                SkillType::GpuInfo,
                "Phrasing '{}' should classify as GpuInfo",
                phrasing
            );
        }
    }

    #[test]
    fn test_classify_os_many_phrasings() {
        let os_phrasings = [
            "os",
            "operating system",
            "what distro",
            "which linux",
            "kernel version",
            "what system",
            "uname",
            "arch linux",
            "ubuntu",
            "fedora",
            "distribution",
        ];

        for phrasing in os_phrasings {
            assert_eq!(
                classify_skill(phrasing),
                SkillType::OsInfo,
                "Phrasing '{}' should classify as OsInfo",
                phrasing
            );
        }
    }

    #[test]
    fn test_brain_first_deterministic_categories() {
        // Per spec: these categories should ALWAYS skip LLM
        let deterministic_skills = [
            ("annad logs", SkillType::LogsAnnadSummary),
            ("system updates", SkillType::UpdatesPlan),
            ("uptime", SkillType::UptimeInfo),
            ("gpu info", SkillType::GpuInfo),
            ("health check", SkillType::SelfHealth),
            ("os info", SkillType::OsInfo),
            ("network interfaces", SkillType::NetworkSummary),
            ("disk space", SkillType::RootDiskInfo),
        ];

        for (input, expected) in deterministic_skills {
            let result = classify_skill(input);
            assert_eq!(
                result, expected,
                "Input '{}' should classify as {:?}, got {:?}",
                input, expected, result
            );

            // CRITICAL: All deterministic categories must be brain-only
            assert!(
                result.is_brain_only(),
                "Skill {:?} MUST be brain-only for '{}' per spec",
                result, input
            );
            assert!(
                !result.requires_llm(),
                "Skill {:?} MUST NOT require LLM for '{}' per spec",
                result, input
            );
        }
    }
}
