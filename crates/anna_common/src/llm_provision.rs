//! Adaptive LLM Provisioning v2.0
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
//!
//! ## Model Selection Flow
//!
//! ```text
//! First Launch / Reset
//!        ↓
//! List available Ollama models
//!        ↓
//! Benchmark each candidate
//!        ↓
//! Select best Junior (JSON-fast)
//! Select best Senior (reasoning)
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

/// Fallback model if nothing else works
pub const FALLBACK_MODEL: &str = "qwen2.5:1.5b-instruct";

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
    pub junior_model: String,
    pub junior_score: f64,
    pub senior_model: String,
    pub senior_score: f64,
    pub last_benchmark: String,
    pub autoprovision_enabled: bool,
    pub suggestions: Vec<String>,
}

impl Default for LlmSelection {
    fn default() -> Self {
        Self {
            junior_model: FALLBACK_MODEL.to_string(),
            junior_score: 0.0,
            senior_model: FALLBACK_MODEL.to_string(),
            senior_score: 0.0,
            last_benchmark: String::new(),
            autoprovision_enabled: true,
            suggestions: vec![],
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
        output.push_str("LLM AUTOPROVISION\n");
        output.push_str("──────────────────────────────────────────\n");
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

/// Run full autoprovision: benchmark all candidates and select best
pub fn run_autoprovision() -> LlmSelection {
    let available_models = list_ollama_models();
    let mut all_benchmarks: Vec<ModelBenchmark> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    // Benchmark Junior candidates
    for candidate in JUNIOR_CANDIDATES {
        let benchmark = benchmark_model(candidate);
        if !benchmark.is_available {
            suggestions.push(format!("Consider installing {} for Junior role", candidate));
        }
        all_benchmarks.push(benchmark);
    }

    // Benchmark Senior candidates
    for candidate in SENIOR_CANDIDATES {
        // Skip if already benchmarked as Junior candidate
        if all_benchmarks.iter().any(|b| b.model_name == *candidate) {
            continue;
        }
        let benchmark = benchmark_model(candidate);
        if !benchmark.is_available {
            suggestions.push(format!("Consider installing {} for Senior role", candidate));
        }
        all_benchmarks.push(benchmark);
    }

    // Select best models
    let junior = select_best_junior(&all_benchmarks)
        .map(|b| (b.model_name.clone(), b.junior_score))
        .unwrap_or_else(|| (FALLBACK_MODEL.to_string(), 0.0));

    let senior = select_best_senior(&all_benchmarks)
        .map(|b| (b.model_name.clone(), b.senior_score))
        .unwrap_or_else(|| (FALLBACK_MODEL.to_string(), 0.0));

    // Add performance suggestions
    if junior.1 < 0.5 {
        suggestions.push("Junior model performance is poor. Consider installing qwen2.5:1.5b-instruct".to_string());
    }
    if senior.1 < 0.5 {
        suggestions.push("Senior model performance is poor. Consider installing qwen2.5:7b-instruct".to_string());
    }

    let selection = LlmSelection {
        junior_model: junior.0,
        junior_score: junior.1,
        senior_model: senior.0,
        senior_score: senior.1,
        last_benchmark: chrono::Utc::now().to_rfc3339(),
        autoprovision_enabled: true,
        suggestions,
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
            junior_model: "test-junior".to_string(),
            junior_score: 0.85,
            senior_model: "test-senior".to_string(),
            senior_score: 0.90,
            last_benchmark: "2025-11-29T12:00:00Z".to_string(),
            autoprovision_enabled: true,
            suggestions: vec!["Consider upgrading".to_string()],
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
}
