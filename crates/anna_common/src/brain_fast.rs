//! Brain Fast Path v3.12.0
//!
//! Zero-LLM fast path for simple questions. See `docs/architecture.md` Section 3.
//!
//! ## Supported Question Types
//!
//! | Type | Latency | Command |
//! |------|---------|---------|
//! | RAM | <50ms | `cat /proc/meminfo` |
//! | CPU | <50ms | `lscpu` (cached 60s) |
//! | Disk | <50ms | `df -h /` |
//! | OS/Kernel | <50ms | `uname`, `/etc/os-release` |
//! | Uptime | <50ms | `uptime -p` |
//! | GPU | <100ms | `lspci`, `nvidia-smi` |
//! | Network | <100ms | `ip addr show` |
//! | Health | <100ms | `pgrep`, `curl` |
//! | Debug | <10ms | State file |
//! | Reset | <50ms | Experience/Factory reset |
//!
//! ## v1.3.0 Changes
//!
//! - Two reset modes: Experience (soft) and Factory (hard)
//! - Experience reset: XP to baseline, clear telemetry/stats, keep knowledge
//! - Factory reset: Delete everything including knowledge (requires strong confirmation)
//!
//! ## v1.1.0 Improvements
//!
//! - Micro-caching for stable facts (CPU model, total RAM) with 60s TTL
//! - Expanded pattern matching for natural question variations
//! - CPU model query support

use regex::Regex;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::debug_state::{DebugIntent, DebugState, debug_is_enabled, debug_set_enabled};

// ============================================================================
// Micro-Cache for Stable Facts (v1.1.0)
// ============================================================================

/// Cache TTL for stable facts (60 seconds)
const CACHE_TTL_SECS: u64 = 60;

/// Cached CPU info
static CPU_CACHE: Mutex<Option<(String, Instant)>> = Mutex::new(None);

/// Cached RAM total (doesn't change without reboot)
static RAM_TOTAL_CACHE: Mutex<Option<(u64, Instant)>> = Mutex::new(None);

/// Get cached CPU info or fetch fresh
fn get_cached_cpu_info() -> Option<String> {
    // Check cache first
    if let Ok(guard) = CPU_CACHE.lock() {
        if let Some((ref data, ref cached_at)) = *guard {
            if cached_at.elapsed() < Duration::from_secs(CACHE_TTL_SECS) {
                return Some(data.clone());
            }
        }
    }

    // Fetch fresh
    let output = Command::new("lscpu").output().ok()?;
    let content = String::from_utf8_lossy(&output.stdout).to_string();

    // Update cache
    if let Ok(mut guard) = CPU_CACHE.lock() {
        *guard = Some((content.clone(), Instant::now()));
    }

    Some(content)
}

/// Get cached RAM total or fetch fresh
fn get_cached_ram_total() -> Option<u64> {
    // Check cache first
    if let Ok(guard) = RAM_TOTAL_CACHE.lock() {
        if let Some((kb, ref cached_at)) = *guard {
            if cached_at.elapsed() < Duration::from_secs(CACHE_TTL_SECS) {
                return Some(kb);
            }
        }
    }

    // Fetch fresh
    let output = Command::new("cat").arg("/proc/meminfo").output().ok()?;
    let content = String::from_utf8_lossy(&output.stdout);

    let re = Regex::new(r"MemTotal:\s*(\d+)\s*kB").ok()?;
    let caps = re.captures(&content)?;
    let kb: u64 = caps.get(1)?.as_str().parse().ok()?;

    // Update cache
    if let Ok(mut guard) = RAM_TOTAL_CACHE.lock() {
        *guard = Some((kb, Instant::now()));
    }

    Some(kb)
}

/// Time budget for Brain fast path (150ms)
pub const BRAIN_BUDGET_MS: u64 = 150;

/// v3.13.0: Realistic time budgets for various LLM sizes
/// Small models (1.5b-3b): 4-6s, Medium (4b-7b): 8-10s, Large (14b+): 10-15s
pub const LLM_A_BUDGET_MS: u64 = 10000;   // Junior: 10 seconds (realistic for 4b+ models)
pub const LLM_B_BUDGET_MS: u64 = 12000;   // Senior: 12 seconds (realistic for 14b models)
pub const GLOBAL_SOFT_LIMIT_MS: u64 = 15000;   // 15 second soft target
pub const GLOBAL_HARD_LIMIT_MS: u64 = 20000;   // 20 second hard cutoff

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
    /// What CPU model? (v1.1.0)
    CpuModel,
    /// How much disk space on root?
    RootDiskSpace,
    /// What OS/distro/kernel? (v3.10.0)
    OsInfo,
    /// System uptime (v3.10.0)
    Uptime,
    /// Anna self-health check
    AnnaHealth,
    /// Enable debug mode (v0.89.0)
    DebugEnable,
    /// Disable debug mode (v0.89.0)
    DebugDisable,
    /// Check debug mode status (v0.89.0)
    DebugStatus,
    /// Reset experience/memory - soft reset (v1.3.0)
    ResetExperience,
    /// Factory reset - hard reset, deletes everything (v1.3.0)
    ResetFactory,
    /// Run Snow Leopard benchmark - full (v1.4.0)
    BenchmarkFull,
    /// Run Snow Leopard benchmark - quick (v1.4.0)
    BenchmarkQuick,
    /// Show benchmark history (v1.5.0)
    BenchmarkHistory,
    /// Compare benchmarks / show delta (v1.5.0)
    BenchmarkDelta,
    /// Daily check-in (v2.2.0)
    DailyCheckIn,
    /// GPU information (v3.12.0)
    GpuInfo,
    /// Network interfaces (v3.12.0)
    NetworkInfo,
    /// v3.13.0: Factory reset confirmation (executes reset immediately)
    ResetFactoryConfirm,
    /// v3.13.0: Experience reset confirmation (executes reset immediately)
    ResetExperienceConfirm,
    /// v3.13.1: First Light self-test (Brain-only, no LLM)
    FirstLight,
    /// Not a fast-path question
    Unknown,
}

impl FastQuestionType {
    /// Classify a question (v1.1.0 - expanded patterns)
    pub fn classify(question: &str) -> Self {
        let q = question.to_lowercase();
        let q = q.trim();

        // =================================================================
        // RAM patterns (expanded v1.1.0)
        // =================================================================
        // Direct RAM questions
        if (q.contains("ram") || q.contains("memory"))
            && (q.contains("how much") || q.contains("how many") || q.contains("total")
                || q.contains("installed") || q.contains("have") || q.contains("got"))
        {
            return Self::Ram;
        }
        // "what's my ram" / "my ram?" / "check ram"
        if q.contains("ram") && (q.contains("my") || q.contains("check") || q.starts_with("ram")) {
            return Self::Ram;
        }
        // "how much mem do i have" (but not "remember")
        if q.contains("mem") && !q.contains("remember") && (q.contains("have") || q.contains("got")) {
            return Self::Ram;
        }
        // Short forms: "ram?", "memory?"
        if q == "ram?" || q == "ram" || q == "memory?" || q == "memory" {
            return Self::Ram;
        }

        // =================================================================
        // OS/Kernel patterns (v3.10.0 - NEW)
        // =================================================================
        // "what os" / "which os" / "what operating system"
        if (q.contains("what") || q.contains("which"))
            && (q.contains(" os") || q.contains("os ") || q.contains("operating system"))
        {
            return Self::OsInfo;
        }
        // "what distro" / "which distro" / "what distribution" / "linux distribution"
        if (q.contains("distro") || q.contains("distribution"))
            && (q.contains("what") || q.contains("which") || q.contains("linux") || q.contains("my"))
        {
            return Self::OsInfo;
        }
        // "what kernel" / "kernel version" / "which kernel"
        if q.contains("kernel")
            && (q.contains("what") || q.contains("which") || q.contains("version") || q.contains("running"))
        {
            return Self::OsInfo;
        }
        // Combined: "what os and kernel" / "os and kernel version"
        if q.contains("os") && q.contains("kernel") {
            return Self::OsInfo;
        }
        // "uname" direct command
        if q.trim() == "uname" || q.contains("uname -") {
            return Self::OsInfo;
        }
        // Short forms
        if q == "os?" || q == "kernel?" || q == "distro?" {
            return Self::OsInfo;
        }

        // =================================================================
        // Uptime patterns (v3.10.0 - NEW)
        // =================================================================
        if q.contains("uptime") || q.contains("how long")
            && (q.contains("running") || q.contains("been up") || q.contains("on for"))
        {
            return Self::Uptime;
        }
        if q == "uptime?" || q == "uptime" {
            return Self::Uptime;
        }

        // =================================================================
        // CPU Model patterns (v1.1.0 - NEW)
        // =================================================================
        // "what cpu" / "which cpu" / "cpu model" / "what processor"
        if (q.contains("what") || q.contains("which"))
            && (q.contains("cpu") || q.contains("processor"))
            && !q.contains("cores") && !q.contains("threads") && !q.contains("many")
        {
            return Self::CpuModel;
        }
        // "cpu model" / "processor model" / "my cpu" / "my processor"
        if (q.contains("cpu") || q.contains("processor"))
            && (q.contains("model") || q.contains("name") || q.contains("type"))
        {
            return Self::CpuModel;
        }
        // "what chip" / "which chip"
        if (q.contains("what") || q.contains("which")) && q.contains("chip") {
            return Self::CpuModel;
        }

        // =================================================================
        // CPU Cores patterns (expanded v1.1.0)
        // =================================================================
        // Direct core/thread questions
        if (q.contains("cpu") || q.contains("core") || q.contains("processor") || q.contains("thread"))
            && (q.contains("how many") || q.contains("how much") || q.contains("number")
                || q.contains("count") || q.contains("total"))
        {
            return Self::CpuCores;
        }
        // "cores?" / "threads?" / "cpu cores" / "my cores"
        if q.contains("cores") && (q.contains("my") || q.contains("computer") || q.contains("cpu")
            || q.contains("have") || q.contains("got") || q.starts_with("cores"))
        {
            return Self::CpuCores;
        }
        if q.contains("threads") && (q.contains("my") || q.contains("cpu") || q.contains("have")) {
            return Self::CpuCores;
        }
        // Short forms
        if q == "cores?" || q == "cores" || q == "threads?" || q == "threads" {
            return Self::CpuCores;
        }

        // =================================================================
        // Root disk patterns (expanded v1.1.0)
        // =================================================================
        if (q.contains("disk") || q.contains("space") || q.contains("storage") || q.contains("filesystem"))
            && (q.contains("root") || q.contains("free") || q.contains("available")
                || q.contains("/") || q.contains("left") || q.contains("remaining"))
        {
            return Self::RootDiskSpace;
        }
        if q.contains("how much") && (q.contains("free") || q.contains("space") || q.contains("disk")) {
            return Self::RootDiskSpace;
        }
        // "df" / "disk usage" / "storage?"
        if q == "df" || q.contains("disk usage") || q == "storage?" || q == "storage" {
            return Self::RootDiskSpace;
        }

        // =================================================================
        // Anna health patterns (expanded v1.1.0)
        // =================================================================
        if (q.contains("health") || q.contains("diagnose") || q.contains("status"))
            && (q.contains("your") || q.contains("anna") || q.contains("yourself") || q.contains("self"))
        {
            return Self::AnnaHealth;
        }
        if q.contains("are you") && (q.contains("ok") || q.contains("working") || q.contains("healthy")
            || q.contains("alive") || q.contains("running"))
        {
            return Self::AnnaHealth;
        }
        // "check yourself" / "self check" / "health check"
        if (q.contains("check") && (q.contains("yourself") || q.contains("self")))
            || q.contains("health check")
        {
            return Self::AnnaHealth;
        }

        // =================================================================
        // Debug mode patterns (v0.89.0)
        // =================================================================
        match DebugIntent::classify(question) {
            DebugIntent::Enable => return Self::DebugEnable,
            DebugIntent::Disable => return Self::DebugDisable,
            DebugIntent::Status => return Self::DebugStatus,
            DebugIntent::None => {}
        }

        // =================================================================
        // Daily Check-In (v2.2.0)
        // =================================================================
        if crate::first_light::is_daily_checkin_question(q) {
            return Self::DailyCheckIn;
        }

        // =================================================================
        // First Light self-test (v3.13.1 - Brain-only, no LLM)
        // =================================================================
        if crate::first_light::is_first_light_question(q) {
            return Self::FirstLight;
        }

        // =================================================================
        // Reset patterns (v1.3.0 - Experience vs Factory)
        // v3.13.0: Check confirmations FIRST (before new reset requests)
        // =================================================================
        // v3.13.0: Confirmation patterns - execute reset immediately
        if is_factory_reset_confirmation(q) {
            return Self::ResetFactoryConfirm;
        }
        if is_confirmation(q) {
            // Bare "yes" / "y" confirms soft reset
            return Self::ResetExperienceConfirm;
        }
        // New reset requests (not confirmations)
        if is_factory_reset_question(q) {
            return Self::ResetFactory;
        }
        if is_reset_experience_question(q) {
            return Self::ResetExperience;
        }

        // =================================================================
        // Benchmark patterns (v1.4.0 - Snow Leopard)
        // =================================================================
        // v1.5.0: Check delta FIRST (more specific than history)
        // "show benchmark delta" should be delta, not history
        if is_benchmark_delta_question(q) {
            return Self::BenchmarkDelta;
        }
        if is_benchmark_history_question(q) {
            return Self::BenchmarkHistory;
        }
        // Run benchmark
        if is_benchmark_question(q) {
            if is_quick_benchmark(q) {
                return Self::BenchmarkQuick;
            }
            return Self::BenchmarkFull;
        }

        // =================================================================
        // GPU patterns (v3.12.0 - NEW)
        // =================================================================
        // "what gpu" / "which gpu" / "gpu info" / "graphics card"
        if (q.contains("gpu") || q.contains("graphics card") || q.contains("video card"))
            && (q.contains("what") || q.contains("which") || q.contains("my")
                || q.contains("info") || q.contains("have") || q.contains("show"))
        {
            return Self::GpuInfo;
        }
        // Short forms
        if q == "gpu?" || q == "gpu" || q == "graphics?" {
            return Self::GpuInfo;
        }

        // =================================================================
        // Network patterns (v3.12.0 - NEW)
        // =================================================================
        // "network interfaces" / "my ip" / "what's my ip" / "network info"
        if (q.contains("network") || q.contains("interface"))
            && (q.contains("what") || q.contains("show") || q.contains("list") || q.contains("my"))
        {
            return Self::NetworkInfo;
        }
        // IP address questions
        if (q.contains("ip") && (q.contains("address") || q.contains("my") || q.contains("what")))
            || q.contains("my ip")
        {
            return Self::NetworkInfo;
        }
        // Short forms
        if q == "network?" || q == "interfaces?" || q == "ip?" {
            return Self::NetworkInfo;
        }

        Self::Unknown
    }
}

/// Check if question is asking for Snow Leopard benchmark (v1.4.0)
/// v2.3.0: Enhanced triggers for "run the full snow leopard benchmark", etc.
fn is_benchmark_question(q: &str) -> bool {
    // "snow leopard benchmark" / "snow leopard test"
    if q.contains("snow leopard") && (q.contains("benchmark") || q.contains("test")) {
        return true;
    }
    // "run benchmark" / "run the benchmark" / "run a benchmark"
    if q.contains("run") && q.contains("benchmark") {
        return true;
    }
    // "run the snow leopard" / "run snow leopard"
    if q.contains("run") && q.contains("snow leopard") {
        return true;
    }
    // "benchmark anna" / "benchmark yourself" / "benchmark me"
    if q.contains("benchmark") && (q.contains("anna") || q.contains("yourself") || q.contains(" me")) {
        return true;
    }
    // "sanity benchmark" / "quick benchmark" / "fast benchmark" / "short benchmark"
    if q.contains("benchmark") && (q.contains("sanity") || q.contains("quick")
        || q.contains("fast") || q.contains("short")) {
        return true;
    }
    // v2.3.0: "full benchmark" / "complete benchmark"
    if q.contains("benchmark") && (q.contains("full") || q.contains("complete")) {
        return true;
    }
    // v2.3.0: Just "benchmark" alone (interpreted as full)
    if q.trim() == "benchmark" || q.trim() == "benchmark!" {
        return true;
    }
    false
}

/// Check if benchmark request is for quick mode (v1.4.0)
fn is_quick_benchmark(q: &str) -> bool {
    q.contains("quick") || q.contains("short") || q.contains("fast") || q.contains("sanity")
}

/// Check if question is asking for benchmark history (v1.5.0)
fn is_benchmark_history_question(q: &str) -> bool {
    // "benchmark history" / "snow leopard history"
    if (q.contains("benchmark") || q.contains("snow leopard")) && q.contains("history") {
        return true;
    }
    // "show benchmark runs" / "list benchmarks"
    if (q.contains("show") || q.contains("list")) && q.contains("benchmark") {
        return true;
    }
    // "past benchmarks" / "previous benchmarks"
    if (q.contains("past") || q.contains("previous")) && q.contains("benchmark") {
        return true;
    }
    false
}

/// Check if question is asking for benchmark delta/comparison (v1.5.0)
fn is_benchmark_delta_question(q: &str) -> bool {
    // "compare benchmarks" / "benchmark comparison" / "benchmark delta"
    if q.contains("benchmark") && (q.contains("compare") || q.contains("comparison") || q.contains("delta")) {
        return true;
    }
    // "compare snow leopard" / "snow leopard delta"
    if q.contains("snow leopard") && (q.contains("compare") || q.contains("delta") || q.contains("diff")) {
        return true;
    }
    // "compare last two benchmarks"
    if q.contains("compare") && q.contains("last") && (q.contains("benchmark") || q.contains("two")) {
        return true;
    }
    // "benchmark change" / "what changed in benchmark"
    if q.contains("benchmark") && (q.contains("change") || q.contains("improved") || q.contains("regressed")) {
        return true;
    }
    false
}

/// Check if question is asking for factory reset (hard reset, deletes knowledge) (v1.3.0)
/// v2.1.0: Added more natural language triggers
/// v3.13.0: Skip if starts with "yes" (that's a confirmation, not a new request)
fn is_factory_reset_question(q: &str) -> bool {
    // v3.13.0: If the message starts with "yes", it's a confirmation, not a new reset request
    // This prevents "yes, hard reset everything" from triggering a NEW reset dialog
    if q.trim_start().starts_with("yes") {
        return false;
    }

    // Factory reset requires explicit "factory reset" phrase or explicit "delete knowledge/everything"
    // This is the HARD reset that deletes knowledge

    // Direct factory reset phrase
    if q.contains("factory reset") {
        return true;
    }

    // v2.1.0: "hard reset" as explicit trigger
    if q.contains("hard reset") {
        return true;
    }

    // v2.1.0: "reset everything" / "reset all"
    if q.contains("reset") && (q.contains("everything") || q.contains(" all")) {
        return true;
    }

    // "delete everything" / "wipe everything" / "erase everything"
    if (q.contains("delete") || q.contains("wipe") || q.contains("erase"))
        && (q.contains("everything") || q.contains("all data") || q.contains("all your"))
    {
        return true;
    }

    // "reset to factory" / "full reset" / "complete reset"
    if q.contains("reset") && (q.contains("full") || q.contains("complete") || q.contains("total")) {
        return true;
    }

    // "delete knowledge" / "wipe knowledge" / "clear knowledge"
    if (q.contains("delete") || q.contains("wipe") || q.contains("clear") || q.contains("remove"))
        && q.contains("knowledge")
    {
        return true;
    }

    false
}

/// Check if question is asking for experience reset (soft reset, keeps knowledge) (v1.3.0)
/// v2.1.0: Added more natural language triggers including "soft reset", "clear memory"
/// v3.13.0: Skip if starts with "yes" (that's a confirmation, not a new request)
fn is_reset_experience_question(q: &str) -> bool {
    // v3.13.0: If the message starts with "yes", it's a confirmation, not a new reset request
    if q.trim_start().starts_with("yes") {
        return false;
    }

    // v2.1.0: Direct "soft reset" trigger
    if q.contains("soft reset") {
        return true;
    }

    // v2.1.0: "clear your memory" / "clear memory" (soft reset, not factory)
    if q.contains("clear") && q.contains("memory") && !q.contains("everything") && !q.contains("knowledge") {
        return true;
    }

    // Must contain a reset verb
    let has_reset_verb = q.contains("reset")
        || q.contains("clear")
        || q.contains("wipe")
        || q.contains("forget")
        || q.contains("fresh");

    if !has_reset_verb {
        return false;
    }

    // Must reference experience, xp, stats, learning, or similar (but NOT knowledge)
    let has_target = q.contains("experience")
        || q.contains("xp")
        || q.contains("stats")
        || q.contains("learning")
        || q.contains("telemetry")
        || q.contains("history")
        || q.contains("progress")
        || q.contains("level")
        || q.contains("trust")
        || q.contains("streaks")
        || (q.contains("your") && (q.contains("data") || q.contains("state") || q.contains("memory")))
        || q.contains("start fresh")
        || q.contains("start over")
        || q.contains("clean slate");

    // Exclude if it's asking about knowledge (that's factory reset)
    // Also exclude if it's "reset everything" (that's factory reset)
    let excludes_knowledge = !q.contains("knowledge") && !q.contains("everything");

    has_target && excludes_knowledge
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
    /// Whether this answer requires confirmation before action (v1.3.0)
    pub pending_confirmation: bool,
    /// Type of pending action (for confirmation flow) (v1.3.0)
    pub pending_action: Option<PendingActionType>,
}

/// Type of pending action requiring confirmation (v1.3.0)
#[derive(Debug, Clone, PartialEq)]
pub enum PendingActionType {
    /// Experience reset (soft reset) - requires "yes"
    ExperienceReset,
    /// Factory reset (hard reset) - requires exact phrase
    FactoryReset,
}

impl PendingActionType {
    /// Get the confirmation string required for this action
    pub fn confirmation_required(&self) -> &'static str {
        match self {
            PendingActionType::ExperienceReset => "yes",
            PendingActionType::FactoryReset => "I UNDERSTAND AND CONFIRM FACTORY RESET",
        }
    }

    /// Check if the given input confirms this action
    /// v3.13.0: Factory reset now uses flexible confirmation matching
    pub fn is_confirmed(&self, input: &str) -> bool {
        let trimmed = input.trim();
        match self {
            PendingActionType::ExperienceReset => {
                trimmed.eq_ignore_ascii_case("yes") || trimmed.eq_ignore_ascii_case("y")
            }
            PendingActionType::FactoryReset => {
                // v3.13.0: Use the flexible confirmation matcher
                is_factory_reset_confirmation(input)
            }
        }
    }
}

impl FastAnswer {
    pub fn new(text: &str, citations: Vec<&str>, reliability: f64) -> Self {
        Self {
            text: text.to_string(),
            citations: citations.into_iter().map(|s| s.to_string()).collect(),
            reliability,
            origin: "Brain".to_string(),
            duration_ms: 0,
            pending_confirmation: false,
            pending_action: None,
        }
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Mark as requiring experience reset confirmation (v1.3.0)
    pub fn with_experience_reset_pending(mut self) -> Self {
        self.pending_confirmation = true;
        self.pending_action = Some(PendingActionType::ExperienceReset);
        self
    }

    /// Mark as requiring factory reset confirmation (v1.3.0)
    pub fn with_factory_reset_pending(mut self) -> Self {
        self.pending_confirmation = true;
        self.pending_action = Some(PendingActionType::FactoryReset);
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
        FastQuestionType::CpuModel => fast_cpu_model_answer(),
        FastQuestionType::RootDiskSpace => fast_disk_answer(),
        FastQuestionType::OsInfo => fast_os_answer(),
        FastQuestionType::Uptime => fast_uptime_answer(),
        FastQuestionType::AnnaHealth => fast_health_answer(),
        FastQuestionType::DebugEnable => fast_debug_enable(),
        FastQuestionType::DebugDisable => fast_debug_disable(),
        FastQuestionType::DebugStatus => fast_debug_status(),
        FastQuestionType::ResetExperience => fast_reset_experience_confirm(),
        FastQuestionType::ResetFactory => fast_reset_factory_confirm(),
        // v3.13.0: Direct confirmation handlers - execute reset immediately
        FastQuestionType::ResetFactoryConfirm => Some(execute_factory_reset()),
        FastQuestionType::ResetExperienceConfirm => Some(execute_experience_reset()),
        FastQuestionType::BenchmarkFull => fast_benchmark_response(false),
        FastQuestionType::BenchmarkQuick => fast_benchmark_response(true),
        FastQuestionType::BenchmarkHistory => fast_benchmark_history(),
        FastQuestionType::BenchmarkDelta => fast_benchmark_delta(),
        FastQuestionType::DailyCheckIn => fast_daily_checkin(),
        FastQuestionType::GpuInfo => fast_gpu_answer(),
        FastQuestionType::NetworkInfo => fast_network_answer(),
        FastQuestionType::FirstLight => fast_first_light(),
        FastQuestionType::Unknown => return None,
    };

    result.map(|mut ans| {
        ans.duration_ms = start.elapsed().as_millis() as u64;
        ans
    })
}

/// Get RAM info fast (v1.1.0 - uses cache for total, fresh for available)
pub fn fast_ram_answer() -> Option<FastAnswer> {
    // Get total from cache (doesn't change)
    let total_kb = get_cached_ram_total()?;
    let gib = total_kb as f64 / 1024.0 / 1024.0;

    // Get available memory fresh (changes frequently)
    let output = Command::new("cat")
        .arg("/proc/meminfo")
        .output()
        .ok()?;
    let content = String::from_utf8_lossy(&output.stdout);

    let re_avail = Regex::new(r"MemAvailable:\s*(\d+)\s*kB").ok()?;
    let avail_kb: u64 = if let Some(caps) = re_avail.captures(&content) {
        caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(0)
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

/// Get CPU info fast (v1.1.0 - uses cache)
pub fn fast_cpu_answer() -> Option<FastAnswer> {
    let content = get_cached_cpu_info()?;

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

/// Get CPU model fast (v1.1.0 - NEW)
fn fast_cpu_model_answer() -> Option<FastAnswer> {
    let content = get_cached_cpu_info()?;

    let re_model = Regex::new(r"(?m)Model name:\s*(.+)").ok()?;

    let model: String = re_model.captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    if model == "Unknown" {
        return None;
    }

    let text = format!("Your CPU is a {}.", model);

    Some(FastAnswer::new(&text, vec!["lscpu"], 0.99))
}

/// Get disk info fast
pub fn fast_disk_answer() -> Option<FastAnswer> {
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

/// Get GPU information fast path
pub fn fast_gpu_answer() -> Option<FastAnswer> {
    // Try lspci for GPU detection
    let lspci_output = Command::new("lspci")
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&lspci_output.stdout);

    // Look for VGA or 3D controller entries
    let gpu_lines: Vec<&str> = content
        .lines()
        .filter(|line| {
            line.contains("VGA") || line.contains("3D controller") || line.contains("Display")
        })
        .collect();

    if gpu_lines.is_empty() {
        return Some(FastAnswer::new(
            "No GPU detected on this system (no VGA/3D controller found in lspci).",
            vec!["lspci"],
            0.95
        ));
    }

    // Try nvidia-smi for detailed NVIDIA info
    let nvidia_info = Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total,driver_version", "--format=csv,noheader"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let text = if let Some(nvidia) = nvidia_info {
        // Parse NVIDIA output: "GeForce RTX 3080, 10240 MiB, 535.154.05"
        let parts: Vec<&str> = nvidia.split(", ").collect();
        if parts.len() >= 3 {
            format!(
                "GPU: {} with {} VRAM (NVIDIA driver {})",
                parts[0], parts[1], parts[2]
            )
        } else {
            format!("GPU detected: {}", gpu_lines.join("; "))
        }
    } else {
        // No NVIDIA, report what lspci found
        let gpus: Vec<String> = gpu_lines
            .iter()
            .map(|line| {
                // Extract just the device name from lspci output
                line.split(": ").nth(1).unwrap_or(line).to_string()
            })
            .collect();
        format!("GPU: {}", gpus.join("; "))
    };

    Some(FastAnswer::new(&text, vec!["lspci", "nvidia-smi"], 0.95))
}

/// Get OS/distro information fast path
pub fn fast_os_answer() -> Option<FastAnswer> {
    // First, get kernel info
    let uname_output = Command::new("uname")
        .args(["-srm"])
        .output()
        .ok()?;
    let kernel = String::from_utf8_lossy(&uname_output.stdout).trim().to_string();

    // Try to get distro info from /etc/os-release
    let distro = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            let mut name = None;
            let mut version = None;
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return Some(line.trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"').to_string());
                }
                if line.starts_with("NAME=") {
                    name = Some(line.trim_start_matches("NAME=").trim_matches('"').to_string());
                }
                if line.starts_with("VERSION=") {
                    version = Some(line.trim_start_matches("VERSION=").trim_matches('"').to_string());
                }
            }
            match (name, version) {
                (Some(n), Some(v)) => Some(format!("{} {}", n, v)),
                (Some(n), None) => Some(n),
                _ => None
            }
        })
        .unwrap_or_else(|| "Unknown Linux distribution".to_string());

    let text = format!("{} running kernel {}", distro, kernel);

    Some(FastAnswer::new(&text, vec!["uname -srm", "cat /etc/os-release"], 0.99))
}

/// Get uptime information fast path
pub fn fast_uptime_answer() -> Option<FastAnswer> {
    let output = Command::new("uptime")
        .args(["-p"])
        .output()
        .ok()?;

    let uptime_pretty = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // uptime -p outputs "up X days, Y hours, Z minutes"
    let text = if uptime_pretty.starts_with("up ") {
        format!("System has been {} .", uptime_pretty)
    } else {
        format!("System uptime: {}", uptime_pretty)
    };

    Some(FastAnswer::new(&text, vec!["uptime -p"], 0.99))
}

/// Get network interfaces summary fast path
pub fn fast_network_answer() -> Option<FastAnswer> {
    let output = Command::new("ip")
        .args(["addr", "show"])
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&output.stdout);

    // Parse interfaces and their IPs
    let mut interfaces: Vec<String> = Vec::new();
    let mut current_iface = String::new();
    let mut current_ips: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some(caps) = line.strip_prefix(|c: char| c.is_ascii_digit()) {
            // New interface line like "2: eth0: ..."
            if !current_iface.is_empty() && !current_ips.is_empty() {
                interfaces.push(format!("{}: {}", current_iface, current_ips.join(", ")));
            }
            // Extract interface name
            if let Some(name) = caps.split(':').nth(1) {
                current_iface = name.trim().split('@').next().unwrap_or(name).to_string();
                current_ips.clear();
            }
        } else if line.contains("inet ") && !line.contains("inet6") {
            // IPv4 address line
            if let Some(ip) = line.split_whitespace().nth(1) {
                current_ips.push(ip.to_string());
            }
        }
    }

    // Don't forget last interface
    if !current_iface.is_empty() && !current_ips.is_empty() {
        interfaces.push(format!("{}: {}", current_iface, current_ips.join(", ")));
    }

    let text = if interfaces.is_empty() {
        "No network interfaces with IP addresses found.".to_string()
    } else {
        format!("Network interfaces:\n{}", interfaces.join("\n"))
    };

    Some(FastAnswer::new(&text, vec!["ip addr show"], 0.95))
}

/// Get annad service logs summary fast path
pub fn fast_logs_summary() -> Option<FastAnswer> {
    // Get recent logs from journalctl
    let output = Command::new("journalctl")
        .args(["--user-unit=annad", "--since", "1 hour ago", "-n", "20", "--no-pager"])
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || (lines.len() == 1 && lines[0].contains("No entries")) {
        return Some(FastAnswer::new(
            "No annad logs found in the last hour. The service may not be logging or may use a different name.",
            vec!["journalctl --user-unit=annad --since '1 hour ago'"],
            0.8
        ));
    }

    let count = lines.len();
    let errors = lines.iter().filter(|l| l.contains("error") || l.contains("ERROR")).count();
    let warnings = lines.iter().filter(|l| l.contains("warn") || l.contains("WARN")).count();

    let text = format!(
        "Last hour annad logs: {} entries ({} errors, {} warnings).\n\nMost recent:\n{}",
        count, errors, warnings,
        lines.iter().rev().take(5).cloned().collect::<Vec<_>>().join("\n")
    );

    Some(FastAnswer::new(&text, vec!["journalctl --user-unit=annad --since '1 hour ago' -n 20"], 0.9))
}

/// Check for available system updates fast path
pub fn fast_updates_check() -> Option<FastAnswer> {
    // Try pacman first (Arch Linux)
    if let Ok(output) = Command::new("checkupdates").output() {
        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            let count = content.lines().count();
            if count == 0 {
                return Some(FastAnswer::new(
                    "No package updates available. System is up to date.",
                    vec!["checkupdates"],
                    0.95
                ));
            }
            let packages: Vec<&str> = content.lines().take(10).collect();
            let text = if count > 10 {
                format!("{} updates available:\n{}\n... and {} more", count, packages.join("\n"), count - 10)
            } else {
                format!("{} updates available:\n{}", count, packages.join("\n"))
            };
            return Some(FastAnswer::new(&text, vec!["checkupdates"], 0.95));
        }
    }

    // Fallback: check if pacman database is recent
    Some(FastAnswer::new(
        "Could not check for updates. Run 'checkupdates' manually (Arch) or use your package manager.",
        vec!["checkupdates"],
        0.5
    ))
}

/// Check Anna health fast
/// v3.10.0: Fixed to be honest - can't say "healthy" if API isn't responding
pub fn fast_health_answer() -> Option<FastAnswer> {
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

    // Check daemon port (use port 7865 which is the actual annad port)
    let port_open = Command::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "2", "http://127.0.0.1:7865/health"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("200"))
        .unwrap_or(false);

    let mut status_parts = Vec::new();
    let mut issues = 0;

    if annad_running {
        status_parts.push("annad daemon is running");
    } else {
        status_parts.push("annad daemon is NOT running");
        issues += 1;
    }

    if ollama_running {
        status_parts.push("Ollama LLM service is running");
    } else {
        status_parts.push("Ollama LLM service is NOT running");
        issues += 1;
    }

    if port_open {
        status_parts.push("API endpoint is responding");
    } else {
        status_parts.push("API endpoint is NOT responding");
        // v3.10.0: This IS a problem - count it as an issue
        issues += 1;
    }

    // v3.10.0: Be honest about health status
    let summary = match issues {
        0 => "I'm healthy and fully operational.",
        1 => "I'm partially operational with one issue.",
        2 => "I'm degraded with multiple issues.",
        _ => "I'm experiencing significant issues.",
    };

    let text = format!(
        "{}\n\nStatus:\n- {}",
        summary,
        status_parts.join("\n- ")
    );

    // v3.10.0: Reliability reflects actual health
    let reliability = match issues {
        0 => 0.99,
        1 => 0.75,
        2 => 0.50,
        _ => 0.30,
    };

    Some(FastAnswer::new(&text, vec!["pgrep annad", "pgrep ollama", "curl /health"], reliability))
}

// ============================================================================
// Debug Mode Handlers (v0.89.0)
// ============================================================================

/// Enable debug mode
pub fn fast_debug_enable() -> Option<FastAnswer> {
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
pub fn fast_debug_disable() -> Option<FastAnswer> {
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
pub fn fast_debug_status() -> Option<FastAnswer> {
    let state = DebugState::load();
    Some(FastAnswer::new(
        &state.format_status(),
        vec!["debug_state.json"],
        0.99,
    ))
}

// ============================================================================
// Reset Functions (v1.3.0) - Experience and Factory
// ============================================================================

/// Ask for confirmation before experience reset (soft reset)
/// v2.2.0: Improved UX with clearer confirmation
pub fn fast_reset_experience_confirm() -> Option<FastAnswer> {
    use crate::experience_reset::{ExperiencePaths, ExperienceSnapshot};

    let paths = ExperiencePaths::default();
    let snapshot = ExperienceSnapshot::capture(&paths);

    let text = format!(
        "═══════════════════════════════════════════\n\
         🔄  SOFT RESET REQUESTED\n\
         ═══════════════════════════════════════════\n\n\
         This clears short-term memory and patterns but keeps XP.\n\n\
         What will be reset:\n\
         - Trust scores → 0.5 (neutral)\n\
         - Streaks → 0\n\
         - Stats counters → 0\n\
         - Telemetry history ({} events)\n\n\
         What will be preserved:\n\
         - XP and levels (currently: level {}, {} XP)\n\
         - Knowledge base\n\n\
         ───────────────────────────────────────────\n\
         Type: yes, soft reset",
        snapshot.telemetry_line_count, snapshot.anna_level, snapshot.anna_xp
    );

    Some(FastAnswer::new(&text, vec!["experience_reset"], 0.99).with_experience_reset_pending())
}

/// Ask for confirmation before factory reset (hard reset)
/// v2.2.0: Improved UX with clearer confirmation
pub fn fast_reset_factory_confirm() -> Option<FastAnswer> {
    use crate::experience_reset::{ExperiencePaths, ExperienceSnapshot};

    let paths = ExperiencePaths::default();
    let snapshot = ExperienceSnapshot::capture(&paths);

    let text = format!(
        "═══════════════════════════════════════════\n\
         ⚠️   HARD RESET REQUESTED\n\
         ═══════════════════════════════════════════\n\n\
         This will erase XP, telemetry, stats, patterns and all learning.\n\n\
         What will be DELETED:\n\
         - XP, levels, trust, streaks → reset to baseline\n\
         - Telemetry history ({} events)\n\
         - Stats and learning artifacts\n\
         - Knowledge base ({} files)\n\
         - All patterns and learning\n\n\
         Current state you will LOSE:\n\
         - Level {}, {} XP, {} questions answered\n\n\
         This is IRREVERSIBLE.\n\n\
         ───────────────────────────────────────────\n\
         Are you absolutely sure?\n\
         Type: yes, hard reset",
        snapshot.telemetry_line_count,
        snapshot.knowledge_file_count,
        snapshot.anna_level,
        snapshot.anna_xp,
        snapshot.total_questions
    );

    Some(FastAnswer::new(&text, vec!["factory_reset"], 0.99).with_factory_reset_pending())
}

/// Execute experience reset (called after confirmation)
pub fn execute_experience_reset() -> FastAnswer {
    use crate::experience_reset::{reset_experience_default, ExperienceSnapshot, ExperiencePaths};

    // Capture pre-reset snapshot for summary
    let paths = ExperiencePaths::default();
    let snapshot = ExperienceSnapshot::capture(&paths);

    // Perform reset
    let result = reset_experience_default();

    if result.success {
        let text = format!(
            "Experience reset complete. XP, telemetry, and stats have been reset to baseline.\n\
             Knowledge has been preserved.\n\n\
             Reset summary:\n\
             - {} components reset\n\
             - {} already clean\n\n\
             Pre-reset state:\n\
             - Level: {} (now 1)\n\
             - XP: {} (now 0)\n\
             - Trust: (now 0.5)\n\
             - Questions answered: {} (now 0)\n\
             - Telemetry events: {} (now 0)",
            result.components_reset.len(),
            result.components_clean.len(),
            snapshot.anna_level,
            snapshot.anna_xp,
            snapshot.total_questions,
            snapshot.telemetry_line_count
        );

        FastAnswer::new(&text, vec!["experience_reset"], 0.99)
    } else {
        let text = format!(
            "Experience reset completed with errors:\n{}\n\n\
             Components reset: {:?}\n\
             Components clean: {:?}",
            result.errors.join("\n"),
            result.components_reset,
            result.components_clean
        );

        FastAnswer::new(&text, vec!["experience_reset"], 0.70)
    }
}

/// Execute factory reset (called after confirmation)
pub fn execute_factory_reset() -> FastAnswer {
    use crate::experience_reset::{reset_factory_default, ExperienceSnapshot, ExperiencePaths};

    // Capture pre-reset snapshot for summary
    let paths = ExperiencePaths::default();
    let snapshot = ExperienceSnapshot::capture(&paths);

    // Perform factory reset
    let result = reset_factory_default();

    if result.success {
        let text = format!(
            "Factory reset complete. ALL data has been deleted.\n\n\
             Reset summary:\n\
             - {} components reset\n\
             - {} already clean\n\n\
             Pre-reset state:\n\
             - Level: {} (now 1)\n\
             - XP: {} (now 0)\n\
             - Questions answered: {}\n\
             - Telemetry events: {}\n\
             - Knowledge files: {} (deleted)",
            result.components_reset.len(),
            result.components_clean.len(),
            snapshot.anna_level,
            snapshot.anna_xp,
            snapshot.total_questions,
            snapshot.telemetry_line_count,
            snapshot.knowledge_file_count
        );

        FastAnswer::new(&text, vec!["factory_reset"], 0.99)
    } else {
        let text = format!(
            "Factory reset completed with errors:\n{}\n\n\
             Components reset: {:?}\n\
             Components clean: {:?}",
            result.errors.join("\n"),
            result.components_reset,
            result.components_clean
        );

        FastAnswer::new(&text, vec!["factory_reset"], 0.70)
    }
}

/// Check if a response is a soft reset confirmation (v2.2.0 - improved UX)
pub fn is_confirmation(response: &str) -> bool {
    let r = response.trim().to_lowercase();
    r == "yes" || r == "y" || r == "confirm" || r == "ok" ||
    r == "yes, soft reset" || r == "yes soft reset"
}

/// Check if a response is a factory/hard reset confirmation (v3.13.0 - more flexible)
pub fn is_factory_reset_confirmation(response: &str) -> bool {
    let r = response.trim().to_lowercase();
    // Accept various confirmation formats
    response.trim() == "I UNDERSTAND AND CONFIRM FACTORY RESET" ||
    r == "yes, hard reset" || r == "yes hard reset" ||
    // v3.13.0: Accept more natural confirmations (user might add extra words)
    (r.starts_with("yes") && r.contains("hard") && r.contains("reset")) ||
    (r.starts_with("yes") && r.contains("factory")) ||
    r == "yes, factory reset" || r == "yes factory reset"
}

// ============================================================================
// Benchmark Handlers (v1.4.0) - Snow Leopard
// ============================================================================

/// Return response for benchmark request
/// The actual benchmark execution is handled by the orchestrator/daemon
fn fast_benchmark_response(is_quick: bool) -> Option<FastAnswer> {
    let mode = if is_quick { "quick" } else { "full" };
    let phases = if is_quick { "3" } else { "6" };
    let est_time = if is_quick { "1-2 minutes" } else { "3-5 minutes" };

    let text = format!(
        "Starting Snow Leopard Benchmark ({} mode)...\n\n\
         This benchmark will run {} phases to measure Anna's real-world performance:\n\
         - Answering canonical and paraphrased questions\n\
         - Testing learning and retention\n\
         - Measuring response times\n\n\
         Estimated time: {}\n\n\
         The benchmark runs asynchronously. Check 'annactl status' for results.",
        mode, phases, est_time
    );

    let mut answer = FastAnswer::new(&text, vec!["bench_snow_leopard"], 0.99);
    answer.origin = format!("Benchmark-{}", if is_quick { "Quick" } else { "Full" });
    Some(answer)
}

/// Check if a fast answer is a benchmark trigger (v1.4.0)
/// This allows the orchestrator to detect and route benchmark requests
pub fn is_benchmark_trigger(answer: &FastAnswer) -> bool {
    answer.origin.starts_with("Benchmark-")
}

/// Get benchmark mode from a trigger answer (v1.4.0)
pub fn get_benchmark_mode_from_trigger(answer: &FastAnswer) -> Option<&'static str> {
    if answer.origin == "Benchmark-Quick" {
        Some("quick")
    } else if answer.origin == "Benchmark-Full" {
        Some("full")
    } else {
        None
    }
}

/// Return benchmark history (v1.5.0)
fn fast_benchmark_history() -> Option<FastAnswer> {
    use crate::bench_snow_leopard::{BenchmarkHistoryEntry, format_benchmark_history};

    let entries = BenchmarkHistoryEntry::list_recent(10);

    if entries.is_empty() {
        let text = "No benchmark history found.\n\n\
                    Run a benchmark first with: \"run the snow leopard benchmark\"";
        return Some(FastAnswer::new(text, vec!["bench_history"], 0.99));
    }

    let text = format_benchmark_history(&entries);
    let mut answer = FastAnswer::new(&text, vec!["bench_history"], 0.99);
    answer.origin = "Benchmark-History".to_string();
    Some(answer)
}

/// Return benchmark delta/comparison (v1.5.0)
fn fast_benchmark_delta() -> Option<FastAnswer> {
    use crate::bench_snow_leopard::{compare_last_two_benchmarks, format_benchmark_delta};

    match compare_last_two_benchmarks() {
        Some(delta) => {
            let text = format_benchmark_delta(&delta);
            let mut answer = FastAnswer::new(&text, vec!["bench_delta"], 0.99);
            answer.origin = "Benchmark-Delta".to_string();
            Some(answer)
        }
        None => {
            let text = "Need at least 2 benchmark runs to show comparison.\n\n\
                        Run benchmarks with: \"run the snow leopard benchmark\"";
            Some(FastAnswer::new(text, vec!["bench_delta"], 0.99))
        }
    }
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
// Daily Check-In (v2.2.0)
// ============================================================================

/// Generate daily check-in response
pub fn fast_daily_checkin() -> Option<FastAnswer> {
    use crate::first_light::DailyCheckIn;

    let checkin = DailyCheckIn::generate();
    let text = checkin.format_display();

    Some(FastAnswer::new(&text, vec!["daily_checkin"], 0.99))
}

/// v3.13.1: First Light self-test (Brain-only, bypasses LLM entirely)
/// Runs the diagnostic tests locally using system commands
pub fn fast_first_light() -> Option<FastAnswer> {
    use crate::first_light::run_first_light;

    let result = run_first_light();
    let text = result.format_summary();

    // Calculate reliability based on test pass rate
    let reliability = if result.all_passed {
        0.99
    } else {
        let passed = result.questions.iter().filter(|q| q.success).count();
        let total = result.questions.len();
        (passed as f64 / total as f64) * 0.99
    };

    Some(FastAnswer::new(&text, vec!["first_light"], reliability))
}

// ============================================================================
// Fallback Answer
// ============================================================================

/// Create a fallback answer when everything fails
pub fn create_fallback_answer(_question: &str, evidence: Option<&str>, error: &str) -> FastAnswer {
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
        pending_confirmation: false,
        pending_action: None,
    }
}

/// Create a partial answer from available evidence
pub fn create_partial_answer(question: &str, evidence: &str, _probe_used: &str) -> Option<FastAnswer> {
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
        // v1.1.0 expanded patterns
        assert_eq!(FastQuestionType::classify("my ram?"), FastQuestionType::Ram);
        assert_eq!(FastQuestionType::classify("check ram"), FastQuestionType::Ram);
        assert_eq!(FastQuestionType::classify("ram"), FastQuestionType::Ram);
        assert_eq!(FastQuestionType::classify("memory?"), FastQuestionType::Ram);
    }

    #[test]
    fn test_classify_cpu_model() {
        // v1.1.0 CPU model patterns
        assert_eq!(FastQuestionType::classify("what cpu do I have?"), FastQuestionType::CpuModel);
        assert_eq!(FastQuestionType::classify("which processor?"), FastQuestionType::CpuModel);
        assert_eq!(FastQuestionType::classify("cpu model?"), FastQuestionType::CpuModel);
        assert_eq!(FastQuestionType::classify("what chip?"), FastQuestionType::CpuModel);
    }

    #[test]
    fn test_classify_cpu() {
        assert_eq!(FastQuestionType::classify("how many cpu cores?"), FastQuestionType::CpuCores);
        assert_eq!(FastQuestionType::classify("how many cores does my computer have"), FastQuestionType::CpuCores);
        assert_eq!(FastQuestionType::classify("number of threads?"), FastQuestionType::CpuCores);
        // v1.1.0 expanded patterns
        assert_eq!(FastQuestionType::classify("cores?"), FastQuestionType::CpuCores);
        assert_eq!(FastQuestionType::classify("threads?"), FastQuestionType::CpuCores);
        assert_eq!(FastQuestionType::classify("my cpu threads"), FastQuestionType::CpuCores);
    }

    #[test]
    fn test_classify_disk() {
        assert_eq!(FastQuestionType::classify("how much free disk space on root?"), FastQuestionType::RootDiskSpace);
        assert_eq!(FastQuestionType::classify("disk space available?"), FastQuestionType::RootDiskSpace);
        // v1.1.0 expanded patterns
        assert_eq!(FastQuestionType::classify("df"), FastQuestionType::RootDiskSpace);
        assert_eq!(FastQuestionType::classify("disk usage"), FastQuestionType::RootDiskSpace);
        assert_eq!(FastQuestionType::classify("storage?"), FastQuestionType::RootDiskSpace);
        assert_eq!(FastQuestionType::classify("how much space left"), FastQuestionType::RootDiskSpace);
    }

    #[test]
    fn test_classify_health() {
        assert_eq!(FastQuestionType::classify("diagnose yourself"), FastQuestionType::AnnaHealth);
        assert_eq!(FastQuestionType::classify("what is your health?"), FastQuestionType::AnnaHealth);
        assert_eq!(FastQuestionType::classify("are you ok?"), FastQuestionType::AnnaHealth);
        // v1.1.0 expanded patterns
        assert_eq!(FastQuestionType::classify("are you alive?"), FastQuestionType::AnnaHealth);
        assert_eq!(FastQuestionType::classify("health check"), FastQuestionType::AnnaHealth);
        assert_eq!(FastQuestionType::classify("check yourself"), FastQuestionType::AnnaHealth);
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

    #[test]
    fn test_classify_reset_experience() {
        // Experience reset patterns (soft reset, keeps knowledge)
        assert_eq!(
            FastQuestionType::classify("reset your experience"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("reset your xp"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("start fresh and forget your XP"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("clear your telemetry"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("wipe your learning history"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("reset your stats"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("reset your progress"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("clear your level and trust"),
            FastQuestionType::ResetExperience
        );

        // v2.1.0: New soft reset triggers
        assert_eq!(
            FastQuestionType::classify("soft reset"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("clear your memory"),
            FastQuestionType::ResetExperience
        );
        assert_eq!(
            FastQuestionType::classify("clear memory"),
            FastQuestionType::ResetExperience
        );

        // Should NOT match experience reset
        assert_ne!(
            FastQuestionType::classify("reset my computer"),
            FastQuestionType::ResetExperience
        );
        assert_ne!(
            FastQuestionType::classify("how much experience do you have?"),
            FastQuestionType::ResetExperience
        );
    }

    #[test]
    fn test_classify_factory_reset() {
        // Factory reset patterns (hard reset, deletes everything)
        assert_eq!(
            FastQuestionType::classify("factory reset"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("factory reset your memory"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("delete everything"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("wipe all your data"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("full reset"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("complete reset"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("delete knowledge"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("wipe your knowledge base"),
            FastQuestionType::ResetFactory
        );

        // v2.1.0: New hard reset triggers
        assert_eq!(
            FastQuestionType::classify("hard reset"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("reset everything"),
            FastQuestionType::ResetFactory
        );
        assert_eq!(
            FastQuestionType::classify("reset all"),
            FastQuestionType::ResetFactory
        );

        // Should NOT match factory reset
        assert_ne!(
            FastQuestionType::classify("reset your xp"),
            FastQuestionType::ResetFactory
        );
    }

    #[test]
    fn test_is_confirmation() {
        assert!(is_confirmation("yes"));
        assert!(is_confirmation("Yes"));
        assert!(is_confirmation("YES"));
        assert!(is_confirmation("y"));
        assert!(is_confirmation("confirm"));
        assert!(is_confirmation("ok"));
        assert!(is_confirmation("  yes  ")); // with whitespace

        assert!(!is_confirmation("no"));
        assert!(!is_confirmation("maybe"));
        assert!(!is_confirmation("cancel"));
        assert!(!is_confirmation(""));
    }

    #[test]
    fn test_factory_reset_confirmation() {
        assert!(is_factory_reset_confirmation("I UNDERSTAND AND CONFIRM FACTORY RESET"));
        assert!(!is_factory_reset_confirmation("yes")); // bare "yes" is not enough
        assert!(!is_factory_reset_confirmation("i understand and confirm factory reset")); // wrong case for old format
        assert!(is_factory_reset_confirmation("  I UNDERSTAND AND CONFIRM FACTORY RESET  ")); // whitespace is trimmed

        // v3.13.0: More flexible formats
        assert!(is_factory_reset_confirmation("yes, hard reset"));
        assert!(is_factory_reset_confirmation("yes hard reset"));
        assert!(is_factory_reset_confirmation("yes, hard reset everything to factory state")); // user's actual input
        assert!(is_factory_reset_confirmation("yes, factory reset"));
        assert!(is_factory_reset_confirmation("yes factory reset"));
        assert!(is_factory_reset_confirmation("yes, do the hard reset"));
        assert!(is_factory_reset_confirmation("YES, HARD RESET")); // case insensitive
    }

    #[test]
    fn test_pending_action_type_confirmation() {
        // Experience reset accepts "yes" or "y"
        assert!(PendingActionType::ExperienceReset.is_confirmed("yes"));
        assert!(PendingActionType::ExperienceReset.is_confirmed("YES"));
        assert!(PendingActionType::ExperienceReset.is_confirmed("y"));
        assert!(!PendingActionType::ExperienceReset.is_confirmed("no"));

        // Factory reset - v3.13.0: Now accepts flexible confirmations
        assert!(PendingActionType::FactoryReset.is_confirmed("I UNDERSTAND AND CONFIRM FACTORY RESET"));
        assert!(!PendingActionType::FactoryReset.is_confirmed("yes")); // bare "yes" still not enough
        assert!(PendingActionType::FactoryReset.is_confirmed("yes, hard reset")); // v3.13.0 flexible
        assert!(PendingActionType::FactoryReset.is_confirmed("yes, hard reset everything to factory state")); // user's input
    }

    #[test]
    fn test_classify_benchmark() {
        // Full benchmark patterns
        assert_eq!(
            FastQuestionType::classify("run the snow leopard benchmark"),
            FastQuestionType::BenchmarkFull
        );
        assert_eq!(
            FastQuestionType::classify("run the full benchmark"),
            FastQuestionType::BenchmarkFull
        );
        assert_eq!(
            FastQuestionType::classify("benchmark anna"),
            FastQuestionType::BenchmarkFull
        );
        assert_eq!(
            FastQuestionType::classify("snow leopard test"),
            FastQuestionType::BenchmarkFull
        );

        // Quick benchmark patterns
        assert_eq!(
            FastQuestionType::classify("run a quick snow leopard benchmark"),
            FastQuestionType::BenchmarkQuick
        );
        assert_eq!(
            FastQuestionType::classify("run a fast benchmark"),
            FastQuestionType::BenchmarkQuick
        );
        assert_eq!(
            FastQuestionType::classify("run short benchmark"),
            FastQuestionType::BenchmarkQuick
        );
        assert_eq!(
            FastQuestionType::classify("sanity benchmark"),
            FastQuestionType::BenchmarkQuick
        );

        // Should NOT match benchmark
        assert_ne!(
            FastQuestionType::classify("what is a benchmark?"),
            FastQuestionType::BenchmarkFull
        );
        assert_ne!(
            FastQuestionType::classify("tell me about snow leopards"),
            FastQuestionType::BenchmarkFull
        );
    }

    #[test]
    fn test_benchmark_response() {
        // Full benchmark response
        let full = fast_benchmark_response(false).unwrap();
        assert!(full.text.contains("full mode"));
        assert!(full.text.contains("6 phases"));
        assert_eq!(full.origin, "Benchmark-Full");
        assert!(is_benchmark_trigger(&full));
        assert_eq!(get_benchmark_mode_from_trigger(&full), Some("full"));

        // Quick benchmark response
        let quick = fast_benchmark_response(true).unwrap();
        assert!(quick.text.contains("quick mode"));
        assert!(quick.text.contains("3 phases"));
        assert_eq!(quick.origin, "Benchmark-Quick");
        assert!(is_benchmark_trigger(&quick));
        assert_eq!(get_benchmark_mode_from_trigger(&quick), Some("quick"));

        // Non-benchmark answer
        let other = FastAnswer::new("test", vec![], 0.99);
        assert!(!is_benchmark_trigger(&other));
        assert_eq!(get_benchmark_mode_from_trigger(&other), None);
    }

    // v1.5.0: History and Delta classification tests
    #[test]
    fn test_classify_benchmark_history() {
        assert_eq!(
            FastQuestionType::classify("show benchmark history"),
            FastQuestionType::BenchmarkHistory
        );
        assert_eq!(
            FastQuestionType::classify("snow leopard history"),
            FastQuestionType::BenchmarkHistory
        );
        assert_eq!(
            FastQuestionType::classify("list benchmarks"),
            FastQuestionType::BenchmarkHistory
        );
        assert_eq!(
            FastQuestionType::classify("show past benchmarks"),
            FastQuestionType::BenchmarkHistory
        );
        assert_eq!(
            FastQuestionType::classify("previous benchmark runs"),
            FastQuestionType::BenchmarkHistory
        );
    }

    #[test]
    fn test_classify_benchmark_delta() {
        assert_eq!(
            FastQuestionType::classify("compare benchmarks"),
            FastQuestionType::BenchmarkDelta
        );
        assert_eq!(
            FastQuestionType::classify("benchmark comparison"),
            FastQuestionType::BenchmarkDelta
        );
        assert_eq!(
            FastQuestionType::classify("show benchmark delta"),
            FastQuestionType::BenchmarkDelta
        );
        assert_eq!(
            FastQuestionType::classify("compare last two benchmarks"),
            FastQuestionType::BenchmarkDelta
        );
        assert_eq!(
            FastQuestionType::classify("what changed in the benchmark"),
            FastQuestionType::BenchmarkDelta
        );
        assert_eq!(
            FastQuestionType::classify("has benchmark improved"),
            FastQuestionType::BenchmarkDelta
        );
    }

    // v2.3.0: Enhanced benchmark trigger tests
    #[test]
    fn test_v230_enhanced_benchmark_triggers() {
        // "run the full snow leopard benchmark" - should be full
        assert_eq!(
            FastQuestionType::classify("run the full snow leopard benchmark"),
            FastQuestionType::BenchmarkFull
        );

        // "benchmark anna" - should be full
        assert_eq!(
            FastQuestionType::classify("benchmark anna"),
            FastQuestionType::BenchmarkFull
        );

        // Just "benchmark" alone - should be full
        assert_eq!(
            FastQuestionType::classify("benchmark"),
            FastQuestionType::BenchmarkFull
        );

        // "run a quick benchmark" - should be quick
        assert_eq!(
            FastQuestionType::classify("run a quick benchmark"),
            FastQuestionType::BenchmarkQuick
        );

        // "full benchmark" - should be full
        assert_eq!(
            FastQuestionType::classify("full benchmark"),
            FastQuestionType::BenchmarkFull
        );

        // "complete benchmark" - should be full
        assert_eq!(
            FastQuestionType::classify("complete benchmark"),
            FastQuestionType::BenchmarkFull
        );
    }

    // v2.3.0: Test reduced time budgets
    #[test]
    fn test_v313_realistic_time_budgets() {
        // v3.13.0: Junior budget should be 10 seconds for 4b+ models
        assert_eq!(LLM_A_BUDGET_MS, 10000);

        // v3.13.0: Senior budget should be 12 seconds for 14b models
        assert_eq!(LLM_B_BUDGET_MS, 12000);

        // v3.13.0: Global hard limit should be 20 seconds
        assert_eq!(GLOBAL_HARD_LIMIT_MS, 20000);

        // v3.13.0: Global soft limit should be 15 seconds
        assert_eq!(GLOBAL_SOFT_LIMIT_MS, 15000);

        // Brain budget remains unchanged at 150ms
        assert_eq!(BRAIN_BUDGET_MS, 150);
    }

    // v3.10.0: OS/Kernel classification tests
    #[test]
    fn test_classify_os_kernel() {
        // OS patterns
        assert_eq!(
            FastQuestionType::classify("what os am i running?"),
            FastQuestionType::OsInfo
        );
        assert_eq!(
            FastQuestionType::classify("what operating system is this?"),
            FastQuestionType::OsInfo
        );
        assert_eq!(
            FastQuestionType::classify("which os?"),
            FastQuestionType::OsInfo
        );

        // Kernel patterns
        assert_eq!(
            FastQuestionType::classify("what kernel version?"),
            FastQuestionType::OsInfo
        );
        assert_eq!(
            FastQuestionType::classify("which kernel am i running?"),
            FastQuestionType::OsInfo
        );

        // Combined patterns
        assert_eq!(
            FastQuestionType::classify("what os and kernel version am i running?"),
            FastQuestionType::OsInfo
        );

        // Distro patterns
        assert_eq!(
            FastQuestionType::classify("what distro is this?"),
            FastQuestionType::OsInfo
        );
        assert_eq!(
            FastQuestionType::classify("which linux distribution?"),
            FastQuestionType::OsInfo
        );

        // Short forms
        assert_eq!(
            FastQuestionType::classify("os?"),
            FastQuestionType::OsInfo
        );
        assert_eq!(
            FastQuestionType::classify("kernel?"),
            FastQuestionType::OsInfo
        );
        assert_eq!(
            FastQuestionType::classify("distro?"),
            FastQuestionType::OsInfo
        );
    }

    // v3.10.0: Uptime classification tests
    #[test]
    fn test_classify_uptime() {
        assert_eq!(
            FastQuestionType::classify("uptime"),
            FastQuestionType::Uptime
        );
        assert_eq!(
            FastQuestionType::classify("uptime?"),
            FastQuestionType::Uptime
        );
    }

    // v3.12.0: GPU classification tests
    #[test]
    fn test_classify_gpu() {
        assert_eq!(
            FastQuestionType::classify("what gpu do I have?"),
            FastQuestionType::GpuInfo
        );
        assert_eq!(
            FastQuestionType::classify("which gpu?"),
            FastQuestionType::GpuInfo
        );
        assert_eq!(
            FastQuestionType::classify("my gpu"),
            FastQuestionType::GpuInfo
        );
        assert_eq!(
            FastQuestionType::classify("show gpu info"),
            FastQuestionType::GpuInfo
        );
        assert_eq!(
            FastQuestionType::classify("what graphics card do I have?"),
            FastQuestionType::GpuInfo
        );
        assert_eq!(
            FastQuestionType::classify("gpu?"),
            FastQuestionType::GpuInfo
        );
        assert_eq!(
            FastQuestionType::classify("gpu"),
            FastQuestionType::GpuInfo
        );
    }

    // v3.12.0: Network classification tests
    #[test]
    fn test_classify_network() {
        assert_eq!(
            FastQuestionType::classify("what are my network interfaces?"),
            FastQuestionType::NetworkInfo
        );
        assert_eq!(
            FastQuestionType::classify("show network interfaces"),
            FastQuestionType::NetworkInfo
        );
        assert_eq!(
            FastQuestionType::classify("my ip address"),
            FastQuestionType::NetworkInfo
        );
        assert_eq!(
            FastQuestionType::classify("what's my ip?"),
            FastQuestionType::NetworkInfo
        );
        assert_eq!(
            FastQuestionType::classify("what is my ip address"),
            FastQuestionType::NetworkInfo
        );
        assert_eq!(
            FastQuestionType::classify("network?"),
            FastQuestionType::NetworkInfo
        );
        assert_eq!(
            FastQuestionType::classify("ip?"),
            FastQuestionType::NetworkInfo
        );
    }
}
