//! Ollama management - install, run, and interact with Ollama.
//!
//! Includes hardware detection and automatic model selection.
//! v0.0.923: Configurable retry logic and improved error handling

mod hardware;
mod service;

pub use hardware::{detect_hardware, select_best_model, GpuType, HardwareInfo};
pub use service::{
    cleanup_anna_resources, delete_model, get_ollama_diagnostics, install, is_installed,
    is_running, list_models, pull_model, start_service, AnnaRegistry,
};

use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::core_loop::cache::get_perf_config;

pub(crate) const OLLAMA_API: &str = "http://127.0.0.1:11434";

/// Circuit breaker state for Ollama
static CONSECUTIVE_FAILURES: AtomicU32 = AtomicU32::new(0);
static CIRCUIT_OPENED_AT: AtomicU64 = AtomicU64::new(0);

/// Get current time as seconds since epoch
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// v0.0.923: Check if circuit breaker is open (uses config)
fn is_circuit_open() -> bool {
    let perf = get_perf_config();
    let threshold = perf.llm_circuit_threshold;
    let cooldown = perf.llm_circuit_cooldown_secs;

    let failures = CONSECUTIVE_FAILURES.load(Ordering::SeqCst);
    if failures >= threshold {
        let opened_at = CIRCUIT_OPENED_AT.load(Ordering::SeqCst);
        let now = now_secs();

        if now.saturating_sub(opened_at) < cooldown {
            return true;
        }

        // Half-open: allow one test request
        if CONSECUTIVE_FAILURES
            .compare_exchange(
                failures,
                threshold - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            info!("LLM circuit breaker half-open, allowing test request");
        }
    }
    false
}

/// Record a successful request
fn record_success() {
    let threshold = get_perf_config().llm_circuit_threshold;
    let prev = CONSECUTIVE_FAILURES.swap(0, Ordering::SeqCst);
    if prev >= threshold - 1 {
        info!("LLM circuit breaker closed after successful request");
    }
}

/// Record a failed request
fn record_failure() {
    let perf = get_perf_config();
    let failures = CONSECUTIVE_FAILURES.fetch_add(1, Ordering::SeqCst) + 1;
    if failures == perf.llm_circuit_threshold {
        CIRCUIT_OPENED_AT.store(now_secs(), Ordering::SeqCst);
        error!(
            "LLM circuit breaker OPEN - {} consecutive failures, cooling down for {}s",
            failures, perf.llm_circuit_cooldown_secs
        );
    }
}

/// v0.0.923: Check if an error is transient and retryable
fn is_transient_error(error: &anyhow::Error) -> bool {
    let err_str = error.to_string().to_lowercase();
    // Network/timeout errors are transient
    err_str.contains("timeout")
        || err_str.contains("connection")
        || err_str.contains("network")
        || err_str.contains("temporarily")
        || err_str.contains("503")  // Service unavailable
        || err_str.contains("502")  // Bad gateway
        || err_str.contains("504")  // Gateway timeout
        || err_str.contains("reset")
        || err_str.contains("broken pipe")
}

/// Send a chat request to Ollama with timeout and retry
/// v0.0.923: Uses config settings for retry logic
/// v0.0.933: Added memoization for repeated prompts
pub async fn chat_with_timeout(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
    use crate::core_loop::cache::{get_cached_llm_response, cache_llm_response};

    // v0.0.933: Check memoization cache first
    if let Some(cached) = get_cached_llm_response(prompt) {
        debug!("LLM memoization hit - skipping API call");
        return Ok(cached);
    }

    if is_circuit_open() {
        return Err(anyhow!(
            "Circuit breaker OPEN - Ollama is unavailable (too many failures). \
             Waiting for cooldown before retrying."
        ));
    }

    let perf = get_perf_config();
    let max_retries = perf.llm_max_retries;
    let base_delay_ms = perf.llm_retry_delay_ms;
    let mut last_error = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay_ms = base_delay_ms * (1 << (attempt - 1));
            debug!("LLM retry {} after {}ms delay", attempt, delay_ms);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        match chat_single_attempt(model, prompt, timeout_secs).await {
            Ok(response) => {
                record_success();
                // v0.0.933: Cache successful response
                cache_llm_response(prompt, &response);
                return Ok(response);
            }
            Err(e) => {
                let is_transient = is_transient_error(&e);
                if attempt < max_retries && is_transient {
                    debug!("LLM attempt {} failed (transient): {}", attempt + 1, e);
                } else if attempt < max_retries {
                    // Non-transient error, skip remaining retries
                    warn!("LLM request failed (non-transient): {}", e);
                    record_failure();
                    return Err(e);
                } else {
                    warn!("LLM attempt {} failed: {}", attempt + 1, e);
                }
                last_error = Some(e);
            }
        }
    }

    record_failure();
    Err(last_error.unwrap_or_else(|| anyhow!("LLM request failed after {} retries", max_retries)))
}

/// Single LLM request attempt
pub async fn chat_single_attempt(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false
    });

    let response = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!("Ollama request failed: {}", response.status()));
    }

    let json: serde_json::Value = response.json().await?;
    let response_text = json
        .get("response")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_string();

    Ok(response_text)
}

/// Streaming LLM request - writes tokens to the provided async writer
pub async fn chat_streaming_to_writer<W>(
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    writer: &mut W,
) -> Result<String>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    chat_streaming_validated(model, prompt, timeout_secs, "", writer).await
}

/// Streaming LLM request with validation (v0.0.889)
/// v0.0.923: Added retry logic for connection failures
pub async fn chat_streaming_validated<W>(
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    command_output: &str,
    writer: &mut W,
) -> Result<String>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    use anna_shared::rpc::StreamingResponse;
    use crate::validation::StreamingValidator;
    use futures_util::StreamExt;

    if is_circuit_open() {
        return Err(anyhow!(
            "Circuit breaker OPEN - Ollama is unavailable (too many failures). \
             Waiting for cooldown before retrying."
        ));
    }

    let perf = get_perf_config();
    let max_retries = perf.llm_max_retries;
    let base_delay_ms = perf.llm_retry_delay_ms;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": true
    });

    // v0.0.923: Retry logic for initial connection
    let mut last_error = None;
    let mut response_opt = None;

    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay_ms = base_delay_ms * (1 << (attempt - 1));
            debug!("Streaming LLM retry {} after {}ms delay", attempt, delay_ms);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        match client
            .post(format!("{}/api/generate", OLLAMA_API))
            .json(&body)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                response_opt = Some(resp);
                break;
            }
            Ok(resp) => {
                let status = resp.status();
                let err = anyhow!("Ollama request failed: {}", status);
                if attempt < max_retries && is_transient_error(&err) {
                    debug!("Streaming attempt {} failed (transient): {}", attempt + 1, status);
                    last_error = Some(err);
                } else {
                    record_failure();
                    return Err(err);
                }
            }
            Err(e) => {
                let err = anyhow::Error::from(e);
                if attempt < max_retries && is_transient_error(&err) {
                    debug!("Streaming attempt {} failed (transient): {}", attempt + 1, err);
                    last_error = Some(err);
                } else {
                    record_failure();
                    return Err(err);
                }
            }
        }
    }

    let response = match response_opt {
        Some(r) => r,
        None => {
            record_failure();
            return Err(last_error.unwrap_or_else(|| anyhow!("Streaming request failed after retries")));
        }
    };

    let mut full_response = String::new();
    let mut stream = response.bytes_stream();

    let mut validator = if !command_output.is_empty() {
        Some(StreamingValidator::new(command_output))
    } else {
        None
    };

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                record_failure();
                return Err(anyhow!("Stream error: {}", e));
            }
        };
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(token) = json.get("response").and_then(|r| r.as_str()) {
                    full_response.push_str(token);

                    let response = StreamingResponse::Token {
                        token: token.to_string(),
                    };
                    let json_str = serde_json::to_string(&response)?;
                    writer
                        .write_all(format!("{}\n", json_str).as_bytes())
                        .await?;
                    writer.flush().await?;

                    if let Some(ref mut v) = validator {
                        let warnings = v.add_token(token);
                        for warning in warnings {
                            let warning_response = StreamingResponse::Validation { warning };
                            let warning_json = serde_json::to_string(&warning_response)?;
                            writer
                                .write_all(format!("{}\n", warning_json).as_bytes())
                                .await?;
                            writer.flush().await?;
                        }
                    }
                }
            }
        }
    }

    record_success();
    Ok(full_response)
}
