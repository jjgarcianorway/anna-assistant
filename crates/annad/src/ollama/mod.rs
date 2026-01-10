//! Ollama management - install, run, and interact with Ollama.
//!
//! Includes hardware detection and automatic model selection.

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
use tracing::{error, info, warn};

pub(crate) const OLLAMA_API: &str = "http://127.0.0.1:11434";

/// Circuit breaker state for Ollama
static CONSECUTIVE_FAILURES: AtomicU32 = AtomicU32::new(0);
static CIRCUIT_OPENED_AT: AtomicU64 = AtomicU64::new(0);

const CIRCUIT_OPEN_THRESHOLD: u32 = 3;
const CIRCUIT_COOLDOWN_SECS: u64 = 30;

/// Get current time as seconds since epoch
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Check if circuit breaker is open
fn is_circuit_open() -> bool {
    let failures = CONSECUTIVE_FAILURES.load(Ordering::SeqCst);
    if failures >= CIRCUIT_OPEN_THRESHOLD {
        let opened_at = CIRCUIT_OPENED_AT.load(Ordering::SeqCst);
        let now = now_secs();

        if now.saturating_sub(opened_at) < CIRCUIT_COOLDOWN_SECS {
            return true;
        }

        if CONSECUTIVE_FAILURES
            .compare_exchange(
                failures,
                CIRCUIT_OPEN_THRESHOLD - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            info!("Circuit breaker half-open, allowing test request");
        }
    }
    false
}

/// Record a successful request
fn record_success() {
    let prev = CONSECUTIVE_FAILURES.swap(0, Ordering::SeqCst);
    if prev >= CIRCUIT_OPEN_THRESHOLD - 1 {
        info!("Circuit breaker closed after successful request");
    }
}

/// Record a failed request
fn record_failure() {
    let failures = CONSECUTIVE_FAILURES.fetch_add(1, Ordering::SeqCst) + 1;
    if failures == CIRCUIT_OPEN_THRESHOLD {
        CIRCUIT_OPENED_AT.store(now_secs(), Ordering::SeqCst);
        error!(
            "Circuit breaker OPEN - Ollama has {} consecutive failures, cooling down for {}s",
            failures, CIRCUIT_COOLDOWN_SECS
        );
    }
}

/// Send a chat request to Ollama with timeout and retry
pub async fn chat_with_timeout(model: &str, prompt: &str, timeout_secs: u64) -> Result<String> {
    if is_circuit_open() {
        return Err(anyhow!(
            "Circuit breaker OPEN - Ollama is unavailable (too many failures). \
             Waiting for cooldown before retrying."
        ));
    }

    const MAX_RETRIES: u32 = 2;
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay_ms = 500 * (1 << (attempt - 1));
            info!("LLM retry {} after {}ms delay", attempt, delay_ms);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        match chat_single_attempt(model, prompt, timeout_secs).await {
            Ok(response) => {
                record_success();
                return Ok(response);
            }
            Err(e) => {
                warn!("LLM attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
            }
        }
    }

    record_failure();
    Err(last_error.unwrap_or_else(|| anyhow!("LLM request failed after retries")))
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

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": true
    });

    let response = client
        .post(format!("{}/api/generate", OLLAMA_API))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        record_failure();
        return Err(anyhow!("Ollama request failed: {}", response.status()));
    }

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
