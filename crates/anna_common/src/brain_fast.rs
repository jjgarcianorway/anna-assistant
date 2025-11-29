//! Brain Fast Path for v0.89.0
//!
//! Direct answer generation for simple questions without LLM calls.
//! Handles: RAM, CPU cores, disk usage, Anna health, debug mode toggle.

use regex::Regex;
use std::process::Command;
use std::time::Instant;

use crate::debug_state::{DebugIntent, DebugState, debug_is_enabled, debug_set_enabled};

/// Time budget for Brain fast path (150ms)
pub const BRAIN_BUDGET_MS: u64 = 150;

/// v0.87.0 time budgets
pub const LLM_A_BUDGET_MS: u64 = 15000;  // Junior: 15 seconds
pub const LLM_B_BUDGET_MS: u64 = 15000;  // Senior: 15 seconds
pub const GLOBAL_SOFT_LIMIT_MS: u64 = 20000;  // 20 second soft target
pub const GLOBAL_HARD_LIMIT_MS: u64 = 30000;  // 30 second hard cutoff

// ============================================================================
// Question Pattern Matching
// ============================================================================

/// Question type that Brain can handle directly
#[derive(Debug, Clone, PartialEq)]
pub enum FastQuestionType {
    /// How much RAM/memory?
    Ram,
    /// How many CPU cores/threads?
    CpuCores,
    /// How much disk space on root?
    RootDiskSpace,
    /// Anna self-health check
    AnnaHealth,
    /// Enable debug mode (v0.89.0)
    DebugEnable,
    /// Disable debug mode (v0.89.0)
    DebugDisable,
    /// Check debug mode status (v0.89.0)
    DebugStatus,
    /// Not a fast-path question
    Unknown,
}

impl FastQuestionType {
    /// Classify a question
    pub fn classify(question: &str) -> Self {
        let q = question.to_lowercase();
        let q = q.trim();

        // RAM patterns
        if (q.contains("ram") || q.contains("memory"))
            && (q.contains("how much") || q.contains("how many") || q.contains("total"))
        {
            return Self::Ram;
        }
        if q.contains("mem") && !q.contains("remember") && q.contains("have") {
            return Self::Ram;
        }

        // CPU cores patterns
        if (q.contains("cpu") || q.contains("core") || q.contains("processor") || q.contains("thread"))
            && (q.contains("how many") || q.contains("how much") || q.contains("number"))
        {
            return Self::CpuCores;
        }
        if q.contains("cores") && (q.contains("my") || q.contains("computer") || q.contains("cpu")) {
            return Self::CpuCores;
        }

        // Root disk patterns
        if (q.contains("disk") || q.contains("space") || q.contains("storage"))
            && (q.contains("root") || q.contains("free") || q.contains("available") || q.contains("/"))
        {
            return Self::RootDiskSpace;
        }
        if q.contains("how much") && q.contains("free") && (q.contains("disk") || q.contains("space")) {
            return Self::RootDiskSpace;
        }

        // Anna health patterns
        if (q.contains("health") || q.contains("diagnose") || q.contains("status"))
            && (q.contains("your") || q.contains("anna") || q.contains("yourself"))
        {
            return Self::AnnaHealth;
        }
        if q.contains("are you") && (q.contains("ok") || q.contains("working") || q.contains("healthy")) {
            return Self::AnnaHealth;
        }

        // Debug mode patterns (v0.89.0) - check before Unknown
        match DebugIntent::classify(question) {
            DebugIntent::Enable => return Self::DebugEnable,
            DebugIntent::Disable => return Self::DebugDisable,
            DebugIntent::Status => return Self::DebugStatus,
            DebugIntent::None => {}
        }

        Self::Unknown
    }
}

// ============================================================================
// Fast Answer Results
// ============================================================================

/// Result of a fast path answer
#[derive(Debug, Clone)]
pub struct FastAnswer {
    /// Human-readable answer text
    pub text: String,
    /// Probes/commands used
    pub citations: Vec<String>,
    /// Reliability score (0.0-1.0)
    pub reliability: f64,
    /// Origin marker
    pub origin: String,
    /// Time taken in ms
    pub duration_ms: u64,
}

impl FastAnswer {
    pub fn new(text: &str, citations: Vec<&str>, reliability: f64) -> Self {
        Self {
            text: text.to_string(),
            citations: citations.into_iter().map(|s| s.to_string()).collect(),
            reliability,
            origin: "Brain".to_string(),
            duration_ms: 0,
        }
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Format as structured output
    pub fn format_answer(&self) -> String {
        format!(
            "Anna\n{}\n\nReliability: {:.0}% ({})\nOrigin: {}\nDuration: {:.2}s",
            self.text,
            self.reliability * 100.0,
            if self.reliability >= 0.9 { "Green" }
            else if self.reliability >= 0.7 { "Yellow" }
            else { "Red" },
            self.origin,
            self.duration_ms as f64 / 1000.0
        )
    }
}

// ============================================================================
// Fast Path Execution
// ============================================================================

/// Try to answer a question using the fast path
pub fn try_fast_answer(question: &str) -> Option<FastAnswer> {
    let start = Instant::now();
    let qtype = FastQuestionType::classify(question);

    let result = match qtype {
        FastQuestionType::Ram => fast_ram_answer(),
        FastQuestionType::CpuCores => fast_cpu_answer(),
        FastQuestionType::RootDiskSpace => fast_disk_answer(),
        FastQuestionType::AnnaHealth => fast_health_answer(),
        FastQuestionType::DebugEnable => fast_debug_enable(),
        FastQuestionType::DebugDisable => fast_debug_disable(),
        FastQuestionType::DebugStatus => fast_debug_status(),
        FastQuestionType::Unknown => return None,
    };

    result.map(|mut ans| {
        ans.duration_ms = start.elapsed().as_millis() as u64;
        ans
    })
}

/// Get RAM info fast
fn fast_ram_answer() -> Option<FastAnswer> {
    // Try /proc/meminfo first (fastest)
    let output = Command::new("cat")
        .arg("/proc/meminfo")
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&output.stdout);

    // Parse MemTotal line: MemTotal:       32554948 kB
    let re = Regex::new(r"MemTotal:\s*(\d+)\s*kB").ok()?;
    let caps = re.captures(&content)?;
    let kb: u64 = caps.get(1)?.as_str().parse().ok()?;

    let gib = kb as f64 / 1024.0 / 1024.0;

    // Also get available memory
    let re_avail = Regex::new(r"MemAvailable:\s*(\d+)\s*kB").ok()?;
    let avail_kb: u64 = if let Some(caps) = re_avail.captures(&content) {
        caps.get(1)?.as_str().parse().ok()?
    } else {
        0
    };
    let avail_gib = avail_kb as f64 / 1024.0 / 1024.0;

    let text = if avail_kb > 0 {
        format!(
            "You have {:.1} GiB of RAM installed, with {:.1} GiB currently available.",
            gib, avail_gib
        )
    } else {
        format!("You have about {:.1} GiB of RAM installed.", gib)
    };

    Some(FastAnswer::new(&text, vec!["cat /proc/meminfo"], 0.99))
}

/// Get CPU info fast
fn fast_cpu_answer() -> Option<FastAnswer> {
    let output = Command::new("lscpu")
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&output.stdout);

    // Extract: CPU(s), Core(s) per socket, Socket(s) - use multiline mode
    let re_cpus = Regex::new(r"(?m)^CPU\(s\):\s*(\d+)").ok()?;
    let re_cores_per = Regex::new(r"(?m)Core\(s\) per socket:\s*(\d+)").ok()?;
    let re_sockets = Regex::new(r"(?m)Socket\(s\):\s*(\d+)").ok()?;
    let re_model = Regex::new(r"(?m)Model name:\s*(.+)").ok()?;

    let cpus: u32 = re_cpus.captures(&content)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    let cores_per: u32 = re_cores_per.captures(&content)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    let sockets: u32 = re_sockets.captures(&content)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(1);

    let model: String = re_model.captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    if cpus == 0 {
        return None;
    }

    let physical_cores = cores_per * sockets;
    let threads = cpus;

    let text = format!(
        "Your CPU ({}) has {} physical cores and {} threads (logical CPUs).",
        model, physical_cores, threads
    );

    Some(FastAnswer::new(&text, vec!["lscpu"], 0.99))
}

/// Get disk info fast
fn fast_disk_answer() -> Option<FastAnswer> {
    let output = Command::new("df")
        .args(["-h", "/"])
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < 2 {
        return None;
    }

    // Parse: Filesystem  Size  Used  Avail  Use%  Mounted on
    // Example: /dev/sda2   234G   67G   155G   31%  /
    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let size = parts[1];
    let used = parts[2];
    let avail = parts[3];
    let use_pct = parts[4];

    let text = format!(
        "Your root filesystem has {} total, {} used, and {} available ({} used).",
        size, used, avail, use_pct
    );

    Some(FastAnswer::new(&text, vec!["df -h /"], 0.99))
}

/// Check Anna health fast
fn fast_health_answer() -> Option<FastAnswer> {
    // Check if annad is running
    let annad_running = Command::new("pgrep")
        .args(["-f", "annad"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if ollama is running
    let ollama_running = Command::new("pgrep")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check daemon port
    let port_open = Command::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "http://127.0.0.1:8080/health"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("200"))
        .unwrap_or(false);

    let mut status_parts = Vec::new();
    let mut all_ok = true;

    if annad_running {
        status_parts.push("annad daemon is running");
    } else {
        status_parts.push("annad daemon is NOT running");
        all_ok = false;
    }

    if ollama_running {
        status_parts.push("Ollama LLM service is running");
    } else {
        status_parts.push("Ollama LLM service is NOT running");
        all_ok = false;
    }

    if port_open {
        status_parts.push("API endpoint is responding");
    } else {
        status_parts.push("API endpoint is not responding");
        // Not critical if annad is running
    }

    let summary = if all_ok {
        "I'm healthy and operational."
    } else if annad_running {
        "I'm partially operational with some issues."
    } else {
        "I'm experiencing issues."
    };

    let text = format!(
        "{}\n\nStatus:\n- {}",
        summary,
        status_parts.join("\n- ")
    );

    let reliability = if all_ok { 0.99 } else if annad_running { 0.85 } else { 0.70 };

    Some(FastAnswer::new(&text, vec!["pgrep annad", "pgrep ollama", "curl /health"], reliability))
}

// ============================================================================
// Debug Mode Handlers (v0.89.0)
// ============================================================================

/// Enable debug mode
fn fast_debug_enable() -> Option<FastAnswer> {
    match debug_set_enabled(true, "user_command") {
        Ok(()) => Some(FastAnswer::new(
            &DebugState::format_enable_message(),
            vec!["debug_state.json"],
            0.99,
        )),
        Err(_) => Some(FastAnswer::new(
            "Failed to enable debug mode. Check file permissions.",
            vec!["debug_state.json"],
            0.50,
        )),
    }
}

/// Disable debug mode
fn fast_debug_disable() -> Option<FastAnswer> {
    match debug_set_enabled(false, "user_command") {
        Ok(()) => Some(FastAnswer::new(
            &DebugState::format_disable_message(),
            vec!["debug_state.json"],
            0.99,
        )),
        Err(_) => Some(FastAnswer::new(
            "Failed to disable debug mode. Check file permissions.",
            vec!["debug_state.json"],
            0.50,
        )),
    }
}

/// Check debug mode status
fn fast_debug_status() -> Option<FastAnswer> {
    let state = DebugState::load();
    Some(FastAnswer::new(
        &state.format_status(),
        vec!["debug_state.json"],
        0.99,
    ))
}

// ============================================================================
// Timing Summary
// ============================================================================

/// Timing summary for debug output
#[derive(Debug, Clone, Default)]
pub struct TimingSummary {
    pub brain_ms: u64,
    pub junior_calls: u32,
    pub junior_ms: u64,
    pub senior_calls: u32,
    pub senior_ms: u64,
    pub command_ms: u64,
    pub total_ms: u64,
}

impl TimingSummary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn format(&self) -> String {
        format!(
            "[TIMING] brain={}ms junior_calls={} ({}ms) senior_calls={} ({}ms) cmd={}ms total={}ms",
            self.brain_ms,
            self.junior_calls,
            self.junior_ms,
            self.senior_calls,
            self.senior_ms,
            self.command_ms,
            self.total_ms
        )
    }
}

// ============================================================================
// Fallback Answer
// ============================================================================

/// Create a fallback answer when everything fails
pub fn create_fallback_answer(question: &str, evidence: Option<&str>, error: &str) -> FastAnswer {
    let text = if let Some(ev) = evidence {
        format!(
            "I could not answer this question reliably because {}.\n\nEvidence I collected:\n{}",
            error,
            ev.chars().take(500).collect::<String>()
        )
    } else {
        format!(
            "I could not answer this question reliably because {}.",
            error
        )
    };

    FastAnswer {
        text,
        citations: vec![],
        reliability: 0.0,
        origin: "Fallback".to_string(),
        duration_ms: 0,
    }
}

/// Create a partial answer from available evidence
pub fn create_partial_answer(question: &str, evidence: &str, probe_used: &str) -> Option<FastAnswer> {
    let qtype = FastQuestionType::classify(question);

    match qtype {
        FastQuestionType::Ram => parse_ram_from_evidence(evidence),
        FastQuestionType::CpuCores => parse_cpu_from_evidence(evidence),
        FastQuestionType::RootDiskSpace => parse_disk_from_evidence(evidence),
        _ => None,
    }.map(|mut ans| {
        ans.reliability = 0.85; // Lower than direct, since we're parsing fallback
        ans.origin = "Brain-Fallback".to_string();
        ans
    })
}

fn parse_ram_from_evidence(evidence: &str) -> Option<FastAnswer> {
    let re = Regex::new(r"MemTotal:\s*(\d+)\s*kB").ok()?;
    let caps = re.captures(evidence)?;
    let kb: u64 = caps.get(1)?.as_str().parse().ok()?;
    let gib = kb as f64 / 1024.0 / 1024.0;

    Some(FastAnswer::new(
        &format!("You have about {:.1} GiB of RAM installed.", gib),
        vec!["evidence"],
        0.85,
    ))
}

fn parse_cpu_from_evidence(evidence: &str) -> Option<FastAnswer> {
    let re_cpus = Regex::new(r"CPU\(s\):\s*(\d+)").ok()?;
    let cpus: u32 = re_cpus.captures(evidence)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())?;

    Some(FastAnswer::new(
        &format!("Your CPU has {} logical CPUs (threads).", cpus),
        vec!["evidence"],
        0.85,
    ))
}

fn parse_disk_from_evidence(evidence: &str) -> Option<FastAnswer> {
    // Try to find df output pattern
    let re = Regex::new(r"(\d+%)\s+/$").ok()?;
    if let Some(caps) = re.captures(evidence) {
        let pct = caps.get(1)?.as_str();
        return Some(FastAnswer::new(
            &format!("Your root filesystem is {} used.", pct),
            vec!["evidence"],
            0.85,
        ));
    }
    None
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_ram() {
        assert_eq!(FastQuestionType::classify("how much ram do i have?"), FastQuestionType::Ram);
        assert_eq!(FastQuestionType::classify("How much memory is installed?"), FastQuestionType::Ram);
        assert_eq!(FastQuestionType::classify("total RAM?"), FastQuestionType::Ram);
    }

    #[test]
    fn test_classify_cpu() {
        assert_eq!(FastQuestionType::classify("how many cpu cores?"), FastQuestionType::CpuCores);
        assert_eq!(FastQuestionType::classify("how many cores does my computer have"), FastQuestionType::CpuCores);
        assert_eq!(FastQuestionType::classify("number of threads?"), FastQuestionType::CpuCores);
    }

    #[test]
    fn test_classify_disk() {
        assert_eq!(FastQuestionType::classify("how much free disk space on root?"), FastQuestionType::RootDiskSpace);
        assert_eq!(FastQuestionType::classify("disk space available?"), FastQuestionType::RootDiskSpace);
    }

    #[test]
    fn test_classify_health() {
        assert_eq!(FastQuestionType::classify("diagnose yourself"), FastQuestionType::AnnaHealth);
        assert_eq!(FastQuestionType::classify("what is your health?"), FastQuestionType::AnnaHealth);
        assert_eq!(FastQuestionType::classify("are you ok?"), FastQuestionType::AnnaHealth);
    }

    #[test]
    fn test_classify_unknown() {
        assert_eq!(FastQuestionType::classify("what is the weather?"), FastQuestionType::Unknown);
        assert_eq!(FastQuestionType::classify("tell me a joke"), FastQuestionType::Unknown);
    }

    #[test]
    fn test_classify_debug_enable() {
        assert_eq!(FastQuestionType::classify("enable debug mode"), FastQuestionType::DebugEnable);
        assert_eq!(FastQuestionType::classify("turn debug mode on"), FastQuestionType::DebugEnable);
        assert_eq!(FastQuestionType::classify("activate debug"), FastQuestionType::DebugEnable);
        assert_eq!(FastQuestionType::classify("start debug mode"), FastQuestionType::DebugEnable);
    }

    #[test]
    fn test_classify_debug_disable() {
        assert_eq!(FastQuestionType::classify("disable debug mode"), FastQuestionType::DebugDisable);
        assert_eq!(FastQuestionType::classify("turn debug mode off"), FastQuestionType::DebugDisable);
        assert_eq!(FastQuestionType::classify("deactivate debug"), FastQuestionType::DebugDisable);
        assert_eq!(FastQuestionType::classify("stop debug mode"), FastQuestionType::DebugDisable);
    }

    #[test]
    fn test_classify_debug_status() {
        assert_eq!(FastQuestionType::classify("is debug mode enabled?"), FastQuestionType::DebugStatus);
        assert_eq!(FastQuestionType::classify("is debug on?"), FastQuestionType::DebugStatus);
        assert_eq!(FastQuestionType::classify("debug status"), FastQuestionType::DebugStatus);
        assert_eq!(FastQuestionType::classify("what is your debug mode state?"), FastQuestionType::DebugStatus);
    }

    #[test]
    fn test_parse_ram_evidence() {
        let evidence = "MemTotal:       32554948 kB\nMemFree: 1234 kB";
        let ans = parse_ram_from_evidence(evidence).unwrap();
        assert!(ans.text.contains("31."));
        assert!(ans.text.contains("GiB"));
    }

    #[test]
    fn test_parse_cpu_evidence() {
        let evidence = "CPU(s):                  32\nCore(s) per socket: 16";
        let ans = parse_cpu_from_evidence(evidence).unwrap();
        assert!(ans.text.contains("32"));
    }

    #[test]
    fn test_timing_summary_format() {
        let ts = TimingSummary {
            brain_ms: 10,
            junior_calls: 1,
            junior_ms: 5000,
            senior_calls: 1,
            senior_ms: 3000,
            command_ms: 100,
            total_ms: 8110,
        };
        let s = ts.format();
        assert!(s.contains("brain=10ms"));
        assert!(s.contains("total=8110ms"));
    }

    #[test]
    fn test_fallback_answer() {
        let ans = create_fallback_answer("test?", Some("evidence here"), "the LLM failed");
        assert!(ans.text.contains("could not answer"));
        assert!(ans.text.contains("LLM failed"));
        assert_eq!(ans.reliability, 0.0);
    }
}
