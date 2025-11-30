//! Router LLM v3.0 - Tiny Question Classifier
//!
//! A fast, small model whose ONLY job is to classify questions into
//! internal types and determine if LLM reasoning is needed.
//!
//! ## Design Goals
//!
//! 1. Fast: Must respond in <1 second (target 500ms)
//! 2. Tiny: Uses qwen2.5:0.5b or similar micro model
//! 3. Focused: Only outputs strict JSON classification
//! 4. Fallback: If router fails/times out, Brain heuristics take over
//!
//! ## Flow
//!
//! ```text
//! Question → Router LLM (500ms budget) → Classification
//!                                      ↓
//!                         { type: "CpuInfo", needs_llm: false, probes: ["cpu.info"] }
//! ```

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{Duration, Instant};

// ============================================================================
// Configuration
// ============================================================================

/// Router model - tiny and fast
pub const ROUTER_MODEL: &str = "qwen2.5:0.5b-instruct";

/// Fallback router model if primary not available
pub const ROUTER_FALLBACK_MODEL: &str = "qwen2.5:1.5b-instruct";

/// Router time budget in milliseconds (500ms target, 1000ms max)
pub const ROUTER_BUDGET_MS: u64 = 1000;

/// Router target latency for good performance
pub const ROUTER_TARGET_MS: u64 = 500;

// ============================================================================
// Question Types (v3.0)
// ============================================================================

/// Question types that Anna recognizes
/// Each type maps to a Brain playbook or LLM routing decision
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    // System Info - Brain can answer directly
    CpuInfo,        // "What CPU?", "How many cores?"
    RamInfo,        // "How much RAM?", "Memory usage?"
    DiskInfo,       // "Disk space?", "How much free?"
    OsInfo,         // "What distro?", "Kernel version?"
    UptimeInfo,     // "How long running?", "Uptime?"
    NetworkInfo,    // "IP address?", "Network interfaces?"
    GpuInfo,        // "What GPU?", "Graphics card?"

    // Self Monitoring - Brain can answer directly
    SelfHealth,     // "Are you ok?", "Health check?"
    SelfLogs,       // "Show logs", "Annad errors?"
    UpdatesCheck,   // "Any updates?", "Pending upgrades?"

    // Control Commands - Brain handles directly
    DebugEnable,    // "Enable debug mode"
    DebugDisable,   // "Disable debug mode"
    DebugStatus,    // "Is debug on?"
    ResetSoft,      // "Reset experience", "Soft reset"
    ResetHard,      // "Factory reset", "Delete everything"

    // Meta Commands - Brain handles directly
    BenchmarkRun,   // "Run benchmark", "Snow leopard test"
    BenchmarkHistory, // "Show benchmark history"
    DailyCheckIn,   // "Daily check in", "How are you?"

    // Complex - Requires LLM reasoning
    ServiceDiagnosis,  // "Why is X slow?", "Debug service Y"
    LogAnalysis,       // "Find errors in logs", "Summarize warnings"
    ConfigHelp,        // "How to configure X?", "Set up Y"
    Comparison,        // "Compare A vs B", "Difference between X and Y"

    // Unknown - Router couldn't classify, use fallback
    Unknown,
}

impl QuestionType {
    /// Does this type require LLM reasoning?
    pub fn needs_llm(&self) -> bool {
        matches!(
            self,
            QuestionType::ServiceDiagnosis
                | QuestionType::LogAnalysis
                | QuestionType::ConfigHelp
                | QuestionType::Comparison
                | QuestionType::Unknown
        )
    }

    /// Is this a Brain-only type (no LLM needed)?
    pub fn is_brain_only(&self) -> bool {
        !self.needs_llm()
    }

    /// Get default probes for this question type
    pub fn default_probes(&self) -> Vec<&'static str> {
        match self {
            QuestionType::CpuInfo => vec!["cpu.info"],
            QuestionType::RamInfo => vec!["mem.info"],
            QuestionType::DiskInfo => vec!["disk.df", "disk.lsblk"],
            QuestionType::OsInfo => vec!["system.os"],
            QuestionType::UptimeInfo => vec!["system.uptime"],
            QuestionType::NetworkInfo => vec!["network.interfaces", "network.routes"],
            QuestionType::GpuInfo => vec!["hardware.gpu"],
            QuestionType::SelfHealth => vec!["self.health"],
            QuestionType::SelfLogs => vec!["logs.annad"],
            QuestionType::UpdatesCheck => vec!["updates.pending"],
            QuestionType::ServiceDiagnosis => vec!["logs.annad", "system.services"],
            QuestionType::LogAnalysis => vec!["logs.annad"],
            _ => vec![],
        }
    }

    /// Get all known types (for router prompt)
    pub fn all_types() -> &'static [QuestionType] {
        &[
            QuestionType::CpuInfo,
            QuestionType::RamInfo,
            QuestionType::DiskInfo,
            QuestionType::OsInfo,
            QuestionType::UptimeInfo,
            QuestionType::NetworkInfo,
            QuestionType::GpuInfo,
            QuestionType::SelfHealth,
            QuestionType::SelfLogs,
            QuestionType::UpdatesCheck,
            QuestionType::DebugEnable,
            QuestionType::DebugDisable,
            QuestionType::DebugStatus,
            QuestionType::ResetSoft,
            QuestionType::ResetHard,
            QuestionType::BenchmarkRun,
            QuestionType::BenchmarkHistory,
            QuestionType::DailyCheckIn,
            QuestionType::ServiceDiagnosis,
            QuestionType::LogAnalysis,
            QuestionType::ConfigHelp,
            QuestionType::Comparison,
            QuestionType::Unknown,
        ]
    }

    /// Parse from string (for JSON deserialization)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cpu_info" | "cpuinfo" => QuestionType::CpuInfo,
            "ram_info" | "raminfo" => QuestionType::RamInfo,
            "disk_info" | "diskinfo" => QuestionType::DiskInfo,
            "os_info" | "osinfo" => QuestionType::OsInfo,
            "uptime_info" | "uptimeinfo" => QuestionType::UptimeInfo,
            "network_info" | "networkinfo" => QuestionType::NetworkInfo,
            "gpu_info" | "gpuinfo" => QuestionType::GpuInfo,
            "self_health" | "selfhealth" => QuestionType::SelfHealth,
            "self_logs" | "selflogs" => QuestionType::SelfLogs,
            "updates_check" | "updatescheck" => QuestionType::UpdatesCheck,
            "debug_enable" | "debugenable" => QuestionType::DebugEnable,
            "debug_disable" | "debugdisable" => QuestionType::DebugDisable,
            "debug_status" | "debugstatus" => QuestionType::DebugStatus,
            "reset_soft" | "resetsoft" => QuestionType::ResetSoft,
            "reset_hard" | "resethard" => QuestionType::ResetHard,
            "benchmark_run" | "benchmarkrun" => QuestionType::BenchmarkRun,
            "benchmark_history" | "benchmarkhistory" => QuestionType::BenchmarkHistory,
            "daily_check_in" | "dailycheckin" => QuestionType::DailyCheckIn,
            "service_diagnosis" | "servicediagnosis" => QuestionType::ServiceDiagnosis,
            "log_analysis" | "loganalysis" => QuestionType::LogAnalysis,
            "config_help" | "confighelp" => QuestionType::ConfigHelp,
            "comparison" => QuestionType::Comparison,
            _ => QuestionType::Unknown,
        }
    }
}

// ============================================================================
// Router Response
// ============================================================================

/// Router classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterClassification {
    /// Classified question type
    #[serde(rename = "type")]
    pub question_type: QuestionType,

    /// Does this require LLM reasoning beyond probes?
    pub needs_llm: bool,

    /// Suggested probes to run (may be empty)
    #[serde(default)]
    pub requires_probes: Vec<String>,

    /// Confidence in classification (0.0-1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f64,

    /// Classification latency in ms
    #[serde(skip)]
    pub latency_ms: u64,

    /// Was this from Brain heuristics (not LLM)?
    #[serde(skip)]
    pub from_brain: bool,
}

fn default_confidence() -> f64 {
    0.5
}

impl RouterClassification {
    /// Create a Brain-derived classification (no LLM used)
    pub fn from_brain(question_type: QuestionType, latency_ms: u64) -> Self {
        let needs_llm = question_type.needs_llm();
        let probes = question_type
            .default_probes()
            .iter()
            .map(|s| s.to_string())
            .collect();

        Self {
            question_type,
            needs_llm,
            requires_probes: probes,
            confidence: 0.95, // Brain heuristics are reliable
            latency_ms,
            from_brain: true,
        }
    }

    /// Create an unknown classification (fallback)
    pub fn unknown(latency_ms: u64) -> Self {
        Self {
            question_type: QuestionType::Unknown,
            needs_llm: true,
            requires_probes: vec![],
            confidence: 0.0,
            latency_ms,
            from_brain: false,
        }
    }
}

// ============================================================================
// Brain Heuristic Classifier (No LLM)
// ============================================================================

/// Classify a question using only Brain heuristics (no LLM)
/// This is the fast path that runs before Router LLM
pub fn classify_with_brain(question: &str) -> Option<RouterClassification> {
    let start = Instant::now();
    let q = question.to_lowercase();
    let q = q.trim();

    // CPU patterns
    if (q.contains("cpu") || q.contains("processor"))
        && (q.contains("what") || q.contains("which") || q.contains("model")
            || q.contains("how many") || q.contains("cores") || q.contains("threads"))
    {
        return Some(RouterClassification::from_brain(
            QuestionType::CpuInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Core/thread questions (standalone - implies CPU)
    if (q.contains("cores") || q.contains("threads"))
        && (q.contains("how many") || q.contains("what") || q.contains("count"))
    {
        return Some(RouterClassification::from_brain(
            QuestionType::CpuInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // RAM patterns
    if (q.contains("ram") || q.contains("memory"))
        && (q.contains("how much") || q.contains("total") || q.contains("available")
            || q.contains("free") || q.contains("installed"))
    {
        return Some(RouterClassification::from_brain(
            QuestionType::RamInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Disk patterns
    if (q.contains("disk") || q.contains("storage") || q.contains("space"))
        && (q.contains("free") || q.contains("available") || q.contains("used")
            || q.contains("root") || q.contains("how much"))
    {
        return Some(RouterClassification::from_brain(
            QuestionType::DiskInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // OS patterns
    if (q.contains("os") || q.contains("distro") || q.contains("linux") || q.contains("kernel"))
        && (q.contains("what") || q.contains("which") || q.contains("version"))
    {
        return Some(RouterClassification::from_brain(
            QuestionType::OsInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Uptime patterns
    if q.contains("uptime") || (q.contains("how long") && (q.contains("running") || q.contains("up"))) {
        return Some(RouterClassification::from_brain(
            QuestionType::UptimeInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Network patterns
    if (q.contains("network") || q.contains("ip") || q.contains("interface"))
        && !q.contains("configure")
    {
        return Some(RouterClassification::from_brain(
            QuestionType::NetworkInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // GPU patterns
    if q.contains("gpu") || q.contains("graphics") || q.contains("video card") {
        return Some(RouterClassification::from_brain(
            QuestionType::GpuInfo,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Health patterns
    if (q.contains("health") || q.contains("diagnose") || q.contains("status"))
        && (q.contains("your") || q.contains("anna") || q.contains("yourself"))
    {
        return Some(RouterClassification::from_brain(
            QuestionType::SelfHealth,
            start.elapsed().as_millis() as u64,
        ));
    }
    if q.contains("are you") && (q.contains("ok") || q.contains("working") || q.contains("alive")) {
        return Some(RouterClassification::from_brain(
            QuestionType::SelfHealth,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Logs patterns
    if q.contains("logs") || q.contains("log ") || (q.contains("annad") && q.contains("error")) {
        return Some(RouterClassification::from_brain(
            QuestionType::SelfLogs,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Updates patterns
    if q.contains("update") && (q.contains("pending") || q.contains("available") || q.contains("check")) {
        return Some(RouterClassification::from_brain(
            QuestionType::UpdatesCheck,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Debug patterns
    if q.contains("debug") {
        if q.contains("enable") || q.contains("turn on") || q.contains("activate") {
            return Some(RouterClassification::from_brain(
                QuestionType::DebugEnable,
                start.elapsed().as_millis() as u64,
            ));
        }
        if q.contains("disable") || q.contains("turn off") || q.contains("deactivate") {
            return Some(RouterClassification::from_brain(
                QuestionType::DebugDisable,
                start.elapsed().as_millis() as u64,
            ));
        }
        if q.contains("status") || q.contains("enabled") || q.contains("on?") {
            return Some(RouterClassification::from_brain(
                QuestionType::DebugStatus,
                start.elapsed().as_millis() as u64,
            ));
        }
    }

    // Reset patterns
    if q.contains("factory reset") || q.contains("hard reset") || q.contains("delete everything") {
        return Some(RouterClassification::from_brain(
            QuestionType::ResetHard,
            start.elapsed().as_millis() as u64,
        ));
    }
    if (q.contains("reset") || q.contains("clear"))
        && (q.contains("experience") || q.contains("xp") || q.contains("soft"))
    {
        return Some(RouterClassification::from_brain(
            QuestionType::ResetSoft,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Benchmark patterns
    if q.contains("benchmark") {
        if q.contains("history") || q.contains("past") || q.contains("previous") {
            return Some(RouterClassification::from_brain(
                QuestionType::BenchmarkHistory,
                start.elapsed().as_millis() as u64,
            ));
        }
        return Some(RouterClassification::from_brain(
            QuestionType::BenchmarkRun,
            start.elapsed().as_millis() as u64,
        ));
    }
    if q.contains("snow leopard") {
        return Some(RouterClassification::from_brain(
            QuestionType::BenchmarkRun,
            start.elapsed().as_millis() as u64,
        ));
    }

    // Daily check-in patterns
    if q.contains("daily") || q.contains("check in") || q.contains("how are you") {
        return Some(RouterClassification::from_brain(
            QuestionType::DailyCheckIn,
            start.elapsed().as_millis() as u64,
        ));
    }

    None // Brain couldn't classify - need Router LLM
}

// ============================================================================
// Router LLM
// ============================================================================

/// Build the router prompt for classification
fn build_router_prompt(question: &str) -> String {
    format!(
        r#"You are a question classifier. Classify this question into ONE type.

QUESTION: "{}"

TYPES:
- cpu_info: CPU model, cores, threads, speed
- ram_info: Memory amount, usage, available
- disk_info: Disk space, storage, filesystem
- os_info: OS version, distro, kernel
- uptime_info: System uptime, how long running
- network_info: IP addresses, interfaces, routes
- gpu_info: Graphics card, GPU
- self_health: Anna's health, status checks
- self_logs: Anna's logs, annad errors
- updates_check: Package updates available
- debug_enable: Turn on debug mode
- debug_disable: Turn off debug mode
- debug_status: Check if debug is on
- reset_soft: Reset experience/XP
- reset_hard: Factory reset
- benchmark_run: Run Snow Leopard benchmark
- benchmark_history: Show past benchmarks
- daily_check_in: Daily status report
- service_diagnosis: Debug a service problem (needs LLM)
- log_analysis: Analyze logs for issues (needs LLM)
- config_help: How to configure something (needs LLM)
- comparison: Compare things (needs LLM)
- unknown: Cannot classify

Respond with ONLY this JSON:
{{"type":"<type>","needs_llm":<true/false>,"requires_probes":["probe1","probe2"]}}

JSON:"#,
        question
    )
}

/// Parse router response JSON
fn parse_router_response(response: &str) -> Option<RouterClassification> {
    let trimmed = response.trim();

    // Try to find JSON in response
    let json_str = if trimmed.starts_with('{') {
        // Direct JSON
        if let Some(end) = trimmed.find('}') {
            &trimmed[..=end]
        } else {
            trimmed
        }
    } else if let Some(start) = trimmed.find('{') {
        // JSON embedded in text
        if let Some(end) = trimmed[start..].find('}') {
            &trimmed[start..=start + end]
        } else {
            return None;
        }
    } else {
        return None;
    };

    // Parse JSON
    #[derive(Deserialize)]
    struct RawResponse {
        #[serde(rename = "type")]
        type_str: String,
        needs_llm: Option<bool>,
        requires_probes: Option<Vec<String>>,
    }

    let raw: RawResponse = serde_json::from_str(json_str).ok()?;
    let question_type = QuestionType::from_str(&raw.type_str);
    let needs_llm = raw.needs_llm.unwrap_or(question_type.needs_llm());
    let probes = raw.requires_probes.unwrap_or_else(|| {
        question_type
            .default_probes()
            .iter()
            .map(|s| s.to_string())
            .collect()
    });

    Some(RouterClassification {
        question_type,
        needs_llm,
        requires_probes: probes,
        confidence: 0.8,
        latency_ms: 0,
        from_brain: false,
    })
}

/// Call the Router LLM for classification
pub fn classify_with_router(question: &str, model: Option<&str>) -> RouterClassification {
    let start = Instant::now();
    let model = model.unwrap_or(ROUTER_MODEL);

    let prompt = build_router_prompt(question);

    // Call Ollama with timeout
    let result = Command::new("timeout")
        .args([
            &format!("{}ms", ROUTER_BUDGET_MS),
            "ollama",
            "run",
            model,
            &prompt,
        ])
        .output();

    let latency = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) if output.status.success() => {
            let response = String::from_utf8_lossy(&output.stdout);
            if let Some(mut classification) = parse_router_response(&response) {
                classification.latency_ms = latency;
                return classification;
            }
        }
        _ => {}
    }

    // Fallback to unknown
    RouterClassification::unknown(latency)
}

/// Classify a question using Brain first, then Router LLM if needed
pub fn classify_question(question: &str, router_model: Option<&str>) -> RouterClassification {
    // Step 1: Try Brain heuristics first (fast, deterministic)
    if let Some(brain_result) = classify_with_brain(question) {
        return brain_result;
    }

    // Step 2: Brain didn't match - use Router LLM
    classify_with_router(question, router_model)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_cpu_classification() {
        let result = classify_with_brain("What CPU do I have?");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::CpuInfo);

        let result = classify_with_brain("How many cores?");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::CpuInfo);
    }

    #[test]
    fn test_brain_ram_classification() {
        let result = classify_with_brain("How much RAM do I have?");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::RamInfo);

        let result = classify_with_brain("Available memory?");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::RamInfo);
    }

    #[test]
    fn test_brain_disk_classification() {
        let result = classify_with_brain("How much free disk space?");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::DiskInfo);
    }

    #[test]
    fn test_brain_health_classification() {
        let result = classify_with_brain("Are you ok?");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::SelfHealth);

        let result = classify_with_brain("Diagnose yourself");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::SelfHealth);
    }

    #[test]
    fn test_brain_debug_classification() {
        let result = classify_with_brain("Enable debug mode");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::DebugEnable);

        let result = classify_with_brain("Turn off debug");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::DebugDisable);
    }

    #[test]
    fn test_brain_benchmark_classification() {
        let result = classify_with_brain("Run benchmark");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::BenchmarkRun);

        let result = classify_with_brain("Show benchmark history");
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::BenchmarkHistory);
    }

    #[test]
    fn test_brain_unknown() {
        let result = classify_with_brain("What's the weather like?");
        assert!(result.is_none());

        let result = classify_with_brain("Tell me a joke");
        assert!(result.is_none());
    }

    #[test]
    fn test_question_type_needs_llm() {
        assert!(!QuestionType::CpuInfo.needs_llm());
        assert!(!QuestionType::RamInfo.needs_llm());
        assert!(!QuestionType::SelfHealth.needs_llm());
        assert!(QuestionType::ServiceDiagnosis.needs_llm());
        assert!(QuestionType::LogAnalysis.needs_llm());
        assert!(QuestionType::Unknown.needs_llm());
    }

    #[test]
    fn test_question_type_default_probes() {
        assert_eq!(QuestionType::CpuInfo.default_probes(), vec!["cpu.info"]);
        assert_eq!(QuestionType::RamInfo.default_probes(), vec!["mem.info"]);
        assert!(!QuestionType::DiskInfo.default_probes().is_empty());
    }

    #[test]
    fn test_parse_router_response() {
        let response = r#"{"type":"cpu_info","needs_llm":false,"requires_probes":["cpu.info"]}"#;
        let result = parse_router_response(response);
        assert!(result.is_some());
        let classification = result.unwrap();
        assert_eq!(classification.question_type, QuestionType::CpuInfo);
        assert!(!classification.needs_llm);
    }

    #[test]
    fn test_parse_router_response_with_text() {
        let response = r#"Here is the classification: {"type":"ram_info","needs_llm":false} done"#;
        let result = parse_router_response(response);
        assert!(result.is_some());
        assert_eq!(result.unwrap().question_type, QuestionType::RamInfo);
    }

    #[test]
    fn test_router_classification_from_brain() {
        let classification = RouterClassification::from_brain(QuestionType::CpuInfo, 5);
        assert!(classification.from_brain);
        assert_eq!(classification.question_type, QuestionType::CpuInfo);
        assert!(!classification.needs_llm);
        assert!(classification.confidence >= 0.9);
    }

    #[test]
    fn test_question_type_from_str() {
        assert_eq!(QuestionType::from_str("cpu_info"), QuestionType::CpuInfo);
        assert_eq!(QuestionType::from_str("ram_info"), QuestionType::RamInfo);
        assert_eq!(QuestionType::from_str("unknown_type"), QuestionType::Unknown);
    }
}
