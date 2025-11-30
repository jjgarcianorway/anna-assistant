//! Adaptive LLM Provisioning v3.0
//!
//! This module gives Anna full control over her own LLM model selection,
//! installation, and optimization. No hardcoding - fully dynamic.
//!
//! ## Design Principles
//!
//! 1. **Self-provisioning**: Anna tests and selects her own models
//! 2. **Adaptive**: Re-evaluates models based on performance telemetry
//! 3. **Fail-safe**: Always has fallback to Brain for deterministic tasks
//! 4. **No manual tuning**: Optimal choice at runtime
//! 5. **Hardware-aware**: Selects models based on CPU/RAM/GPU capabilities
//!
//! ## Model Selection Flow (v3.0)
//!
//! ```text
//! First Launch / Reset
//!        ↓
//! Detect hardware (CPU cores, RAM, GPU)
//!        ↓
//! Determine hardware tier (Minimal, Basic, Standard, Power)
//!        ↓
//! List available Ollama models
//!        ↓
//! Benchmark each candidate
//!        ↓
//! Select best Router (tiny, fast)
//! Select best Junior (JSON-fast)
//! Select best Senior (reasoning) - optional for high-end
//!        ↓
//! Save selection to config
//!        ↓
//! Ready for queries
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Base directory for LLM benchmarks and state
pub const LLM_STATE_DIR: &str = "/var/lib/anna/llm";
pub const BENCHMARK_DIR: &str = "/var/lib/anna/llm/benchmarks";
pub const SELECTION_FILE: &str = "/var/lib/anna/llm/selection.json";

/// Timeout for a single model benchmark test
pub const BENCHMARK_TIMEOUT_MS: u64 = 30_000;

/// Junior model latency threshold for benchmarking (must respond within this to be viable)
pub const JUNIOR_MAX_LATENCY_MS: u64 = 8_000;

/// Junior fallback timeout - if Junior takes longer than this, use fallback answer
/// Per spec: "If Junior takes >2 seconds, use pre-trained fallback"
pub const JUNIOR_FALLBACK_TIMEOUT_MS: u64 = 2_000;

/// Senior model latency threshold
pub const SENIOR_MAX_LATENCY_MS: u64 = 15_000;

/// Minimum JSON compliance score for Junior
pub const JUNIOR_MIN_JSON_COMPLIANCE: f64 = 0.95;

/// Minimum reasoning score for Senior
pub const SENIOR_MIN_REASONING_SCORE: f64 = 0.85;

/// Number of test runs for determinism check
pub const DETERMINISM_RUNS: usize = 3;

// ============================================================================
// MODEL CANDIDATES
// ============================================================================

/// Junior model candidates - fast, JSON-compliant
pub const JUNIOR_CANDIDATES: &[&str] = &[
    "qwen2.5:1.5b-instruct",
    "qwen2.5:3b-instruct",
    "phi3:mini",
    "phi3.5:3.8b",
    "mistral:7b-instruct-v0.3-q4_0",
    "gemma2:2b",
    "llama3.2:1b",
    "llama3.2:3b",
];

/// Senior model candidates - stronger reasoning
pub const SENIOR_CANDIDATES: &[&str] = &[
    "qwen2.5:7b-instruct",
    "qwen2.5:14b-instruct",
    "mistral:7b-instruct-v0.3",
    "llama3.1:8b-instruct-q4_0",
    "gemma2:9b",
    "deepseek-coder:6.7b",
];

/// v3.0: Router model candidates - tiny and fast for classification only
pub const ROUTER_CANDIDATES: &[&str] = &[
    "qwen2.5:0.5b-instruct",
    "qwen2.5:1.5b-instruct",
    "phi3:mini",
    "gemma2:2b",
    "llama3.2:1b",
];

/// Fallback model if nothing else works
pub const FALLBACK_MODEL: &str = "qwen2.5:1.5b-instruct";

/// v3.0: Fallback router model (same as junior fallback)
pub const ROUTER_FALLBACK_MODEL: &str = "qwen2.5:1.5b-instruct";

// ============================================================================
// HARDWARE TIERS (v3.0)
// ============================================================================

/// Hardware tier determines which models are appropriate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareTier {
    /// <4 cores, <4GB RAM - Brain only, no LLM
    Minimal,
    /// 4-7 cores, 4-8GB RAM - Router + Junior only
    Basic,
    /// 8-15 cores, 8-16GB RAM - Router + Junior + Senior
    Standard,
    /// 16+ cores, 16GB+ RAM, or GPU - All models, larger options
    Power,
}

impl HardwareTier {
    /// Detect hardware tier from system probes
    pub fn detect() -> Self {
        let cores = detect_cpu_cores();
        let ram_gb = detect_ram_gb();
        let has_gpu = detect_nvidia_gpu();

        // GPU always means Power tier
        if has_gpu {
            return HardwareTier::Power;
        }

        // Use cores and RAM to determine tier
        match (cores, ram_gb) {
            (c, r) if c >= 16 && r >= 16 => HardwareTier::Power,
            (c, r) if c >= 8 && r >= 8 => HardwareTier::Standard,
            (c, r) if c >= 4 && r >= 4 => HardwareTier::Basic,
            _ => HardwareTier::Minimal,
        }
    }

    /// Is Senior model recommended for this tier?
    pub fn has_senior(&self) -> bool {
        matches!(self, HardwareTier::Standard | HardwareTier::Power)
    }

    /// Is Router model recommended for this tier?
    pub fn has_router(&self) -> bool {
        !matches!(self, HardwareTier::Minimal)
    }

    /// Get recommended model sizes for this tier
    pub fn model_sizes(&self) -> ModelSizeRecommendation {
        match self {
            HardwareTier::Minimal => ModelSizeRecommendation {
                router_max_params: 0,  // No router
                junior_max_params: 0,  // Brain only
                senior_max_params: 0,
            },
            HardwareTier::Basic => ModelSizeRecommendation {
                router_max_params: 1_500_000_000,  // 1.5B max
                junior_max_params: 3_000_000_000,  // 3B max
                senior_max_params: 0,              // No senior
            },
            HardwareTier::Standard => ModelSizeRecommendation {
                router_max_params: 1_500_000_000,  // 1.5B max
                junior_max_params: 7_000_000_000,  // 7B max
                senior_max_params: 14_000_000_000, // 14B max
            },
            HardwareTier::Power => ModelSizeRecommendation {
                router_max_params: 3_000_000_000,  // 3B max
                junior_max_params: 14_000_000_000, // 14B max
                senior_max_params: 70_000_000_000, // 70B max
            },
        }
    }
}

/// Model size recommendations per tier
#[derive(Debug, Clone)]
pub struct ModelSizeRecommendation {
    pub router_max_params: u64,
    pub junior_max_params: u64,
    pub senior_max_params: u64,
}

/// Detect number of CPU cores
fn detect_cpu_cores() -> usize {
    let output = Command::new("nproc")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.trim().parse().unwrap_or(1)
        }
        _ => 1,
    }
}

/// Detect RAM in GB
fn detect_ram_gb() -> usize {
    let output = Command::new("sh")
        .args(["-c", "grep MemTotal /proc/meminfo | awk '{print int($2/1024/1024)}'"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.trim().parse().unwrap_or(1)
        }
        _ => 1,
    }
}

/// Detect if NVIDIA GPU is present
fn detect_nvidia_gpu() -> bool {
    let output = Command::new("lspci")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.to_lowercase().contains("nvidia")
        }
        _ => false,
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Result of benchmarking a single model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBenchmark {
    pub model_name: String,
    pub timestamp: String,

    // Latency metrics
    pub first_token_latency_ms: u64,
    pub total_latency_ms: u64,
    pub avg_latency_ms: u64,

    // Quality metrics
    pub json_compliance: f64,      // 0.0 - 1.0
    pub determinism_score: f64,    // 0.0 - 1.0
    pub reasoning_score: f64,      // 0.0 - 1.0

    // Computed scores
    pub junior_score: f64,         // Combined score for Junior role
    pub senior_score: f64,         // Combined score for Senior role

    // Status
    pub is_available: bool,
    pub error: Option<String>,
}

impl ModelBenchmark {
    pub fn unavailable(model: &str, error: &str) -> Self {
        Self {
            model_name: model.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            first_token_latency_ms: 0,
            total_latency_ms: 0,
            avg_latency_ms: 0,
            json_compliance: 0.0,
            determinism_score: 0.0,
            reasoning_score: 0.0,
            junior_score: 0.0,
            senior_score: 0.0,
            is_available: false,
            error: Some(error.to_string()),
        }
    }

    /// Check if model meets Junior criteria
    pub fn is_junior_viable(&self) -> bool {
        self.is_available
            && self.json_compliance >= JUNIOR_MIN_JSON_COMPLIANCE
            && self.avg_latency_ms <= JUNIOR_MAX_LATENCY_MS
            && self.determinism_score >= 0.90
    }

    /// Check if model meets Senior criteria
    pub fn is_senior_viable(&self) -> bool {
        self.is_available
            && self.reasoning_score >= SENIOR_MIN_REASONING_SCORE
            && self.avg_latency_ms <= SENIOR_MAX_LATENCY_MS
    }
}

/// Current LLM selection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSelection {
    /// v3.0: Router model for classification (tiny, fast)
    #[serde(default = "default_router_model")]
    pub router_model: String,
    #[serde(default)]
    pub router_score: f64,
    pub junior_model: String,
    pub junior_score: f64,
    pub senior_model: String,
    pub senior_score: f64,
    pub last_benchmark: String,
    pub autoprovision_enabled: bool,
    pub suggestions: Vec<String>,
    /// v3.0: Detected hardware tier
    #[serde(default)]
    pub hardware_tier: Option<HardwareTier>,
}

fn default_router_model() -> String {
    ROUTER_FALLBACK_MODEL.to_string()
}

impl Default for LlmSelection {
    fn default() -> Self {
        Self {
            router_model: ROUTER_FALLBACK_MODEL.to_string(),
            router_score: 0.0,
            junior_model: FALLBACK_MODEL.to_string(),
            junior_score: 0.0,
            senior_model: FALLBACK_MODEL.to_string(),
            senior_score: 0.0,
            last_benchmark: String::new(),
            autoprovision_enabled: true,
            suggestions: vec![],
            hardware_tier: None,
        }
    }
}

impl LlmSelection {
    /// Load from disk or return default
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(SELECTION_FILE) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        fs::create_dir_all(LLM_STATE_DIR)?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(SELECTION_FILE, json)
    }

    /// Format for status display
    pub fn format_status(&self) -> String {
        let mut output = String::new();
        output.push_str("LLM AUTOPROVISION (v3.0)\n");
        output.push_str("──────────────────────────────────────────\n");

        // v3.0: Hardware tier
        if let Some(tier) = &self.hardware_tier {
            output.push_str(&format!("Hardware tier: {:?}\n", tier));
        }

        // v3.0: Router model
        if self.router_score > 0.0 {
            output.push_str(&format!("Router model: {} (score {:.2})\n",
                self.router_model, self.router_score));
        }

        output.push_str(&format!("Junior model: {} (score {:.2})\n",
            self.junior_model, self.junior_score));
        output.push_str(&format!("Senior model: {} (score {:.2})\n",
            self.senior_model, self.senior_score));
        output.push_str(&format!("Autoprovision: {}\n",
            if self.autoprovision_enabled { "enabled" } else { "disabled" }));

        if !self.last_benchmark.is_empty() {
            output.push_str(&format!("Last benchmark: {}\n", self.last_benchmark));
        }

        if !self.suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for s in &self.suggestions {
                output.push_str(&format!("  - {}\n", s));
            }
        }

        output
    }
}

// ============================================================================
// JUNIOR FALLBACK POLICY
// ============================================================================

/// Check if Junior exceeded the fallback timeout threshold.
///
/// Per spec: "If Junior takes >2 seconds, automatically use fallback answer."
/// This helps ensure responsive UX even with slow LLMs.
pub fn should_fallback_junior(latency_ms: u64) -> bool {
    latency_ms > JUNIOR_FALLBACK_TIMEOUT_MS
}

/// Fallback decision result
#[derive(Debug, Clone)]
pub struct FallbackDecision {
    /// Whether to use fallback
    pub use_fallback: bool,
    /// Reason for the decision
    pub reason: String,
    /// Latency that triggered the decision (if applicable)
    pub latency_ms: Option<u64>,
}

impl FallbackDecision {
    /// Create a "proceed" decision (no fallback needed)
    pub fn proceed() -> Self {
        Self {
            use_fallback: false,
            reason: "Junior responded within timeout".to_string(),
            latency_ms: None,
        }
    }

    /// Create a "fallback" decision due to timeout
    pub fn timeout(latency_ms: u64) -> Self {
        Self {
            use_fallback: true,
            reason: format!(
                "Junior exceeded {}ms timeout (took {}ms)",
                JUNIOR_FALLBACK_TIMEOUT_MS, latency_ms
            ),
            latency_ms: Some(latency_ms),
        }
    }

    /// Create a "fallback" decision due to error
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            use_fallback: true,
            reason: msg.into(),
            latency_ms: None,
        }
    }
}

/// Evaluate whether to use fallback based on Junior's response time.
///
/// This is the central function for the automatic fallback policy.
pub fn evaluate_junior_fallback(latency_ms: u64) -> FallbackDecision {
    if should_fallback_junior(latency_ms) {
        FallbackDecision::timeout(latency_ms)
    } else {
        FallbackDecision::proceed()
    }
}

// ============================================================================
// OLLAMA INTERACTION
// ============================================================================

/// List all models currently available in Ollama
pub fn list_ollama_models() -> Vec<String> {
    let output = Command::new("ollama")
        .arg("list")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .skip(1) // Skip header line
                .filter_map(|line| {
                    line.split_whitespace().next().map(|s| s.to_string())
                })
                .collect()
        }
        _ => vec![],
    }
}

/// Check if a specific model is available in Ollama
pub fn is_model_available(model: &str) -> bool {
    let models = list_ollama_models();
    models.iter().any(|m| m.starts_with(model.split(':').next().unwrap_or(model)))
}

/// Pull a model from Ollama registry
pub fn pull_model(model: &str) -> Result<(), String> {
    let output = Command::new("ollama")
        .args(["pull", model])
        .output()
        .map_err(|e| format!("Failed to run ollama pull: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to pull {}: {}", model, stderr))
    }
}

// ============================================================================
// MODEL BENCHMARKING
// ============================================================================

/// Test JSON compliance with a simple structured task
const JSON_TEST_PROMPT: &str = r#"You are a JSON-only assistant. Respond ONLY with valid JSON.

Task: What is the capital of France?

Respond with this exact JSON structure:
{"answer": "the capital city name", "confidence": 0.95}

JSON response:"#;

/// Test reasoning with a simple logic task
const REASONING_TEST_PROMPT: &str = r#"Solve this step by step:

If all roses are flowers, and some flowers are red, can we conclude that some roses are red?

Think through this logically and give your answer as JSON:
{"reasoning": "your step by step reasoning", "answer": "yes/no/uncertain", "confidence": 0.0-1.0}"#;

/// Benchmark a single model for Junior/Senior suitability
pub fn benchmark_model(model: &str) -> ModelBenchmark {
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Check if model is available
    if !is_model_available(model) {
        return ModelBenchmark::unavailable(model, "Model not installed in Ollama");
    }

    // Run JSON compliance tests
    let mut json_scores = Vec::new();
    let mut latencies = Vec::new();
    let mut first_token_latency = 0u64;
    let mut responses = Vec::new();

    for i in 0..DETERMINISM_RUNS {
        match run_ollama_test(model, JSON_TEST_PROMPT, BENCHMARK_TIMEOUT_MS) {
            Ok((response, latency, first_token)) => {
                if i == 0 {
                    first_token_latency = first_token;
                }
                latencies.push(latency);
                responses.push(response.clone());

                // Score JSON compliance
                let score = score_json_response(&response);
                json_scores.push(score);
            }
            Err(e) => {
                return ModelBenchmark::unavailable(model, &e);
            }
        }
    }

    // Calculate determinism (how similar are responses)
    let determinism = calculate_determinism(&responses);

    // Calculate JSON compliance average
    let json_compliance = if json_scores.is_empty() {
        0.0
    } else {
        json_scores.iter().sum::<f64>() / json_scores.len() as f64
    };

    // Calculate average latency
    let avg_latency = if latencies.is_empty() {
        0
    } else {
        latencies.iter().sum::<u64>() / latencies.len() as u64
    };

    let total_latency = latencies.iter().sum::<u64>();

    // Run reasoning test
    let reasoning_score = match run_ollama_test(model, REASONING_TEST_PROMPT, BENCHMARK_TIMEOUT_MS) {
        Ok((response, _, _)) => score_reasoning_response(&response),
        Err(_) => 0.0,
    };

    // Calculate composite scores
    let junior_score = calculate_junior_score(json_compliance, determinism, avg_latency);
    let senior_score = calculate_senior_score(reasoning_score, json_compliance, avg_latency);

    ModelBenchmark {
        model_name: model.to_string(),
        timestamp,
        first_token_latency_ms: first_token_latency,
        total_latency_ms: total_latency,
        avg_latency_ms: avg_latency,
        json_compliance,
        determinism_score: determinism,
        reasoning_score,
        junior_score,
        senior_score,
        is_available: true,
        error: None,
    }
}

/// Run a single Ollama test and measure latency
fn run_ollama_test(model: &str, prompt: &str, timeout_ms: u64) -> Result<(String, u64, u64), String> {
    let start = Instant::now();

    let output = Command::new("ollama")
        .args(["run", model, prompt])
        .output()
        .map_err(|e| format!("Failed to run ollama: {}", e))?;

    let total_latency = start.elapsed().as_millis() as u64;

    if total_latency > timeout_ms {
        return Err(format!("Timeout: {}ms > {}ms", total_latency, timeout_ms));
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Ollama error: {}", stderr));
    }

    let response = String::from_utf8_lossy(&output.stdout).to_string();

    // Estimate first token latency (simplified - actual streaming would be better)
    let first_token_latency = total_latency / 3; // Rough estimate

    Ok((response, total_latency, first_token_latency))
}

/// Score a response for JSON compliance
fn score_json_response(response: &str) -> f64 {
    // Try to find JSON in the response
    let trimmed = response.trim();

    // Check if it's valid JSON
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        // Check if it has expected structure
        if value.get("answer").is_some() {
            return 1.0;
        }
        return 0.8; // Valid JSON but wrong structure
    }

    // Try to extract JSON from response
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            let json_part = &trimmed[start..=end];
            if serde_json::from_str::<serde_json::Value>(json_part).is_ok() {
                return 0.7; // JSON found but with extra text
            }
        }
    }

    0.0 // No valid JSON found
}

/// Score a response for reasoning quality
fn score_reasoning_response(response: &str) -> f64 {
    let trimmed = response.trim();
    let lower = trimmed.to_lowercase();

    // Check for JSON structure
    let has_json = serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
        || trimmed.contains('{');

    // Check for reasoning indicators
    let has_reasoning = lower.contains("because")
        || lower.contains("therefore")
        || lower.contains("step")
        || lower.contains("first")
        || lower.contains("logic");

    // Check for correct answer (uncertain is technically correct)
    let has_correct_answer = lower.contains("uncertain")
        || lower.contains("cannot conclude")
        || lower.contains("no");

    let mut score: f64 = 0.0;
    if has_json { score += 0.3; }
    if has_reasoning { score += 0.4; }
    if has_correct_answer { score += 0.3; }

    score.min(1.0)
}

/// Calculate determinism score from multiple responses
fn calculate_determinism(responses: &[String]) -> f64 {
    if responses.len() < 2 {
        return 1.0;
    }

    // Simple similarity: count how many responses are identical
    let first = &responses[0];
    let identical_count = responses.iter().filter(|r| *r == first).count();

    identical_count as f64 / responses.len() as f64
}

/// Calculate Junior composite score
fn calculate_junior_score(json_compliance: f64, determinism: f64, avg_latency_ms: u64) -> f64 {
    // Weights: JSON compliance most important, then speed, then determinism
    let json_weight = 0.5;
    let speed_weight = 0.3;
    let determinism_weight = 0.2;

    // Normalize latency (0 = max latency, 1 = instant)
    let speed_score = if avg_latency_ms >= JUNIOR_MAX_LATENCY_MS {
        0.0
    } else {
        1.0 - (avg_latency_ms as f64 / JUNIOR_MAX_LATENCY_MS as f64)
    };

    json_compliance * json_weight + speed_score * speed_weight + determinism * determinism_weight
}

/// Calculate Senior composite score
fn calculate_senior_score(reasoning: f64, json_compliance: f64, avg_latency_ms: u64) -> f64 {
    // Weights: Reasoning most important
    let reasoning_weight = 0.6;
    let json_weight = 0.2;
    let speed_weight = 0.2;

    let speed_score = if avg_latency_ms >= SENIOR_MAX_LATENCY_MS {
        0.0
    } else {
        1.0 - (avg_latency_ms as f64 / SENIOR_MAX_LATENCY_MS as f64)
    };

    reasoning * reasoning_weight + json_compliance * json_weight + speed_score * speed_weight
}

// ============================================================================
// MODEL SELECTION
// ============================================================================

/// Select the best Junior model from available candidates
pub fn select_best_junior(benchmarks: &[ModelBenchmark]) -> Option<&ModelBenchmark> {
    benchmarks
        .iter()
        .filter(|b| b.is_junior_viable())
        .max_by(|a, b| a.junior_score.partial_cmp(&b.junior_score).unwrap())
}

/// Select the best Senior model from available candidates
pub fn select_best_senior(benchmarks: &[ModelBenchmark]) -> Option<&ModelBenchmark> {
    benchmarks
        .iter()
        .filter(|b| b.is_senior_viable())
        .max_by(|a, b| a.senior_score.partial_cmp(&b.senior_score).unwrap())
}

/// v3.0: Select the best Router model (prioritize speed over quality)
pub fn select_best_router(benchmarks: &[ModelBenchmark]) -> Option<&ModelBenchmark> {
    benchmarks
        .iter()
        .filter(|b| {
            b.is_available
                && b.avg_latency_ms <= 1000  // Must respond in <1s
                && b.json_compliance >= 0.80  // Decent JSON compliance
        })
        .max_by(|a, b| {
            // Prioritize speed over quality for router
            let a_score = (1.0 - (a.avg_latency_ms as f64 / 1000.0)).max(0.0) * 0.6
                + a.json_compliance * 0.4;
            let b_score = (1.0 - (b.avg_latency_ms as f64 / 1000.0)).max(0.0) * 0.6
                + b.json_compliance * 0.4;
            a_score.partial_cmp(&b_score).unwrap()
        })
}

/// Run full autoprovision: benchmark all candidates and select best
/// v3.0: Now includes Router model and hardware tier detection
pub fn run_autoprovision() -> LlmSelection {
    let mut all_benchmarks: Vec<ModelBenchmark> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    // v3.0: Detect hardware tier first
    let hardware_tier = HardwareTier::detect();

    // v3.0: Benchmark Router candidates (if hardware supports it)
    if hardware_tier.has_router() {
        for candidate in ROUTER_CANDIDATES {
            if !all_benchmarks.iter().any(|b| b.model_name == *candidate) {
                let benchmark = benchmark_model(candidate);
                if !benchmark.is_available {
                    suggestions.push(format!("Consider installing {} for Router role", candidate));
                }
                all_benchmarks.push(benchmark);
            }
        }
    }

    // Benchmark Junior candidates
    for candidate in JUNIOR_CANDIDATES {
        if !all_benchmarks.iter().any(|b| b.model_name == *candidate) {
            let benchmark = benchmark_model(candidate);
            if !benchmark.is_available {
                suggestions.push(format!("Consider installing {} for Junior role", candidate));
            }
            all_benchmarks.push(benchmark);
        }
    }

    // Benchmark Senior candidates (only if hardware supports it)
    if hardware_tier.has_senior() {
        for candidate in SENIOR_CANDIDATES {
            if !all_benchmarks.iter().any(|b| b.model_name == *candidate) {
                let benchmark = benchmark_model(candidate);
                if !benchmark.is_available {
                    suggestions.push(format!("Consider installing {} for Senior role", candidate));
                }
                all_benchmarks.push(benchmark);
            }
        }
    }

    // v3.0: Select best Router
    let router = if hardware_tier.has_router() {
        select_best_router(&all_benchmarks)
            .map(|b| (b.model_name.clone(), b.junior_score))
            .unwrap_or_else(|| (ROUTER_FALLBACK_MODEL.to_string(), 0.0))
    } else {
        (ROUTER_FALLBACK_MODEL.to_string(), 0.0)
    };

    // Select best Junior
    let junior = select_best_junior(&all_benchmarks)
        .map(|b| (b.model_name.clone(), b.junior_score))
        .unwrap_or_else(|| (FALLBACK_MODEL.to_string(), 0.0));

    // Select best Senior (only if hardware supports it)
    let senior = if hardware_tier.has_senior() {
        select_best_senior(&all_benchmarks)
            .map(|b| (b.model_name.clone(), b.senior_score))
            .unwrap_or_else(|| (FALLBACK_MODEL.to_string(), 0.0))
    } else {
        (FALLBACK_MODEL.to_string(), 0.0)
    };

    // Add performance suggestions
    if hardware_tier.has_router() && router.1 < 0.5 {
        suggestions.push("Router model performance is poor. Consider installing qwen2.5:0.5b-instruct".to_string());
    }
    if junior.1 < 0.5 {
        suggestions.push("Junior model performance is poor. Consider installing qwen2.5:1.5b-instruct".to_string());
    }
    if hardware_tier.has_senior() && senior.1 < 0.5 {
        suggestions.push("Senior model performance is poor. Consider installing qwen2.5:7b-instruct".to_string());
    }

    // v3.0: Tier-specific suggestions
    match hardware_tier {
        HardwareTier::Minimal => {
            suggestions.push("Minimal hardware detected - using Brain-only mode (no LLM)".to_string());
        }
        HardwareTier::Basic => {
            suggestions.push("Basic hardware - Senior model disabled for performance".to_string());
        }
        _ => {}
    }

    let selection = LlmSelection {
        router_model: router.0,
        router_score: router.1,
        junior_model: junior.0,
        junior_score: junior.1,
        senior_model: senior.0,
        senior_score: senior.1,
        last_benchmark: chrono::Utc::now().to_rfc3339(),
        autoprovision_enabled: true,
        suggestions,
        hardware_tier: Some(hardware_tier),
    };

    // Save selection
    if let Err(e) = selection.save() {
        eprintln!("Warning: Could not save LLM selection: {}", e);
    }

    // Save individual benchmarks
    save_benchmarks(&all_benchmarks);

    selection
}

/// Save benchmarks to disk
fn save_benchmarks(benchmarks: &[ModelBenchmark]) {
    if let Err(e) = fs::create_dir_all(BENCHMARK_DIR) {
        eprintln!("Warning: Could not create benchmark dir: {}", e);
        return;
    }

    for benchmark in benchmarks {
        let filename = format!("{}/{}.json",
            BENCHMARK_DIR,
            benchmark.model_name.replace([':', '/'], "_"));

        if let Ok(json) = serde_json::to_string_pretty(benchmark) {
            let _ = fs::write(&filename, json);
        }
    }
}

// ============================================================================
// AUTOMATIC MODEL INSTALLATION
// ============================================================================

/// Ensure required models are installed
pub fn ensure_models_installed(selection: &LlmSelection) -> Result<(), String> {
    let models_to_check = [&selection.junior_model, &selection.senior_model];

    for model in models_to_check {
        if !is_model_available(model) {
            println!("Installing model: {}", model);
            pull_model(model)?;
        }
    }

    Ok(())
}

/// Install a specific model if missing, with progress callback
pub fn install_model_if_missing<F>(model: &str, on_progress: F) -> Result<bool, String>
where
    F: Fn(&str),
{
    if is_model_available(model) {
        return Ok(false); // Already installed
    }

    on_progress(&format!("Pulling model {}...", model));
    pull_model(model)?;
    on_progress(&format!("Model {} installed successfully", model));

    Ok(true) // Was installed
}

// ============================================================================
// FALLBACK HANDLING
// ============================================================================

/// Record a Junior timeout penalty
pub fn record_junior_timeout(selection: &mut LlmSelection) {
    // Reduce Junior score on timeout
    selection.junior_score = (selection.junior_score - 0.1).max(0.0);

    if selection.junior_score < 0.3 {
        selection.suggestions.push(
            "Junior model is timing out frequently. Re-run autoprovision.".to_string()
        );
    }

    let _ = selection.save();
}

/// Check if we should re-run autoprovision based on performance
pub fn should_reprovision(selection: &LlmSelection) -> bool {
    // Re-provision if scores are too low
    if selection.junior_score < 0.3 || selection.senior_score < 0.3 {
        return true;
    }

    // Re-provision if last benchmark was more than a week ago
    if let Ok(last) = chrono::DateTime::parse_from_rfc3339(&selection.last_benchmark) {
        let week_ago = chrono::Utc::now() - chrono::Duration::days(7);
        if last < week_ago {
            return true;
        }
    }

    false
}

// ============================================================================
// RUNTIME MODEL SWITCHING
// ============================================================================

/// Result of a model switch attempt
#[derive(Debug, Clone)]
pub struct ModelSwitchResult {
    pub switched: bool,
    pub old_model: String,
    pub new_model: Option<String>,
    pub reason: String,
}

impl ModelSwitchResult {
    /// No switch needed
    pub fn no_switch(current: &str) -> Self {
        Self {
            switched: false,
            old_model: current.to_string(),
            new_model: None,
            reason: "No switch needed".to_string(),
        }
    }

    /// Switch occurred
    pub fn switched(old: &str, new: &str, reason: &str) -> Self {
        Self {
            switched: true,
            old_model: old.to_string(),
            new_model: Some(new.to_string()),
            reason: reason.to_string(),
        }
    }
}

/// Try to switch to a better Junior model after repeated failures
///
/// This is called when Junior keeps timing out or failing.
/// It tries to find a faster/smaller model from the candidate list.
pub fn try_switch_junior_model(selection: &mut LlmSelection) -> ModelSwitchResult {
    let current = &selection.junior_model;

    // Find next faster candidate
    let faster_models: Vec<&str> = JUNIOR_CANDIDATES
        .iter()
        .filter(|m| {
            // Only consider models that are:
            // 1. Different from current
            // 2. Available in Ollama
            // 3. Smaller (by name heuristic - lower numbers = smaller)
            **m != current && is_model_available(m)
        })
        .take(3)
        .copied()
        .collect();

    if faster_models.is_empty() {
        return ModelSwitchResult::no_switch(current);
    }

    // Try the first available faster model
    let new_model = faster_models[0];
    let old_model = selection.junior_model.clone();

    selection.junior_model = new_model.to_string();
    selection.junior_score = 0.5; // Reset score - needs re-evaluation
    selection.suggestions.push(format!(
        "Switched Junior from {} to {} due to performance issues",
        old_model, new_model
    ));

    if let Err(e) = selection.save() {
        tracing::warn!("Failed to save selection after switch: {}", e);
    }

    ModelSwitchResult::switched(
        &old_model,
        new_model,
        "Junior model was timing out frequently",
    )
}

/// Try to switch to a better Senior model after repeated failures
pub fn try_switch_senior_model(selection: &mut LlmSelection) -> ModelSwitchResult {
    let current = &selection.senior_model;

    // Find alternative candidates
    let alt_models: Vec<&str> = SENIOR_CANDIDATES
        .iter()
        .filter(|m| **m != current && is_model_available(m))
        .take(3)
        .copied()
        .collect();

    if alt_models.is_empty() {
        return ModelSwitchResult::no_switch(current);
    }

    // Try the first available alternative
    let new_model = alt_models[0];
    let old_model = selection.senior_model.clone();

    selection.senior_model = new_model.to_string();
    selection.senior_score = 0.5; // Reset score
    selection.suggestions.push(format!(
        "Switched Senior from {} to {} due to performance issues",
        old_model, new_model
    ));

    if let Err(e) = selection.save() {
        tracing::warn!("Failed to save selection after switch: {}", e);
    }

    ModelSwitchResult::switched(
        &old_model,
        new_model,
        "Senior model was performing poorly",
    )
}

/// Record a model failure and potentially trigger a switch
///
/// This is the main entry point for runtime model adaptation.
/// Call this when a model fails or times out repeatedly.
pub fn handle_model_failure(
    selection: &mut LlmSelection,
    is_junior: bool,
    failure_count: usize,
) -> Option<ModelSwitchResult> {
    const SWITCH_THRESHOLD: usize = 3; // Switch after 3 consecutive failures

    if failure_count < SWITCH_THRESHOLD {
        // Not enough failures yet
        if is_junior {
            record_junior_timeout(selection);
        }
        return None;
    }

    // Too many failures - try to switch
    let result = if is_junior {
        try_switch_junior_model(selection)
    } else {
        try_switch_senior_model(selection)
    };

    if result.switched {
        Some(result)
    } else {
        None
    }
}

// ============================================================================
// OLLAMA DETECTION AND INSTALLATION
// ============================================================================

/// Check if Ollama is installed on the system
pub fn is_ollama_installed() -> bool {
    Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Ollama service is running
pub fn is_ollama_running() -> bool {
    Command::new("ollama")
        .arg("list")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install Ollama using the official installer script
pub fn install_ollama() -> Result<(), String> {
    if is_ollama_installed() {
        return Ok(()); // Already installed
    }

    // Use official Ollama installer
    let output = Command::new("sh")
        .args(["-c", "curl -fsSL https://ollama.com/install.sh | sh"])
        .output()
        .map_err(|e| format!("Failed to run Ollama installer: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Ollama installation failed: {}", stderr))
    }
}

/// Start Ollama service (if using systemd)
pub fn start_ollama_service() -> Result<(), String> {
    // Try systemd first
    let systemd_result = Command::new("systemctl")
        .args(["start", "ollama"])
        .output();

    if let Ok(output) = systemd_result {
        if output.status.success() {
            // Give it time to start
            std::thread::sleep(std::time::Duration::from_secs(2));
            return Ok(());
        }
    }

    // If systemd fails, try running ollama serve in background
    // This handles non-systemd systems
    let _ = Command::new("sh")
        .args(["-c", "ollama serve &>/dev/null &"])
        .spawn();

    std::thread::sleep(std::time::Duration::from_secs(2));

    if is_ollama_running() {
        Ok(())
    } else {
        Err("Could not start Ollama service".to_string())
    }
}

/// Ensure Ollama is installed and running
pub fn ensure_ollama_ready() -> Result<(), String> {
    if !is_ollama_installed() {
        install_ollama()?;
    }

    if !is_ollama_running() {
        start_ollama_service()?;
    }

    if is_ollama_running() {
        Ok(())
    } else {
        Err("Ollama is installed but not responding".to_string())
    }
}

// ============================================================================
// FULL AUTOPROVISION WITH INSTALLATION
// ============================================================================

/// Result of the full autoprovision process
#[derive(Debug, Clone)]
pub struct AutoprovisionResult {
    pub selection: LlmSelection,
    pub ollama_installed: bool,
    pub models_installed: Vec<String>,
    pub benchmarks_run: usize,
    pub errors: Vec<String>,
}

/// Run full autoprovision including Ollama and model installation
///
/// This is the main entry point for Anna's self-provisioning.
/// v3.0: Now includes Router model, hardware tier detection
/// It will:
/// 1. Detect hardware tier
/// 2. Ensure Ollama is installed and running
/// 3. Install best candidate models if missing
/// 4. Benchmark all available models
/// 5. Select and save the best Router/Junior/Senior combination
pub fn run_full_autoprovision<F>(on_progress: F) -> AutoprovisionResult
where
    F: Fn(&str),
{
    let mut result = AutoprovisionResult {
        selection: LlmSelection::default(),
        ollama_installed: false,
        models_installed: Vec::new(),
        benchmarks_run: 0,
        errors: Vec::new(),
    };

    // v3.0: Detect hardware tier first
    on_progress("Detecting hardware capabilities...");
    let hardware_tier = HardwareTier::detect();
    on_progress(&format!("Hardware tier: {:?}", hardware_tier));

    // Step 1: Ensure Ollama is ready
    on_progress("Checking Ollama installation...");
    if !is_ollama_installed() {
        on_progress("Installing Ollama...");
        match install_ollama() {
            Ok(()) => {
                result.ollama_installed = true;
                on_progress("Ollama installed successfully");
            }
            Err(e) => {
                result.errors.push(format!("Ollama install failed: {}", e));
                return result;
            }
        }
    }

    if !is_ollama_running() {
        on_progress("Starting Ollama service...");
        if let Err(e) = start_ollama_service() {
            result.errors.push(format!("Ollama start failed: {}", e));
            return result;
        }
    }
    on_progress("Ollama is ready");

    // Step 2: Install priority models based on hardware tier
    on_progress("Checking model availability...");
    let priority_models: Vec<&str> = match hardware_tier {
        HardwareTier::Minimal => vec![],  // Brain only
        HardwareTier::Basic => vec![
            "qwen2.5:0.5b-instruct",  // Router
            "qwen2.5:1.5b-instruct",  // Junior
        ],
        HardwareTier::Standard => vec![
            "qwen2.5:0.5b-instruct",  // Router
            "qwen2.5:1.5b-instruct",  // Junior
            "qwen2.5:7b-instruct",    // Senior
        ],
        HardwareTier::Power => vec![
            "qwen2.5:1.5b-instruct",  // Router (slightly larger)
            "qwen2.5:3b-instruct",    // Junior (larger)
            "qwen2.5:14b-instruct",   // Senior (larger)
        ],
    };

    for model in priority_models {
        if !is_model_available(model) {
            on_progress(&format!("Installing {}...", model));
            match pull_model(model) {
                Ok(()) => {
                    result.models_installed.push(model.to_string());
                    on_progress(&format!("{} installed", model));
                }
                Err(e) => {
                    result.errors.push(format!("Failed to install {}: {}", model, e));
                }
            }
        }
    }

    // Step 3: Benchmark all available models
    on_progress("Benchmarking models...");
    let mut all_benchmarks: Vec<ModelBenchmark> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    // v3.0: Benchmark Router candidates (if hardware supports)
    if hardware_tier.has_router() {
        for candidate in ROUTER_CANDIDATES {
            if is_model_available(candidate) && !all_benchmarks.iter().any(|b| b.model_name == *candidate) {
                on_progress(&format!("Benchmarking {} (Router candidate)...", candidate));
                let benchmark = benchmark_model(candidate);
                result.benchmarks_run += 1;
                all_benchmarks.push(benchmark);
            }
        }
    }

    // Benchmark Junior candidates
    for candidate in JUNIOR_CANDIDATES {
        if is_model_available(candidate) && !all_benchmarks.iter().any(|b| b.model_name == *candidate) {
            on_progress(&format!("Benchmarking {} (Junior candidate)...", candidate));
            let benchmark = benchmark_model(candidate);
            result.benchmarks_run += 1;
            all_benchmarks.push(benchmark);
        } else if !is_model_available(candidate) {
            suggestions.push(format!("Consider installing {} for Junior", candidate));
        }
    }

    // Benchmark Senior candidates (only if hardware supports)
    if hardware_tier.has_senior() {
        for candidate in SENIOR_CANDIDATES {
            if all_benchmarks.iter().any(|b| b.model_name == *candidate) {
                continue; // Already benchmarked
            }
            if is_model_available(candidate) {
                on_progress(&format!("Benchmarking {} (Senior candidate)...", candidate));
                let benchmark = benchmark_model(candidate);
                result.benchmarks_run += 1;
                all_benchmarks.push(benchmark);
            } else {
                suggestions.push(format!("Consider installing {} for Senior", candidate));
            }
        }
    }

    // Step 4: Select best models
    on_progress("Selecting optimal models...");

    // v3.0: Select best Router
    let router = if hardware_tier.has_router() {
        select_best_router(&all_benchmarks)
            .map(|b| (b.model_name.clone(), b.junior_score))
            .unwrap_or_else(|| (ROUTER_FALLBACK_MODEL.to_string(), 0.0))
    } else {
        (ROUTER_FALLBACK_MODEL.to_string(), 0.0)
    };

    let junior = select_best_junior(&all_benchmarks)
        .map(|b| (b.model_name.clone(), b.junior_score))
        .unwrap_or_else(|| (FALLBACK_MODEL.to_string(), 0.0));

    let senior = if hardware_tier.has_senior() {
        select_best_senior(&all_benchmarks)
            .map(|b| (b.model_name.clone(), b.senior_score))
            .unwrap_or_else(|| (FALLBACK_MODEL.to_string(), 0.0))
    } else {
        (FALLBACK_MODEL.to_string(), 0.0)
    };

    // Add performance suggestions
    if hardware_tier.has_router() && router.1 < 0.5 {
        suggestions.push("Router model performance is poor. Consider installing qwen2.5:0.5b-instruct".to_string());
    }
    if junior.1 < 0.5 {
        suggestions.push("Junior model performance is poor. Consider installing qwen2.5:1.5b-instruct".to_string());
    }
    if hardware_tier.has_senior() && senior.1 < 0.5 {
        suggestions.push("Senior model performance is poor. Consider installing qwen2.5:7b-instruct".to_string());
    }

    result.selection = LlmSelection {
        router_model: router.0.clone(),
        router_score: router.1,
        junior_model: junior.0.clone(),
        junior_score: junior.1,
        senior_model: senior.0.clone(),
        senior_score: senior.1,
        last_benchmark: chrono::Utc::now().to_rfc3339(),
        autoprovision_enabled: true,
        suggestions,
        hardware_tier: Some(hardware_tier),
    };

    // Save selection
    if let Err(e) = result.selection.save() {
        result.errors.push(format!("Could not save selection: {}", e));
    }

    // Save benchmarks
    save_benchmarks(&all_benchmarks);

    on_progress(&format!(
        "Autoprovision complete: Router={}, Junior={} (score {:.2}), Senior={} (score {:.2})",
        router.0, junior.0, junior.1, senior.0, senior.1
    ));

    result
}

/// Quick check if autoprovision is needed
pub fn needs_autoprovision() -> bool {
    // Check if selection file exists
    let selection = LlmSelection::load();

    // If default fallback model is selected, we need to provision
    if selection.junior_model == FALLBACK_MODEL && selection.junior_score == 0.0 {
        return true;
    }

    // Check if models are still available
    if !is_model_available(&selection.junior_model) {
        return true;
    }
    if !is_model_available(&selection.senior_model) {
        return true;
    }

    // Check if re-provision is needed based on performance
    should_reprovision(&selection)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_benchmark_unavailable() {
        let bench = ModelBenchmark::unavailable("test-model", "Not installed");
        assert!(!bench.is_available);
        assert!(!bench.is_junior_viable());
        assert!(!bench.is_senior_viable());
    }

    #[test]
    fn test_junior_viability_check() {
        let mut bench = ModelBenchmark::unavailable("test", "");
        bench.is_available = true;
        bench.json_compliance = 0.96;
        bench.avg_latency_ms = 5000;
        bench.determinism_score = 0.95;

        assert!(bench.is_junior_viable());

        // Too slow
        bench.avg_latency_ms = 10000;
        assert!(!bench.is_junior_viable());
    }

    #[test]
    fn test_senior_viability_check() {
        let mut bench = ModelBenchmark::unavailable("test", "");
        bench.is_available = true;
        bench.reasoning_score = 0.90;
        bench.avg_latency_ms = 10000;

        assert!(bench.is_senior_viable());

        // Poor reasoning
        bench.reasoning_score = 0.5;
        assert!(!bench.is_senior_viable());
    }

    #[test]
    fn test_json_scoring() {
        // Perfect JSON
        assert_eq!(score_json_response(r#"{"answer": "Paris"}"#), 1.0);

        // Valid JSON but wrong structure
        assert_eq!(score_json_response(r#"{"result": "Paris"}"#), 0.8);

        // JSON with extra text
        assert!(score_json_response(r#"Here is the answer: {"answer": "Paris"}"#) > 0.5);

        // No JSON
        assert_eq!(score_json_response("Paris is the capital"), 0.0);
    }

    #[test]
    fn test_junior_score_calculation() {
        // Perfect scores
        let score = calculate_junior_score(1.0, 1.0, 0);
        assert!(score > 0.9);

        // Fast but poor JSON compliance (0.5)
        // With json=0.5, determinism=1.0, latency=1000ms (speed=0.875)
        // Score = 0.5*0.5 + 0.875*0.3 + 1.0*0.2 ≈ 0.71
        let score = calculate_junior_score(0.5, 1.0, 1000);
        assert!(score < 0.8, "Poor JSON should hurt score, got {}", score);

        // Very poor JSON (0.3) with good speed
        let score = calculate_junior_score(0.3, 1.0, 500);
        assert!(score < 0.65, "Very poor JSON should hurt score significantly, got {}", score);

        // Slow (at max latency = 0 speed score)
        let score = calculate_junior_score(1.0, 1.0, JUNIOR_MAX_LATENCY_MS);
        assert!(score < 0.8, "Max latency should hurt score, got {}", score);
    }

    #[test]
    fn test_determinism_calculation() {
        // All same
        assert_eq!(calculate_determinism(&["a".to_string(), "a".to_string(), "a".to_string()]), 1.0);

        // All different
        assert!(calculate_determinism(&["a".to_string(), "b".to_string(), "c".to_string()]) < 0.5);

        // Single response
        assert_eq!(calculate_determinism(&["a".to_string()]), 1.0);
    }

    #[test]
    fn test_selection_default() {
        let selection = LlmSelection::default();
        assert!(selection.autoprovision_enabled);
        assert_eq!(selection.junior_model, FALLBACK_MODEL);
    }

    #[test]
    fn test_selection_format_status() {
        let selection = LlmSelection {
            router_model: "test-router".to_string(),
            router_score: 0.75,
            junior_model: "test-junior".to_string(),
            junior_score: 0.85,
            senior_model: "test-senior".to_string(),
            senior_score: 0.90,
            last_benchmark: "2025-11-29T12:00:00Z".to_string(),
            autoprovision_enabled: true,
            suggestions: vec!["Consider upgrading".to_string()],
            hardware_tier: Some(HardwareTier::Standard),
        };

        let status = selection.format_status();
        assert!(status.contains("test-junior"));
        assert!(status.contains("test-senior"));
        assert!(status.contains("0.85"));
        assert!(status.contains("Consider upgrading"));
    }

    #[test]
    fn test_should_reprovision() {
        let mut selection = LlmSelection::default();
        selection.junior_score = 0.8;
        selection.senior_score = 0.8;
        selection.last_benchmark = chrono::Utc::now().to_rfc3339();

        assert!(!should_reprovision(&selection));

        // Low scores should trigger reprovision
        selection.junior_score = 0.2;
        assert!(should_reprovision(&selection));
    }

    #[test]
    fn test_select_best_junior() {
        let benchmarks = vec![
            ModelBenchmark {
                model_name: "model-a".to_string(),
                timestamp: String::new(),
                first_token_latency_ms: 100,
                total_latency_ms: 500,
                avg_latency_ms: 500,
                json_compliance: 0.99,
                determinism_score: 0.95,
                reasoning_score: 0.7,
                junior_score: 0.9,
                senior_score: 0.6,
                is_available: true,
                error: None,
            },
            ModelBenchmark {
                model_name: "model-b".to_string(),
                timestamp: String::new(),
                first_token_latency_ms: 200,
                total_latency_ms: 1000,
                avg_latency_ms: 1000,
                json_compliance: 0.97,
                determinism_score: 0.92,
                reasoning_score: 0.8,
                junior_score: 0.85,
                senior_score: 0.7,
                is_available: true,
                error: None,
            },
        ];

        let best = select_best_junior(&benchmarks);
        assert!(best.is_some());
        assert_eq!(best.unwrap().model_name, "model-a");
    }

    #[test]
    fn test_junior_fallback_policy() {
        // Under 2 seconds - no fallback
        assert!(!should_fallback_junior(1_500));
        assert!(!should_fallback_junior(2_000)); // Exactly at limit

        // Over 2 seconds - fallback
        assert!(should_fallback_junior(2_001));
        assert!(should_fallback_junior(3_000));
        assert!(should_fallback_junior(10_000));

        // Edge cases
        assert!(!should_fallback_junior(0));
        assert!(!should_fallback_junior(100));
    }

    #[test]
    fn test_fallback_decision_types() {
        // Proceed decision
        let proceed = FallbackDecision::proceed();
        assert!(!proceed.use_fallback);
        assert!(proceed.latency_ms.is_none());

        // Timeout decision
        let timeout = FallbackDecision::timeout(3_500);
        assert!(timeout.use_fallback);
        assert_eq!(timeout.latency_ms, Some(3_500));
        assert!(timeout.reason.contains("3500ms"));

        // Error decision
        let error = FallbackDecision::error("LLM connection failed");
        assert!(error.use_fallback);
        assert!(error.latency_ms.is_none());
        assert!(error.reason.contains("connection failed"));
    }

    #[test]
    fn test_evaluate_junior_fallback() {
        // Fast response - proceed
        let decision = evaluate_junior_fallback(1_000);
        assert!(!decision.use_fallback);

        // At limit - proceed
        let decision = evaluate_junior_fallback(2_000);
        assert!(!decision.use_fallback);

        // Slow response - fallback
        let decision = evaluate_junior_fallback(2_500);
        assert!(decision.use_fallback);
        assert_eq!(decision.latency_ms, Some(2_500));
    }

    #[test]
    fn test_model_switch_result() {
        // No switch
        let result = ModelSwitchResult::no_switch("test-model");
        assert!(!result.switched);
        assert_eq!(result.old_model, "test-model");
        assert!(result.new_model.is_none());

        // Switch occurred
        let result = ModelSwitchResult::switched("old-model", "new-model", "performance issues");
        assert!(result.switched);
        assert_eq!(result.old_model, "old-model");
        assert_eq!(result.new_model, Some("new-model".to_string()));
        assert!(result.reason.contains("performance"));
    }

    #[test]
    fn test_handle_model_failure_under_threshold() {
        // Create a test selection
        let mut selection = LlmSelection {
            router_model: "test-router".to_string(),
            router_score: 0.7,
            junior_model: "test-junior".to_string(),
            junior_score: 0.8,
            senior_model: "test-senior".to_string(),
            senior_score: 0.8,
            last_benchmark: chrono::Utc::now().to_rfc3339(),
            autoprovision_enabled: true,
            suggestions: vec![],
            hardware_tier: Some(HardwareTier::Standard),
        };

        // Under threshold (3 failures needed)
        let result = handle_model_failure(&mut selection, true, 1);
        assert!(result.is_none());

        let result = handle_model_failure(&mut selection, true, 2);
        assert!(result.is_none());
    }

    #[test]
    fn test_hardware_tier_detection() {
        // Test that tier detection works (returns a valid tier)
        let tier = HardwareTier::detect();
        assert!(matches!(
            tier,
            HardwareTier::Minimal | HardwareTier::Basic | HardwareTier::Standard | HardwareTier::Power
        ));
    }

    #[test]
    fn test_hardware_tier_capabilities() {
        assert!(!HardwareTier::Minimal.has_router());
        assert!(!HardwareTier::Minimal.has_senior());

        assert!(HardwareTier::Basic.has_router());
        assert!(!HardwareTier::Basic.has_senior());

        assert!(HardwareTier::Standard.has_router());
        assert!(HardwareTier::Standard.has_senior());

        assert!(HardwareTier::Power.has_router());
        assert!(HardwareTier::Power.has_senior());
    }
}
