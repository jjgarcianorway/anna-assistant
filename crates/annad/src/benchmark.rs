//! Model micro-benchmark runner.
//!
//! v0.0.74: Quick benchmark to measure model performance.
//! Uses a tiny classification prompt to measure tokens/sec and TTFT.

use anna_shared::model_selector::{
    parse_benchmark_response, ModelBenchmark, BENCHMARK_EXPECTED_TOKENS, BENCHMARK_PROMPT,
};
use std::time::Instant;
use tracing::{debug, warn};

/// Run a micro-benchmark on a model via ollama
/// Returns ModelBenchmark with tokens/sec and time-to-first-token
pub async fn run_micro_benchmark(
    client: &reqwest::Client,
    ollama_url: &str,
    model: &str,
) -> Result<ModelBenchmark, String> {
    let url = format!("{}/api/generate", ollama_url);
    let body = serde_json::json!({
        "model": model,
        "prompt": BENCHMARK_PROMPT,
        "stream": false,
        "options": {
            "num_predict": BENCHMARK_EXPECTED_TOKENS,
        }
    });

    debug!("Running micro-benchmark on model: {}", model);
    let start = Instant::now();

    let resp = client
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("benchmark request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("benchmark failed: HTTP {}", resp.status()));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("benchmark parse failed: {}", e))?;

    let total_duration = start.elapsed();
    let benchmark = parse_benchmark_response(model, &result, total_duration.as_nanos() as u64);

    debug!(
        "Benchmark complete: {} - {:.1} tok/s, {} ms TTFT",
        model, benchmark.tokens_per_sec, benchmark.ttft_ms
    );

    Ok(benchmark)
}

/// Run benchmarks for all provided models
pub async fn benchmark_models(
    client: &reqwest::Client,
    ollama_url: &str,
    models: &[String],
) -> Vec<ModelBenchmark> {
    let mut results = Vec::new();

    for model in models {
        match run_micro_benchmark(client, ollama_url, model).await {
            Ok(bench) => results.push(bench),
            Err(e) => warn!("Benchmark failed for {}: {}", model, e),
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_benchmark_response() {
        let response = serde_json::json!({
            "prompt_eval_duration": 100_000_000u64,  // 100ms
            "eval_duration": 500_000_000u64,         // 500ms
            "eval_count": 10u64,
        });

        let bench = parse_benchmark_response("test-model", &response, 1_000_000_000);

        assert_eq!(bench.model, "test-model");
        assert!(bench.tokens_per_sec > 15.0 && bench.tokens_per_sec < 25.0); // ~20 tok/s
        assert_eq!(bench.ttft_ms, 100);
    }
}
