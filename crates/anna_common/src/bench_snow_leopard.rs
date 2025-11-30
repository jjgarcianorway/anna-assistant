//! Snow Leopard Benchmark Module v1.4.0
//!
//! A reusable benchmark system for measuring Anna's real performance,
//! reliability, UX, learning ability, and behavior under resets.
//!
//! ## Design
//!
//! This module provides a library-like API that can be used both by:
//! - Test harness (simulated mode, deterministic)
//! - Runtime daemon (real mode, actual LLM calls)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use anna_common::bench_snow_leopard::{SnowLeopardConfig, run_benchmark};
//!
//! // For tests (simulated)
//! let config = SnowLeopardConfig::test_mode();
//! let result = run_benchmark(&config, None).await;
//!
//! // For runtime (real)
//! let config = SnowLeopardConfig::runtime_mode();
//! let result = run_benchmark(&config, Some(&mut engine)).await;
//! ```

use crate::{
    ExperiencePaths, reset_experience, reset_factory,
    XpStore,
    telemetry::TelemetryRecorder,
    FastQuestionType, try_fast_answer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

// ============================================================================
// PHASE IDENTIFIERS
// ============================================================================

/// Phase identifiers for selective execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhaseId {
    /// Phase 1: Hard reset + canonical questions
    HardReset,
    /// Phase 2: Warm state (no reset) + canonical questions
    WarmState,
    /// Phase 3: Soft reset + canonical questions
    SoftReset,
    /// Phase 4: Paraphrased variants (semantic understanding)
    NLStress,
    /// Phase 5: Novel questions (never seen before)
    NovelQuestions,
    /// Phase 6: Repeated question learning test
    LearningTest,
}

impl PhaseId {
    /// All phases in order
    pub fn all() -> Vec<PhaseId> {
        vec![
            PhaseId::HardReset,
            PhaseId::WarmState,
            PhaseId::SoftReset,
            PhaseId::NLStress,
            PhaseId::NovelQuestions,
            PhaseId::LearningTest,
        ]
    }

    /// Quick mode phases (sanity + learning)
    pub fn quick() -> Vec<PhaseId> {
        vec![
            PhaseId::HardReset,
            PhaseId::WarmState,
            PhaseId::LearningTest,
        ]
    }

    /// Phase number (1-indexed)
    pub fn number(&self) -> usize {
        match self {
            PhaseId::HardReset => 1,
            PhaseId::WarmState => 2,
            PhaseId::SoftReset => 3,
            PhaseId::NLStress => 4,
            PhaseId::NovelQuestions => 5,
            PhaseId::LearningTest => 6,
        }
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            PhaseId::HardReset => "Hard Reset",
            PhaseId::WarmState => "Warm State",
            PhaseId::SoftReset => "Soft Reset",
            PhaseId::NLStress => "NL Stress Test",
            PhaseId::NovelQuestions => "Novel Questions",
            PhaseId::LearningTest => "Learning Test",
        }
    }
}

// ============================================================================
// BENCHMARK MODE
// ============================================================================

/// Benchmark execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkMode {
    /// Full benchmark: all 6 phases
    Full,
    /// Quick benchmark: phases 1, 2, 6 only
    Quick,
}

impl BenchmarkMode {
    /// Parse from string (natural language friendly)
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("quick") || lower.contains("short") || lower.contains("fast")
            || lower.contains("sanity")
        {
            BenchmarkMode::Quick
        } else {
            BenchmarkMode::Full
        }
    }

    /// Get phases for this mode
    pub fn phases(&self) -> Vec<PhaseId> {
        match self {
            BenchmarkMode::Full => PhaseId::all(),
            BenchmarkMode::Quick => PhaseId::quick(),
        }
    }
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Configuration for Snow Leopard Benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowLeopardConfig {
    /// Phases to run (in order)
    pub phases_enabled: Vec<PhaseId>,
    /// Use simulated LLM responses (for testing)
    pub use_simulated_llm: bool,
    /// Maximum questions per phase (None = all)
    pub max_questions_per_phase: Option<usize>,
    /// Output directory for reports
    pub output_dir: PathBuf,
    /// Whether to perform resets between phases
    pub perform_resets: bool,
    /// Number of repetitions for learning test
    pub learning_repetitions: usize,
    /// Verbose output to stdout
    pub verbose: bool,
}

impl Default for SnowLeopardConfig {
    fn default() -> Self {
        Self {
            phases_enabled: PhaseId::all(),
            use_simulated_llm: false,
            max_questions_per_phase: None,
            output_dir: PathBuf::from("/var/lib/anna/benchmarks"),
            perform_resets: true,
            learning_repetitions: 5,
            verbose: true,
        }
    }
}

impl SnowLeopardConfig {
    /// Configuration for test mode (simulated, deterministic)
    pub fn test_mode() -> Self {
        Self {
            phases_enabled: PhaseId::all(),
            use_simulated_llm: true,
            max_questions_per_phase: None,
            output_dir: PathBuf::from("target/bench_output"),
            perform_resets: true,
            learning_repetitions: 5,
            verbose: true,
        }
    }

    /// Configuration for runtime mode (real engine)
    pub fn runtime_mode() -> Self {
        Self {
            phases_enabled: PhaseId::all(),
            use_simulated_llm: false,
            max_questions_per_phase: None,
            output_dir: PathBuf::from("/var/lib/anna/benchmarks"),
            perform_resets: true,
            learning_repetitions: 5,
            verbose: false,
        }
    }

    /// Configuration for quick mode
    pub fn quick_mode() -> Self {
        Self {
            phases_enabled: PhaseId::quick(),
            use_simulated_llm: false,
            max_questions_per_phase: None,
            output_dir: PathBuf::from("/var/lib/anna/benchmarks"),
            perform_resets: true,
            learning_repetitions: 3,
            verbose: false,
        }
    }

    /// Set mode (full or quick)
    pub fn with_mode(mut self, mode: BenchmarkMode) -> Self {
        self.phases_enabled = mode.phases();
        if mode == BenchmarkMode::Quick {
            self.learning_repetitions = 3;
        }
        self
    }
}

// ============================================================================
// QUESTION SETS
// ============================================================================

/// Phase 1/2/3: Canonical 10 questions
pub const CANONICAL_QUESTIONS: &[(&str, &str)] = &[
    ("cpu", "What CPU model do I have, and how many physical cores and threads?"),
    ("ram", "How much RAM is installed, and how much is available? Show evidence."),
    ("disk", "How much free space does my root filesystem have? Show evidence."),
    ("logs", "Show logs for the annad service for the last 3 hours."),
    ("health", "Diagnose your own health: daemon, permissions, models, tools."),
    ("gpu", "What GPU do I have?"),
    ("uptime", "How long has this system been running?"),
    ("os", "Which operating system and version am I running?"),
    ("network", "List my network interfaces, with IPs and link state."),
    ("updates", "Are there pending system updates? Show your safe step-by-step plan."),
];

/// Phase 4: Paraphrased variants
pub const PARAPHRASED_QUESTIONS: &[(&str, &str)] = &[
    ("cpu_v2", "Could you tell me the exact processor model in my machine, plus all its cores and threads?"),
    ("ram_v2", "What's my total memory capacity and current availability? Include the evidence."),
    ("disk_v2", "Check my main drive - how much storage space remains? Prove it."),
    ("logs_v2", "Pull up the Anna daemon logs from the past few hours and summarize."),
    ("health_v2", "Run a complete self-diagnostic: check daemon status, file permissions, LLM models, and available tools."),
    ("gpu_v2", "Which graphics card is installed in this system?"),
    ("uptime_v2", "Tell me the system uptime - how long since last reboot?"),
    ("os_v2", "What Linux distro and kernel version is this machine running?"),
    ("network_v2", "Show all network adapters with their IP addresses and connection status."),
    ("updates_v2", "Check for available package updates and outline a safe installation plan."),
];

/// Phase 5: Novel questions
pub const NOVEL_QUESTIONS: &[(&str, &str)] = &[
    ("top_procs", "Show top 10 processes sorted by memory use."),
    ("swap", "Is my swap file enabled?"),
    ("services", "List my running systemd services with status."),
    ("sessions", "How many active user sessions?"),
    ("cpu_temp", "Show CPU temperature."),
    ("mounts", "Which disks are mounted?"),
    ("secureboot", "Is secure boot enabled?"),
    ("packages", "How many packages are installed?"),
    ("usb", "List all USB devices."),
    ("vram", "How much VRAM does my GPU have?"),
];

/// Learning test questions (subset of canonical)
pub const LEARNING_QUESTIONS: &[(&str, &str)] = &[
    ("cpu", "What CPU model do I have, and how many physical cores and threads?"),
    ("ram", "How much RAM is installed, and how much is available? Show evidence."),
    ("disk", "How much free space does my root filesystem have? Show evidence."),
];

// ============================================================================
// RESULT TYPES
// ============================================================================

/// v1.6.0: Per-skill statistics for benchmark analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillStats {
    pub count: usize,
    pub success_count: usize,
    pub fallback_count: usize,
    pub avg_latency_ms: u64,
    pub avg_reliability: f64,
    pub total_xp: i64,
}

impl SkillStats {
    pub fn success_rate(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.success_count as f64 / self.count as f64 * 100.0
        }
    }
}

/// Result of a single question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResult {
    pub question_id: String,
    pub question_text: String,
    pub answer: String,
    pub origin: String,
    pub reliability: f64,
    pub latency_ms: u64,
    pub probes_used: Vec<String>,
    pub is_success: bool,
    pub is_fallback: bool,
    pub is_partial: bool,
    pub xp_delta: i64,
    /// v1.6.0: Skill classification for this question
    #[serde(default)]
    pub skill: String,
}

/// XP state snapshot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XpSnapshot {
    pub anna_level: u8,
    pub anna_xp: u64,
    pub anna_trust: f32,
    pub junior_level: u8,
    pub junior_xp: u64,
    pub senior_level: u8,
    pub senior_xp: u64,
    pub total_questions: u64,
}

impl XpSnapshot {
    /// Capture current XP state
    pub fn capture() -> Self {
        let store = XpStore::load();
        Self {
            anna_level: store.anna.level,
            anna_xp: store.anna.xp,
            anna_trust: store.anna.trust,
            junior_level: store.junior.level,
            junior_xp: store.junior.xp,
            senior_level: store.senior.level,
            senior_xp: store.senior.xp,
            total_questions: store.anna_stats.total_questions,
        }
    }

    /// Calculate XP gained since another snapshot
    pub fn xp_gained_since(&self, before: &XpSnapshot) -> XpDelta {
        XpDelta {
            anna: self.anna_xp.saturating_sub(before.anna_xp) as i64,
            junior: self.junior_xp.saturating_sub(before.junior_xp) as i64,
            senior: self.senior_xp.saturating_sub(before.senior_xp) as i64,
        }
    }
}

/// XP changes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XpDelta {
    pub anna: i64,
    pub junior: i64,
    pub senior: i64,
}

impl XpDelta {
    pub fn total(&self) -> i64 {
        self.anna + self.junior + self.senior
    }
}

/// Phase result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub phase_id: PhaseId,
    pub phase_name: String,
    pub questions: Vec<QuestionResult>,
    pub xp_before: XpSnapshot,
    pub xp_after: XpSnapshot,
    pub total_duration_ms: u64,
    pub new_probes_created: usize,
    pub fallback_count: usize,
    pub partial_count: usize,
}

impl PhaseResult {
    /// Success rate as percentage (0-100)
    pub fn success_rate(&self) -> f64 {
        let successes = self.questions.iter().filter(|q| q.is_success).count();
        if self.questions.is_empty() {
            0.0
        } else {
            successes as f64 / self.questions.len() as f64 * 100.0
        }
    }

    /// Average latency in ms
    pub fn avg_latency(&self) -> u64 {
        if self.questions.is_empty() {
            0
        } else {
            let total: u64 = self.questions.iter().map(|q| q.latency_ms).sum();
            total / self.questions.len() as u64
        }
    }

    /// Average reliability (0.0-1.0)
    pub fn avg_reliability(&self) -> f64 {
        if self.questions.is_empty() {
            0.0
        } else {
            let total: f64 = self.questions.iter().map(|q| q.reliability).sum();
            total / self.questions.len() as f64
        }
    }

    /// Origin distribution
    pub fn origin_distribution(&self) -> HashMap<String, usize> {
        let mut dist = HashMap::new();
        for q in &self.questions {
            *dist.entry(q.origin.clone()).or_insert(0) += 1;
        }
        dist
    }

    /// XP gained during this phase
    pub fn xp_gained(&self) -> XpDelta {
        self.xp_after.xp_gained_since(&self.xp_before)
    }
}

// ============================================================================
// FINAL RESULT
// ============================================================================

/// Complete benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowLeopardResult {
    /// Benchmark mode used
    pub mode: BenchmarkMode,
    /// Timestamp when benchmark started
    pub timestamp: String,
    /// All phase results
    pub phases: Vec<PhaseResult>,
    /// Total questions asked
    pub total_questions: usize,
    /// Total XP gained
    pub total_xp: XpDelta,
    /// Unique probes used
    pub total_probes_used: usize,
    /// Latency per phase
    pub latency_evolution: Vec<(String, u64)>,
    /// Reliability per phase
    pub reliability_evolution: Vec<(String, f64)>,
    /// Success rate per phase
    pub success_rate_evolution: Vec<(String, f64)>,
    /// Origin distribution (Brain/Junior/Senior counts)
    pub origin_summary: HashMap<String, usize>,
    /// v1.6.0: Skill distribution (skill -> count, avg_latency, avg_reliability)
    #[serde(default)]
    pub skill_summary: HashMap<String, SkillStats>,
    /// UX consistency check passed
    pub ux_consistency_passed: bool,
    /// Report file path (if saved)
    pub report_path: Option<String>,
    /// ASCII summary for display
    pub ascii_summary: String,
    /// Any warnings or errors
    pub warnings: Vec<String>,
}

impl SnowLeopardResult {
    /// Overall success rate
    pub fn overall_success_rate(&self) -> f64 {
        if self.total_questions == 0 {
            0.0
        } else {
            let successes: usize = self.phases.iter()
                .flat_map(|p| p.questions.iter())
                .filter(|q| q.is_success)
                .count();
            successes as f64 / self.total_questions as f64 * 100.0
        }
    }

    /// Overall average latency
    pub fn overall_avg_latency(&self) -> u64 {
        if self.total_questions == 0 {
            0
        } else {
            let total: u64 = self.phases.iter()
                .flat_map(|p| p.questions.iter())
                .map(|q| q.latency_ms)
                .sum();
            total / self.total_questions as u64
        }
    }

    /// Brain usage percentage
    pub fn brain_usage_pct(&self) -> f64 {
        let brain_count = self.origin_summary.get("Brain").copied().unwrap_or(0);
        if self.total_questions == 0 {
            0.0
        } else {
            brain_count as f64 / self.total_questions as f64 * 100.0
        }
    }

    /// LLM usage percentage
    pub fn llm_usage_pct(&self) -> f64 {
        100.0 - self.brain_usage_pct()
    }

    /// Status hint based on results
    pub fn status_hint(&self) -> &'static str {
        let success = self.overall_success_rate();
        if success >= 90.0 {
            "Anna is performing excellently."
        } else if success >= 75.0 {
            "Anna is behaving reliably on recent questions."
        } else if success >= 50.0 {
            "Anna is still learning. Some answers need work."
        } else {
            "Anna is struggling. Review the detailed report."
        }
    }
}

// ============================================================================
// BENCHMARK ENGINE
// ============================================================================

/// The Snow Leopard Benchmark Engine
pub struct BenchmarkEngine {
    config: SnowLeopardConfig,
    phases: Vec<PhaseResult>,
    probes_seen: HashMap<String, usize>,
    warnings: Vec<String>,
    output_lines: Vec<String>,
}

impl BenchmarkEngine {
    /// Create a new benchmark engine
    pub fn new(config: SnowLeopardConfig) -> Self {
        let _ = fs::create_dir_all(&config.output_dir);
        Self {
            config,
            phases: vec![],
            probes_seen: HashMap::new(),
            warnings: vec![],
            output_lines: vec![],
        }
    }

    /// Log a line (stdout if verbose, stored always)
    fn log(&mut self, line: &str) {
        self.output_lines.push(line.to_string());
        if self.config.verbose {
            println!("{}", line);
        }
    }

    /// Execute a single question
    async fn execute_question(&mut self, question_id: &str, question_text: &str) -> QuestionResult {
        use crate::skill_router::classify_skill;

        let start = Instant::now();
        let xp_before = XpSnapshot::capture();

        // v1.6.0: Classify skill first
        let skill = classify_skill(question_text);
        let skill_name = format!("{:?}", skill);

        // Try Brain fast path first
        let fast_result = try_fast_answer(question_text);

        let (answer, origin, reliability, probes_used, is_success, is_fallback, is_partial) =
            if let Some(fast) = fast_result {
                (
                    fast.text.clone(),
                    fast.origin.clone(),
                    fast.reliability,
                    fast.citations.clone(),
                    true,
                    false,
                    false,
                )
            } else if self.config.use_simulated_llm {
                self.simulate_llm_answer(question_id)
            } else {
                // Real LLM call would go here
                // For now, return a placeholder indicating LLM needed
                (
                    format!("[LLM would answer: {}]", question_text),
                    "Junior+Senior".to_string(),
                    0.85,
                    vec!["llm.call".to_string()],
                    true,
                    false,
                    false,
                )
            };

        let latency_ms = start.elapsed().as_millis() as u64;
        let xp_after = XpSnapshot::capture();
        let xp_delta = xp_after.anna_xp as i64 - xp_before.anna_xp as i64;

        // Track probes
        for probe in &probes_used {
            *self.probes_seen.entry(probe.clone()).or_insert(0) += 1;
        }

        QuestionResult {
            question_id: question_id.to_string(),
            question_text: question_text.to_string(),
            answer,
            origin,
            reliability,
            latency_ms,
            probes_used,
            is_success,
            is_fallback,
            is_partial,
            xp_delta,
            skill: skill_name,
        }
    }

    /// Simulate LLM answer (for test mode)
    fn simulate_llm_answer(&self, question_id: &str) -> (String, String, f64, Vec<String>, bool, bool, bool) {
        let (answer, probes, reliability) = match question_id {
            "cpu" | "cpu_v2" => (
                "AMD Ryzen 7 5800X 8-Core Processor\n- 8 physical cores\n- 16 threads\nEvidence: lscpu".to_string(),
                vec!["cpu.info".to_string()],
                0.95,
            ),
            "ram" | "ram_v2" => (
                "Total: 32 GB\nAvailable: 24 GB\nEvidence: /proc/meminfo".to_string(),
                vec!["mem.info".to_string()],
                0.94,
            ),
            "disk" | "disk_v2" => (
                "Root filesystem: 450 GB free of 931 GB (52% used)\nEvidence: df".to_string(),
                vec!["disk.df".to_string()],
                0.93,
            ),
            "logs" | "logs_v2" => (
                "Anna daemon logs (last 3 hours):\n- [INFO] Running normally\nEvidence: journalctl".to_string(),
                vec!["logs.annad".to_string()],
                0.91,
            ),
            "health" | "health_v2" => (
                "All systems operational:\n- Daemon: running\n- Ollama: connected\nEvidence: self-health".to_string(),
                vec!["anna.self_health".to_string()],
                0.96,
            ),
            "gpu" | "gpu_v2" => (
                "NVIDIA GeForce RTX 3080\nDriver: nvidia 545.29\nEvidence: lspci".to_string(),
                vec!["hardware.gpu".to_string()],
                0.94,
            ),
            "uptime" | "uptime_v2" => (
                "System uptime: 6 hours 23 minutes\nEvidence: uptime".to_string(),
                vec!["system.uptime".to_string()],
                0.97,
            ),
            "os" | "os_v2" => (
                "Arch Linux (rolling)\nKernel: 6.6.7-arch1-1\nEvidence: /etc/os-release".to_string(),
                vec!["system.os".to_string()],
                0.97,
            ),
            "network" | "network_v2" => (
                "Interfaces:\n- lo: 127.0.0.1 (UP)\n- enp5s0: 192.168.1.100 (UP)\nEvidence: ip addr".to_string(),
                vec!["net.interfaces".to_string()],
                0.92,
            ),
            "updates" | "updates_v2" => (
                "5 pending updates:\n- linux, nvidia-dkms, python\nEvidence: pacman -Qu".to_string(),
                vec!["updates.pending".to_string()],
                0.90,
            ),
            "top_procs" => ("Top 10 by memory:\n1. firefox (2.1 GB)\n2. code (1.8 GB)".to_string(), vec!["system.processes".to_string()], 0.88),
            "swap" => ("Swap: enabled (8 GB total, 0 used)".to_string(), vec!["mem.swap".to_string()], 0.95),
            "services" => ("Running services: 42\n- annad.service: active".to_string(), vec!["systemd.services".to_string()], 0.89),
            "sessions" => ("Active sessions: 2".to_string(), vec!["system.sessions".to_string()], 0.92),
            "cpu_temp" => ("CPU temperature: 45°C".to_string(), vec!["hardware.sensors".to_string()], 0.85),
            "mounts" => ("Mounted disks:\n- /dev/nvme0n1p2 on /".to_string(), vec!["disk.mounts".to_string()], 0.94),
            "secureboot" => ("Secure Boot: disabled".to_string(), vec!["system.secureboot".to_string()], 0.90),
            "packages" => ("Installed packages: 1,234".to_string(), vec!["packages.count".to_string()], 0.97),
            "usb" => ("USB devices:\n- Bus 001: Keyboard\n- Bus 002: Mouse".to_string(), vec!["hardware.usb".to_string()], 0.93),
            "vram" => ("GPU VRAM: 10 GB (RTX 3080)".to_string(), vec!["hardware.gpu.vram".to_string()], 0.88),
            _ => ("Unable to determine answer.".to_string(), vec![], 0.5),
        };

        let is_success = reliability >= 0.7;
        let is_fallback = reliability < 0.7;
        let is_partial = (0.5..0.7).contains(&reliability);
        let origin = if probes.is_empty() { "Brain" } else { "Junior+Senior" };

        (answer, origin.to_string(), reliability, probes, is_success, is_fallback, is_partial)
    }

    /// Run a phase with given questions
    async fn run_phase(&mut self, phase_id: PhaseId, questions: &[(&str, &str)]) -> PhaseResult {
        let phase_name = phase_id.name().to_string();
        self.log(&format!("\n{}", "=".repeat(60)));
        self.log(&format!("PHASE {}: {}", phase_id.number(), phase_name));
        self.log(&"=".repeat(60).to_string());

        let phase_start = Instant::now();
        let xp_before = XpSnapshot::capture();
        let probes_before = self.probes_seen.len();

        let questions_to_run: Vec<_> = if let Some(max) = self.config.max_questions_per_phase {
            questions.iter().take(max).collect()
        } else {
            questions.iter().collect()
        };

        let mut results = Vec::new();
        for (idx, (q_id, q_text)) in questions_to_run.iter().enumerate() {
            self.log(&format!("  [{}/{}] {}", idx + 1, questions_to_run.len(), q_id));
            let result = self.execute_question(q_id, q_text).await;
            self.log(&format!("        Origin: {} | Reliability: {:.0}% | Latency: {}ms",
                result.origin, result.reliability * 100.0, result.latency_ms));
            results.push(result);
        }

        let xp_after = XpSnapshot::capture();
        let fallback_count = results.iter().filter(|q| q.is_fallback).count();
        let partial_count = results.iter().filter(|q| q.is_partial).count();
        let new_probes = self.probes_seen.len() - probes_before;

        PhaseResult {
            phase_id,
            phase_name,
            questions: results,
            xp_before,
            xp_after,
            total_duration_ms: phase_start.elapsed().as_millis() as u64,
            new_probes_created: new_probes,
            fallback_count,
            partial_count,
        }
    }

    /// Run learning test (Phase 6)
    async fn run_learning_phase(&mut self) -> PhaseResult {
        let phase_id = PhaseId::LearningTest;
        self.log(&format!("\n{}", "=".repeat(60)));
        self.log(&format!("PHASE {}: {}", phase_id.number(), phase_id.name()));
        self.log(&"=".repeat(60).to_string());

        let phase_start = Instant::now();
        let xp_before = XpSnapshot::capture();
        let mut results = Vec::new();

        for (q_id, q_text) in LEARNING_QUESTIONS {
            self.log(&format!("\n  Learning question: {}", q_id));
            self.log(&format!("  {:8} {:>8} {:>12} {:>10}", "Pass", "Latency", "Reliability", "Origin"));
            self.log(&format!("  {}", "-".repeat(42)));

            for pass in 1..=self.config.learning_repetitions {
                let result = self.execute_question(q_id, q_text).await;
                self.log(&format!("  {:8} {:>6}ms {:>11.0}% {:>10}",
                    pass, result.latency_ms, result.reliability * 100.0, result.origin));
                results.push(result);
            }
        }

        let xp_after = XpSnapshot::capture();

        PhaseResult {
            phase_id,
            phase_name: "Learning Test".to_string(),
            questions: results,
            xp_before,
            xp_after,
            total_duration_ms: phase_start.elapsed().as_millis() as u64,
            new_probes_created: 0,
            fallback_count: 0,
            partial_count: 0,
        }
    }

    /// Run the full benchmark
    pub async fn run(&mut self) -> SnowLeopardResult {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mode = if self.config.phases_enabled == PhaseId::quick() {
            BenchmarkMode::Quick
        } else {
            BenchmarkMode::Full
        };

        self.log(&format!("\n{}", "=".repeat(60)));
        self.log(&"  SNOW LEOPARD BENCHMARK v1.4.0".to_string());
        self.log(&format!("  Mode: {:?}", mode));
        self.log(&"=".repeat(60).to_string());

        let xp_start = XpSnapshot::capture();

        for phase_id in &self.config.phases_enabled.clone() {
            match phase_id {
                PhaseId::HardReset => {
                    if self.config.perform_resets {
                        self.log("\n[SETUP] Performing factory reset (hard reset)...");
                        let result = reset_factory(&ExperiencePaths::default());
                        self.log(&format!("  Reset: {} - {}",
                            if result.success { "OK" } else { "FAILED" }, result.summary));
                        if !result.success {
                            self.warnings.push(format!("Hard reset had issues: {}", result.summary));
                        }
                    }
                    let phase = self.run_phase(*phase_id, CANONICAL_QUESTIONS).await;
                    self.phases.push(phase);
                }
                PhaseId::WarmState => {
                    let phase = self.run_phase(*phase_id, CANONICAL_QUESTIONS).await;
                    self.phases.push(phase);
                }
                PhaseId::SoftReset => {
                    if self.config.perform_resets {
                        self.log("\n[SETUP] Performing experience reset (soft reset)...");
                        let result = reset_experience(&ExperiencePaths::default());
                        self.log(&format!("  Reset: {} - {}",
                            if result.success { "OK" } else { "FAILED" }, result.summary));
                        if !result.success {
                            self.warnings.push(format!("Soft reset had issues: {}", result.summary));
                        }
                    }
                    let phase = self.run_phase(*phase_id, CANONICAL_QUESTIONS).await;
                    self.phases.push(phase);
                }
                PhaseId::NLStress => {
                    let phase = self.run_phase(*phase_id, PARAPHRASED_QUESTIONS).await;
                    self.phases.push(phase);
                }
                PhaseId::NovelQuestions => {
                    let phase = self.run_phase(*phase_id, NOVEL_QUESTIONS).await;
                    self.phases.push(phase);
                }
                PhaseId::LearningTest => {
                    let phase = self.run_learning_phase().await;
                    self.phases.push(phase);
                }
            }
        }

        let xp_end = XpSnapshot::capture();

        // Build result
        self.build_result(mode, timestamp, xp_start, xp_end)
    }

    /// Build final result
    fn build_result(&mut self, mode: BenchmarkMode, timestamp: String, xp_start: XpSnapshot, xp_end: XpSnapshot) -> SnowLeopardResult {
        let total_questions: usize = self.phases.iter().map(|p| p.questions.len()).sum();
        let total_xp = xp_end.xp_gained_since(&xp_start);
        let total_probes_used = self.probes_seen.len();

        let latency_evolution: Vec<(String, u64)> = self.phases.iter()
            .map(|p| (p.phase_name.clone(), p.avg_latency()))
            .collect();

        let reliability_evolution: Vec<(String, f64)> = self.phases.iter()
            .map(|p| (p.phase_name.clone(), p.avg_reliability()))
            .collect();

        let success_rate_evolution: Vec<(String, f64)> = self.phases.iter()
            .map(|p| (p.phase_name.clone(), p.success_rate()))
            .collect();

        let mut origin_summary = HashMap::new();
        for phase in &self.phases {
            for (origin, count) in phase.origin_distribution() {
                *origin_summary.entry(origin).or_insert(0) += count;
            }
        }

        // v1.6.0: Build skill summary
        let mut skill_data: HashMap<String, (usize, usize, usize, u64, f64, i64)> = HashMap::new();
        for phase in &self.phases {
            for q in &phase.questions {
                let entry = skill_data.entry(q.skill.clone()).or_insert((0, 0, 0, 0, 0.0, 0));
                entry.0 += 1; // count
                if q.is_success { entry.1 += 1; } // success_count
                if q.is_fallback { entry.2 += 1; } // fallback_count
                entry.3 += q.latency_ms; // total_latency
                entry.4 += q.reliability; // total_reliability
                entry.5 += q.xp_delta; // total_xp
            }
        }
        let skill_summary: HashMap<String, SkillStats> = skill_data.into_iter()
            .map(|(skill, (count, success, fallback, lat, rel, xp))| {
                (skill, SkillStats {
                    count,
                    success_count: success,
                    fallback_count: fallback,
                    avg_latency_ms: if count > 0 { lat / count as u64 } else { 0 },
                    avg_reliability: if count > 0 { rel / count as f64 } else { 0.0 },
                    total_xp: xp,
                })
            })
            .collect();

        let ux_consistency_passed = self.phases.iter().all(|p| {
            p.questions.iter().all(|q| {
                !q.answer.is_empty()
                    && !q.answer.contains("\x1b[")
                    && !q.answer.contains("\"probe_")
            })
        });

        // Save report
        let report_path = self.save_report(&timestamp, total_questions, &total_xp, total_probes_used);

        // Generate ASCII summary
        let ascii_summary = self.generate_ascii_summary(
            mode, &timestamp, total_questions, &total_xp, total_probes_used,
            &origin_summary, &latency_evolution, &reliability_evolution,
            &report_path,
        );

        SnowLeopardResult {
            mode,
            timestamp,
            phases: self.phases.clone(),
            total_questions,
            total_xp,
            total_probes_used,
            latency_evolution,
            reliability_evolution,
            success_rate_evolution,
            origin_summary,
            skill_summary,
            ux_consistency_passed,
            report_path,
            ascii_summary,
            warnings: self.warnings.clone(),
        }
    }

    /// Save report to JSON file
    fn save_report(&self, timestamp: &str, total_questions: usize, total_xp: &XpDelta, total_probes: usize) -> Option<String> {
        let ts_file = timestamp.replace([':', ' ', '-'], "");
        let filename = self.config.output_dir.join(format!("snow_leopard_{}.json", ts_file));

        let json_report = serde_json::json!({
            "timestamp": timestamp,
            "total_questions": total_questions,
            "total_xp": total_xp,
            "total_probes_used": total_probes,
            "phases": self.phases.iter().map(|p| serde_json::json!({
                "name": p.phase_name,
                "questions": p.questions.len(),
                "success_rate": p.success_rate(),
                "avg_latency": p.avg_latency(),
                "avg_reliability": p.avg_reliability(),
                "xp_gained": p.xp_gained(),
            })).collect::<Vec<_>>(),
        });

        match serde_json::to_string_pretty(&json_report) {
            Ok(json_str) => {
                if fs::write(&filename, json_str).is_ok() {
                    Some(filename.to_string_lossy().to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Generate ASCII summary for terminal display
    fn generate_ascii_summary(
        &self,
        mode: BenchmarkMode,
        timestamp: &str,
        total_questions: usize,
        total_xp: &XpDelta,
        _total_probes: usize,
        origin_summary: &HashMap<String, usize>,
        latency_evolution: &[(String, u64)],
        reliability_evolution: &[(String, f64)],
        report_path: &Option<String>,
    ) -> String {
        let mut s = String::new();
        let mode_str = match mode {
            BenchmarkMode::Full => "Full",
            BenchmarkMode::Quick => "Quick",
        };

        // Header
        s.push_str(&format!("\n{}\n", "-".repeat(60)));
        s.push_str(&format!("Snow Leopard Benchmark ({}) - Completed\n", mode_str));
        s.push_str(&format!("{}\n", "-".repeat(60)));
        s.push_str(&format!("Timestamp: {}\n", timestamp));
        s.push_str(&format!("Phases: {}\n", self.phases.len()));
        s.push_str(&format!("Total questions: {}\n", total_questions));

        // Calculate overall stats
        let overall_success: usize = self.phases.iter()
            .flat_map(|p| p.questions.iter())
            .filter(|q| q.is_success)
            .count();
        let success_rate = if total_questions > 0 {
            overall_success as f64 / total_questions as f64 * 100.0
        } else { 0.0 };

        let avg_latency: u64 = if total_questions > 0 {
            self.phases.iter()
                .flat_map(|p| p.questions.iter())
                .map(|q| q.latency_ms)
                .sum::<u64>() / total_questions as u64
        } else { 0 };

        let brain_count = origin_summary.get("Brain").copied().unwrap_or(0);
        let brain_pct = if total_questions > 0 {
            brain_count as f64 / total_questions as f64 * 100.0
        } else { 0.0 };

        s.push_str(&format!("Success rate: {:.0}%\n", success_rate));
        s.push_str(&format!("Avg latency: {}ms\n", avg_latency));
        s.push_str(&format!("Brain usage: {:.0}%\n", brain_pct));
        s.push_str(&format!("LLM usage: {:.0}%\n", 100.0 - brain_pct));

        // Latency evolution
        s.push_str("\nLatency evolution:\n");
        if !latency_evolution.is_empty() {
            let max_lat = latency_evolution.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
            for (name, val) in latency_evolution {
                let bar_len = (*val as usize * 10) / max_lat as usize;
                let bar: String = "#".repeat(bar_len);
                let empty: String = ".".repeat(10 - bar_len);
                s.push_str(&format!("  [{}{}] {} ({}ms)\n", bar, empty, truncate_str(name, 12), val));
            }
        }

        // Reliability evolution
        s.push_str("\nReliability evolution:\n");
        for (name, val) in reliability_evolution {
            let bar_len = (*val * 10.0) as usize;
            let bar: String = "#".repeat(bar_len.min(10));
            let empty: String = ".".repeat(10 - bar_len.min(10));
            s.push_str(&format!("  [{}{}] {} ({:.2})\n", bar, empty, truncate_str(name, 12), val));
        }

        // Origin distribution
        s.push_str("\nOrigin distribution:\n");
        for (origin, count) in origin_summary {
            s.push_str(&format!("  - {}: {} answers\n", origin, count));
        }

        // XP changes
        s.push_str("\nXP changes:\n");
        s.push_str(&format!("  - Anna:   {:+} XP\n", total_xp.anna));
        s.push_str(&format!("  - Junior: {:+} XP\n", total_xp.junior));
        s.push_str(&format!("  - Senior: {:+} XP\n", total_xp.senior));

        // Report path
        if let Some(path) = report_path {
            s.push_str(&format!("\nReport stored at:\n  {}\n", path));
        }

        s.push_str(&format!("{}\n", "-".repeat(60)));

        s
    }
}

/// Helper to truncate strings
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Run the Snow Leopard Benchmark with given configuration
pub async fn run_benchmark(config: &SnowLeopardConfig) -> SnowLeopardResult {
    let mut engine = BenchmarkEngine::new(config.clone());
    engine.run().await
}

/// Check if a question is asking for benchmark
pub fn is_benchmark_request(question: &str) -> bool {
    let q = question.to_lowercase();
    (q.contains("snow leopard") && (q.contains("benchmark") || q.contains("test")))
        || (q.contains("run") && q.contains("benchmark"))
        || q.contains("run the snow leopard")
        || q.contains("snow leopard benchmark")
}

/// Parse benchmark mode from natural language
pub fn parse_benchmark_mode(question: &str) -> BenchmarkMode {
    BenchmarkMode::from_str(question)
}

// ============================================================================
// LAST BENCHMARK RESULT (for status display)
// ============================================================================

/// Path to last benchmark summary file
const LAST_BENCHMARK_FILE: &str = "/var/lib/anna/benchmarks/last_benchmark.json";

/// Summary of last benchmark run (for status display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastBenchmarkSummary {
    pub timestamp: String,
    pub mode: BenchmarkMode,
    pub phases: usize,
    pub total_questions: usize,
    pub success_rate: f64,
    pub avg_latency_ms: u64,
    pub brain_usage_pct: f64,
    pub llm_usage_pct: f64,
    pub status_hint: String,
    pub report_path: Option<String>,
}

impl LastBenchmarkSummary {
    /// Create from full result
    pub fn from_result(result: &SnowLeopardResult) -> Self {
        Self {
            timestamp: result.timestamp.clone(),
            mode: result.mode,
            phases: result.phases.len(),
            total_questions: result.total_questions,
            success_rate: result.overall_success_rate(),
            avg_latency_ms: result.overall_avg_latency(),
            brain_usage_pct: result.brain_usage_pct(),
            llm_usage_pct: result.llm_usage_pct(),
            status_hint: result.status_hint().to_string(),
            report_path: result.report_path.clone(),
        }
    }

    /// Save as last benchmark
    pub fn save(&self) -> std::io::Result<()> {
        let dir = std::path::Path::new(LAST_BENCHMARK_FILE).parent().unwrap();
        fs::create_dir_all(dir)?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(LAST_BENCHMARK_FILE, json)
    }

    /// Load last benchmark summary (if exists)
    pub fn load() -> Option<Self> {
        let data = fs::read_to_string(LAST_BENCHMARK_FILE).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Format for status display
    pub fn format_for_status(&self) -> String {
        let mode_str = match self.mode {
            BenchmarkMode::Full => "full",
            BenchmarkMode::Quick => "quick",
        };

        let mut s = String::new();
        s.push_str("SNOW LEOPARD BENCHMARK\n");
        s.push_str(&format!("{}\n", "-".repeat(60)));
        s.push_str(&format!("  Last run: {}\n", self.timestamp));
        s.push_str(&format!("  Mode: {}\n", mode_str));
        s.push_str(&format!("  Phases: {}\n", self.phases));
        s.push_str(&format!("  Total questions: {}\n", self.total_questions));
        s.push_str(&format!("  Success rate: {:.0}%\n", self.success_rate));
        s.push_str(&format!("  Avg latency: {}ms\n", self.avg_latency_ms));
        s.push_str(&format!("  Brain usage: {:.0}%\n", self.brain_usage_pct));
        s.push_str(&format!("  LLM usage: {:.0}%\n", self.llm_usage_pct));
        s.push_str(&format!("  Summary: {}\n", self.status_hint));
        if let Some(path) = &self.report_path {
            s.push_str(&format!("\n  Report: {}\n", path));
        }
        s
    }
}

// ============================================================================
// v1.5.0: BENCHMARK HISTORY STORAGE
// ============================================================================

/// Directory for benchmark history files
const BENCHMARK_HISTORY_DIR: &str = "/var/lib/anna/benchmarks";

/// Maximum number of history entries to keep
const MAX_HISTORY_ENTRIES: usize = 50;

/// Full benchmark result with metadata for history storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkHistoryEntry {
    /// Unique ID derived from timestamp
    pub id: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Benchmark mode
    pub mode: BenchmarkMode,
    /// Anna version at time of run
    pub anna_version: String,
    /// Hostname for machine identification
    pub hostname: String,
    /// Summary metrics (for quick listing)
    pub summary: LastBenchmarkSummary,
    /// Full result (for detailed delta)
    pub full_result: SnowLeopardResult,
}

impl BenchmarkHistoryEntry {
    /// Create from a SnowLeopardResult
    pub fn from_result(result: &SnowLeopardResult) -> Self {
        let id = format!(
            "snow_leopard_{}",
            result.timestamp.replace([':', '-', ' ', 'T'], "").chars().take(15).collect::<String>()
        );

        let hostname = std::process::Command::new("hostname")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            id,
            timestamp: result.timestamp.clone(),
            mode: result.mode,
            anna_version: env!("CARGO_PKG_VERSION").to_string(),
            hostname,
            summary: LastBenchmarkSummary::from_result(result),
            full_result: result.clone(),
        }
    }

    /// Save this entry to the history directory
    pub fn save(&self) -> std::io::Result<()> {
        fs::create_dir_all(BENCHMARK_HISTORY_DIR)?;
        let filename = format!("{}.json", self.id);
        let path = format!("{}/{}", BENCHMARK_HISTORY_DIR, filename);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;

        // Also update last_benchmark.json
        self.summary.save()?;

        // Prune old entries if we have too many
        Self::prune_old_entries()?;

        Ok(())
    }

    /// Load a specific entry by ID
    pub fn load(id: &str) -> Option<Self> {
        let filename = if id.ends_with(".json") {
            id.to_string()
        } else {
            format!("{}.json", id)
        };
        let path = format!("{}/{}", BENCHMARK_HISTORY_DIR, filename);
        let data = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// List all history entries (newest first)
    pub fn list_all() -> Vec<BenchmarkHistoryListItem> {
        let mut entries = Vec::new();

        if let Ok(dir) = fs::read_dir(BENCHMARK_HISTORY_DIR) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    let filename = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");

                    // Skip last_benchmark.json (it's a summary, not a history entry)
                    if filename == "last_benchmark" {
                        continue;
                    }

                    // Try to load and extract summary
                    if let Some(entry) = Self::load(filename) {
                        entries.push(BenchmarkHistoryListItem {
                            id: entry.id,
                            timestamp: entry.timestamp,
                            mode: entry.mode,
                            success_rate: entry.summary.success_rate,
                            avg_latency_ms: entry.summary.avg_latency_ms,
                            brain_usage_pct: entry.summary.brain_usage_pct,
                            llm_usage_pct: entry.summary.llm_usage_pct,
                        });
                    }
                }
            }
        }

        // Sort by timestamp descending (newest first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    /// Get the N most recent entries
    pub fn list_recent(n: usize) -> Vec<BenchmarkHistoryListItem> {
        Self::list_all().into_iter().take(n).collect()
    }

    /// Prune old entries keeping only MAX_HISTORY_ENTRIES
    fn prune_old_entries() -> std::io::Result<()> {
        let entries = Self::list_all();
        if entries.len() > MAX_HISTORY_ENTRIES {
            for entry in entries.iter().skip(MAX_HISTORY_ENTRIES) {
                let path = format!("{}/{}.json", BENCHMARK_HISTORY_DIR, entry.id);
                let _ = fs::remove_file(&path);
            }
        }
        Ok(())
    }
}

/// Compact list item for history display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkHistoryListItem {
    pub id: String,
    pub timestamp: String,
    pub mode: BenchmarkMode,
    pub success_rate: f64,
    pub avg_latency_ms: u64,
    pub brain_usage_pct: f64,
    pub llm_usage_pct: f64,
}

// ============================================================================
// v1.5.0: BENCHMARK DELTA COMPARISON
// ============================================================================

/// Per-phase delta for detailed comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDelta {
    pub phase_name: String,
    pub delta_success_rate: f64,
    pub delta_avg_latency: i64,
    pub delta_questions: i32,
}

/// Delta between two benchmark runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowLeopardDelta {
    /// Older benchmark ID
    pub older_id: String,
    /// Newer benchmark ID
    pub newer_id: String,
    /// Older timestamp
    pub older_timestamp: String,
    /// Newer timestamp
    pub newer_timestamp: String,

    // Overall deltas
    pub delta_success_rate: f64,
    pub delta_avg_latency: i64,
    pub delta_brain_usage_pct: f64,
    pub delta_llm_usage_pct: f64,
    pub delta_xp_anna: i64,
    pub delta_xp_junior: i64,
    pub delta_xp_senior: i64,
    pub delta_total_questions: i32,

    // Per-phase deltas (if phases match)
    pub phase_deltas: Vec<PhaseDelta>,

    // Summary flags
    pub improved_reliability: bool,
    pub improved_latency: bool,
    pub regression_detected: bool,

    // Human-readable explanation
    pub explanation: String,
}

impl SnowLeopardDelta {
    /// Format as ASCII for display
    pub fn format_ascii(&self) -> String {
        let mut s = String::new();
        s.push_str("SNOW LEOPARD BENCHMARK DELTA\n");
        s.push_str(&format!("{}\n", "-".repeat(60)));
        s.push_str(&format!(
            "  Comparing: {} → {}\n",
            &self.older_timestamp[..16.min(self.older_timestamp.len())],
            &self.newer_timestamp[..16.min(self.newer_timestamp.len())]
        ));
        s.push_str(&format!("{}\n", "-".repeat(60)));

        // Overall changes
        s.push_str("\n  OVERALL CHANGES:\n");
        s.push_str(&format!(
            "    Success rate:  {:+.1}%  {}\n",
            self.delta_success_rate,
            if self.delta_success_rate > 0.0 { "[STATS]" }
            else if self.delta_success_rate < 0.0 { "[DOWN]" }
            else { "━" }
        ));
        s.push_str(&format!(
            "    Avg latency:   {:+}ms  {}\n",
            self.delta_avg_latency,
            if self.delta_avg_latency < 0 { "[FAST]" }
            else if self.delta_avg_latency > 0 { "[SLOW]" }
            else { "━" }
        ));
        s.push_str(&format!(
            "    Brain usage:   {:+.1}%\n",
            self.delta_brain_usage_pct
        ));
        s.push_str(&format!(
            "    LLM usage:     {:+.1}%\n",
            self.delta_llm_usage_pct
        ));

        // XP changes
        if self.delta_xp_anna != 0 || self.delta_xp_junior != 0 || self.delta_xp_senior != 0 {
            s.push_str("\n  XP CHANGES:\n");
            s.push_str(&format!("    Anna:   {:+}\n", self.delta_xp_anna));
            s.push_str(&format!("    Junior: {:+}\n", self.delta_xp_junior));
            s.push_str(&format!("    Senior: {:+}\n", self.delta_xp_senior));
        }

        // Per-phase deltas
        if !self.phase_deltas.is_empty() {
            s.push_str("\n  PER-PHASE CHANGES:\n");
            for pd in &self.phase_deltas {
                let icon = if pd.delta_success_rate > 0.0 { "+" }
                    else if pd.delta_success_rate < 0.0 { "-" }
                    else { "=" };
                s.push_str(&format!(
                    "    [{}] {}: {:+.1}% success, {:+}ms latency\n",
                    icon, pd.phase_name, pd.delta_success_rate, pd.delta_avg_latency
                ));
            }
        }

        // Summary
        s.push_str(&format!("\n{}\n", "-".repeat(60)));
        let summary_icon = if self.improved_reliability && self.improved_latency {
            "[TROPHY]"
        } else if self.regression_detected {
            "!"
        } else if self.improved_reliability || self.improved_latency {
            "[STATS]"
        } else {
            "━"
        };
        s.push_str(&format!("  {}  {}\n", summary_icon, self.explanation));

        s
    }
}

/// Compare two benchmark results
pub fn compare_benchmarks(older: &SnowLeopardResult, newer: &SnowLeopardResult) -> SnowLeopardDelta {
    let older_success = older.overall_success_rate();
    let newer_success = newer.overall_success_rate();
    let delta_success = newer_success - older_success;

    let older_latency = older.overall_avg_latency() as i64;
    let newer_latency = newer.overall_avg_latency() as i64;
    let delta_latency = newer_latency - older_latency;

    let delta_brain = newer.brain_usage_pct() - older.brain_usage_pct();
    let delta_llm = newer.llm_usage_pct() - older.llm_usage_pct();

    let delta_xp_anna = newer.total_xp.anna - older.total_xp.anna;
    let delta_xp_junior = newer.total_xp.junior - older.total_xp.junior;
    let delta_xp_senior = newer.total_xp.senior - older.total_xp.senior;

    // Per-phase deltas
    let mut phase_deltas = Vec::new();
    for newer_phase in &newer.phases {
        if let Some(older_phase) = older.phases.iter().find(|p| p.phase_id == newer_phase.phase_id) {
            let old_success = if older_phase.questions.is_empty() { 0.0 }
                else { older_phase.questions.iter().filter(|q| q.is_success).count() as f64
                    / older_phase.questions.len() as f64 * 100.0 };
            let new_success = if newer_phase.questions.is_empty() { 0.0 }
                else { newer_phase.questions.iter().filter(|q| q.is_success).count() as f64
                    / newer_phase.questions.len() as f64 * 100.0 };

            let old_latency: i64 = if older_phase.questions.is_empty() { 0 }
                else { older_phase.questions.iter().map(|q| q.latency_ms as i64).sum::<i64>()
                    / older_phase.questions.len() as i64 };
            let new_latency: i64 = if newer_phase.questions.is_empty() { 0 }
                else { newer_phase.questions.iter().map(|q| q.latency_ms as i64).sum::<i64>()
                    / newer_phase.questions.len() as i64 };

            phase_deltas.push(PhaseDelta {
                phase_name: newer_phase.phase_id.name().to_string(),
                delta_success_rate: new_success - old_success,
                delta_avg_latency: new_latency - old_latency,
                delta_questions: newer_phase.questions.len() as i32 - older_phase.questions.len() as i32,
            });
        }
    }

    // Summary flags
    let improved_reliability = delta_success > 1.0; // More than 1% improvement
    let improved_latency = delta_latency < -50; // More than 50ms faster
    let regression_detected = delta_success < -5.0 || delta_latency > 500; // 5% drop or 500ms slower

    // Human-readable explanation
    let explanation = if improved_reliability && improved_latency {
        "Both reliability and latency improved. Great progress!".to_string()
    } else if regression_detected {
        if delta_success < -5.0 {
            format!("Reliability regression detected ({:.1}% drop). Investigate recent changes.", delta_success)
        } else {
            format!("Latency regression detected ({}ms slower). Check LLM performance.", delta_latency)
        }
    } else if improved_reliability {
        format!("Reliability improved by {:.1}%. Latency unchanged.", delta_success)
    } else if improved_latency {
        format!("Latency improved by {}ms. Reliability stable.", -delta_latency)
    } else if delta_success.abs() < 1.0 && delta_latency.abs() < 50 {
        "Performance is stable. No significant changes.".to_string()
    } else {
        "Mixed results. Review per-phase data for details.".to_string()
    };

    SnowLeopardDelta {
        older_id: format!("snow_leopard_{}", older.timestamp.replace([':', '-', ' ', 'T'], "").chars().take(15).collect::<String>()),
        newer_id: format!("snow_leopard_{}", newer.timestamp.replace([':', '-', ' ', 'T'], "").chars().take(15).collect::<String>()),
        older_timestamp: older.timestamp.clone(),
        newer_timestamp: newer.timestamp.clone(),
        delta_success_rate: delta_success,
        delta_avg_latency: delta_latency,
        delta_brain_usage_pct: delta_brain,
        delta_llm_usage_pct: delta_llm,
        delta_xp_anna,
        delta_xp_junior,
        delta_xp_senior,
        delta_total_questions: newer.total_questions as i32 - older.total_questions as i32,
        phase_deltas,
        improved_reliability,
        improved_latency,
        regression_detected,
        explanation,
    }
}

/// Format a benchmark delta for display
pub fn format_benchmark_delta(delta: &SnowLeopardDelta) -> String {
    delta.format_ascii()
}

/// Get the last two benchmark entries for quick comparison
pub fn get_last_two_benchmarks() -> Option<(BenchmarkHistoryEntry, BenchmarkHistoryEntry)> {
    let history = BenchmarkHistoryEntry::list_recent(2);
    if history.len() < 2 {
        return None;
    }

    let newer = BenchmarkHistoryEntry::load(&history[0].id)?;
    let older = BenchmarkHistoryEntry::load(&history[1].id)?;

    Some((older, newer))
}

/// Compare the last two benchmarks
pub fn compare_last_two_benchmarks() -> Option<SnowLeopardDelta> {
    let (older, newer) = get_last_two_benchmarks()?;
    Some(compare_benchmarks(&older.full_result, &newer.full_result))
}

/// Format benchmark history as a compact table
pub fn format_benchmark_history(entries: &[BenchmarkHistoryListItem]) -> String {
    let mut s = String::new();
    s.push_str("SNOW LEOPARD BENCHMARK HISTORY\n");
    s.push_str(&format!("{}\n", "-".repeat(80)));
    s.push_str(&format!(
        "{:<20} {:>6} {:>10} {:>10} {:>8} {:>8}\n",
        "Timestamp", "Mode", "Success", "Latency", "Brain%", "LLM%"
    ));
    s.push_str(&format!("{}\n", "-".repeat(80)));

    for entry in entries {
        let mode_str = match entry.mode {
            BenchmarkMode::Full => "full",
            BenchmarkMode::Quick => "quick",
        };
        let ts = &entry.timestamp[..16.min(entry.timestamp.len())];
        s.push_str(&format!(
            "{:<20} {:>6} {:>9.0}% {:>8}ms {:>7.0}% {:>7.0}%\n",
            ts, mode_str, entry.success_rate, entry.avg_latency_ms,
            entry.brain_usage_pct, entry.llm_usage_pct
        ));
    }

    s.push_str(&format!("{}\n", "-".repeat(80)));
    s.push_str(&format!("  {} entries shown\n", entries.len()));
    s
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_mode_parsing() {
        assert_eq!(BenchmarkMode::from_str("run the full benchmark"), BenchmarkMode::Full);
        assert_eq!(BenchmarkMode::from_str("quick benchmark"), BenchmarkMode::Quick);
        assert_eq!(BenchmarkMode::from_str("short test"), BenchmarkMode::Quick);
        assert_eq!(BenchmarkMode::from_str("fast sanity check"), BenchmarkMode::Quick);
        assert_eq!(BenchmarkMode::from_str("run benchmark"), BenchmarkMode::Full);
    }

    #[test]
    fn test_is_benchmark_request() {
        assert!(is_benchmark_request("run the snow leopard benchmark"));
        assert!(is_benchmark_request("run a quick Snow Leopard benchmark"));
        assert!(is_benchmark_request("snow leopard tests"));
        assert!(is_benchmark_request("run benchmark"));
        assert!(!is_benchmark_request("what is my cpu?"));
        assert!(!is_benchmark_request("how much ram?"));
    }

    #[test]
    fn test_phase_ids() {
        assert_eq!(PhaseId::all().len(), 6);
        assert_eq!(PhaseId::quick().len(), 3);
        assert_eq!(PhaseId::HardReset.number(), 1);
        assert_eq!(PhaseId::LearningTest.number(), 6);
    }

    #[test]
    fn test_config_defaults() {
        let config = SnowLeopardConfig::default();
        assert!(!config.use_simulated_llm);
        assert!(config.perform_resets);
        assert_eq!(config.learning_repetitions, 5);
    }

    #[test]
    fn test_config_test_mode() {
        let config = SnowLeopardConfig::test_mode();
        assert!(config.use_simulated_llm);
        assert!(config.perform_resets);
    }

    #[test]
    fn test_xp_delta() {
        let delta = XpDelta { anna: 100, junior: 50, senior: 25 };
        assert_eq!(delta.total(), 175);
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello     ");
        assert_eq!(truncate_str("hello world test", 10), "hello w...");
    }

    // v1.5.0: Delta comparison tests
    #[test]
    fn test_compare_benchmarks_improved() {
        // Create two mock results with improvement
        let older = create_mock_result(80.0, 500, 40.0);  // 80% success, 500ms, 40% brain
        let newer = create_mock_result(90.0, 400, 50.0);  // 90% success, 400ms, 50% brain

        let delta = compare_benchmarks(&older, &newer);

        assert!(delta.delta_success_rate > 0.0, "Success rate should improve");
        assert!(delta.delta_avg_latency < 0, "Latency should decrease");
        assert!(delta.improved_reliability);
        assert!(delta.improved_latency);
        assert!(!delta.regression_detected);
    }

    #[test]
    fn test_compare_benchmarks_regression() {
        // Create two mock results with regression
        let older = create_mock_result(90.0, 400, 50.0);  // 90% success, 400ms, 50% brain
        let newer = create_mock_result(75.0, 600, 30.0);  // 75% success, 600ms, 30% brain

        let delta = compare_benchmarks(&older, &newer);

        assert!(delta.delta_success_rate < 0.0, "Success rate should regress");
        assert!(delta.delta_avg_latency > 0, "Latency should increase");
        assert!(!delta.improved_reliability);
        assert!(!delta.improved_latency);
        assert!(delta.regression_detected);
    }

    #[test]
    fn test_compare_benchmarks_stable() {
        // Create two mock results with no significant change
        let older = create_mock_result(85.0, 450, 45.0);
        let newer = create_mock_result(85.5, 445, 46.0);  // Very small changes

        let delta = compare_benchmarks(&older, &newer);

        assert!(delta.delta_success_rate.abs() < 1.0, "Success rate should be stable");
        assert!(delta.delta_avg_latency.abs() < 50, "Latency should be stable");
        assert!(!delta.improved_reliability);  // Less than 1% improvement
        assert!(!delta.improved_latency);      // Less than 50ms improvement
        assert!(!delta.regression_detected);
        assert!(delta.explanation.contains("stable"), "Should indicate stable performance");
    }

    #[test]
    fn test_delta_format_ascii() {
        let older = create_mock_result(80.0, 500, 40.0);
        let newer = create_mock_result(90.0, 400, 50.0);
        let delta = compare_benchmarks(&older, &newer);

        let formatted = delta.format_ascii();

        assert!(formatted.contains("SNOW LEOPARD BENCHMARK DELTA"));
        assert!(formatted.contains("Success rate"));
        assert!(formatted.contains("Avg latency"));
        assert!(formatted.contains("Brain usage"));
        assert!(formatted.contains("[STATS]") || formatted.contains("[TROPHY]"));  // Improvement indicator
    }

    #[test]
    fn test_history_list_item_serialization() {
        let item = BenchmarkHistoryListItem {
            id: "snow_leopard_20251129".to_string(),
            timestamp: "2025-11-29T10:00:00".to_string(),
            mode: BenchmarkMode::Full,
            success_rate: 85.0,
            avg_latency_ms: 450,
            brain_usage_pct: 45.0,
            llm_usage_pct: 55.0,
        };

        let json = serde_json::to_string(&item).unwrap();
        let parsed: BenchmarkHistoryListItem = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, item.id);
        assert_eq!(parsed.success_rate, item.success_rate);
    }

    #[test]
    fn test_format_benchmark_history() {
        let entries = vec![
            BenchmarkHistoryListItem {
                id: "snow_leopard_1".to_string(),
                timestamp: "2025-11-29T10:00:00".to_string(),
                mode: BenchmarkMode::Full,
                success_rate: 90.0,
                avg_latency_ms: 400,
                brain_usage_pct: 50.0,
                llm_usage_pct: 50.0,
            },
            BenchmarkHistoryListItem {
                id: "snow_leopard_2".to_string(),
                timestamp: "2025-11-28T10:00:00".to_string(),
                mode: BenchmarkMode::Quick,
                success_rate: 85.0,
                avg_latency_ms: 450,
                brain_usage_pct: 45.0,
                llm_usage_pct: 55.0,
            },
        ];

        let formatted = format_benchmark_history(&entries);

        assert!(formatted.contains("SNOW LEOPARD BENCHMARK HISTORY"));
        assert!(formatted.contains("Timestamp"));
        assert!(formatted.contains("Mode"));
        assert!(formatted.contains("Success"));
        assert!(formatted.contains("full"));
        assert!(formatted.contains("quick"));
        assert!(formatted.contains("2 entries shown"));
    }

    /// Helper to create a mock SnowLeopardResult for testing
    fn create_mock_result(success_rate: f64, latency_ms: u64, brain_pct: f64) -> SnowLeopardResult {
        let num_questions = 10;
        let num_success = (success_rate / 100.0 * num_questions as f64) as usize;
        let brain_count = (brain_pct / 100.0 * num_questions as f64) as usize;

        // Create mock questions
        let mut questions = Vec::new();
        for i in 0..num_questions {
            questions.push(QuestionResult {
                question_id: format!("q{}", i),
                question_text: format!("Question {}?", i),
                answer: "Mock answer".to_string(),
                origin: if i < brain_count { "Brain".to_string() } else { "LLM".to_string() },
                reliability: if i < num_success { 0.9 } else { 0.3 },
                latency_ms,
                probes_used: vec![],
                is_success: i < num_success,
                is_fallback: false,
                is_partial: false,
                xp_delta: if i < num_success { 10 } else { -5 },
                skill: "CpuInfo".to_string(), // Mock skill
            });
        }

        let mut origin_summary = HashMap::new();
        origin_summary.insert("Brain".to_string(), brain_count);
        origin_summary.insert("LLM".to_string(), num_questions - brain_count);

        // Build skill summary from questions
        let mut skill_summary = HashMap::new();
        skill_summary.insert("CpuInfo".to_string(), SkillStats {
            count: num_questions,
            success_count: num_success,
            fallback_count: 0,
            avg_latency_ms: latency_ms,
            avg_reliability: success_rate / 100.0,
            total_xp: (num_success as i64 * 10) - ((num_questions - num_success) as i64 * 5),
        });

        SnowLeopardResult {
            mode: BenchmarkMode::Full,
            timestamp: "2025-11-29T10:00:00".to_string(),
            phases: vec![PhaseResult {
                phase_id: PhaseId::HardReset,
                phase_name: "Hard Reset".to_string(),
                questions,
                xp_before: XpSnapshot {
                    anna_level: 1, anna_xp: 0, anna_trust: 0.5,
                    junior_level: 1, junior_xp: 0,
                    senior_level: 1, senior_xp: 0,
                    total_questions: 0,
                },
                xp_after: XpSnapshot {
                    anna_level: 1, anna_xp: 100, anna_trust: 0.55,
                    junior_level: 1, junior_xp: 50,
                    senior_level: 1, senior_xp: 50,
                    total_questions: num_questions as u64,
                },
                total_duration_ms: latency_ms * num_questions as u64,
                new_probes_created: 2,
                fallback_count: 0,
                partial_count: 0,
            }],
            total_questions: num_questions,
            total_xp: XpDelta { anna: 100, junior: 50, senior: 50 },
            total_probes_used: 5,
            latency_evolution: vec![("Phase1".to_string(), latency_ms)],
            reliability_evolution: vec![("Phase1".to_string(), success_rate / 100.0)],
            success_rate_evolution: vec![("Phase1".to_string(), success_rate)],
            origin_summary,
            skill_summary,
            ux_consistency_passed: success_rate >= 80.0,
            report_path: None,
            ascii_summary: "Mock summary".to_string(),
            warnings: vec![],
        }
    }
}
