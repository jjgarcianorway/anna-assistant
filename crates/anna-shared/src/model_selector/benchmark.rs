//! Model benchmarking (v0.0.223).

use super::types::ModelBenchmark;

/// Micro-benchmark prompt for quick performance measurement
/// Short enough to measure quickly, long enough to be meaningful
pub const BENCHMARK_PROMPT: &str =
    "Classify: 'how much RAM do I have?' Reply: system/network/storage";

/// Expected response length for benchmark (tokens)
pub const BENCHMARK_EXPECTED_TOKENS: u32 = 10;

/// Parse ollama benchmark response into ModelBenchmark
/// Use this with the result from an ollama /api/generate call
pub fn parse_benchmark_response(
    model: &str,
    response: &serde_json::Value,
    fallback_duration_ns: u64,
) -> ModelBenchmark {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Extract timing from ollama response
    // ollama returns: total_duration, load_duration, prompt_eval_duration, eval_duration in nanoseconds
    let ttft_ns = response
        .get("prompt_eval_duration")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let eval_ns = response
        .get("eval_duration")
        .and_then(|v| v.as_u64())
        .unwrap_or(fallback_duration_ns);
    let eval_count = response
        .get("eval_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    // Calculate tokens per second
    let eval_secs = eval_ns as f64 / 1_000_000_000.0;
    let tokens_per_sec = if eval_secs > 0.0 {
        eval_count as f32 / eval_secs as f32
    } else {
        0.0
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    ModelBenchmark {
        model: model.to_string(),
        tokens_per_sec,
        ttft_ms: (ttft_ns / 1_000_000),
        timestamp,
    }
}
