//! Streaming LLM support for Ollama.
//!
//! Extracted from ollama.rs (v0.0.158) for modularization.
//! Provides word-by-word streaming output.

use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use std::time::Duration;
use tracing::{info, warn};

const OLLAMA_API: &str = "http://127.0.0.1:11434";

/// Send a chat request with streaming response, calling on_token for each token
/// This allows word-by-word display to the user
pub async fn chat_streaming<F>(
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    mut on_token: F,
) -> Result<String>
where
    F: FnMut(&str),
{
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
        return Err(anyhow!("Ollama request failed: {}", response.status()));
    }

    let mut full_response = String::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let bytes = chunk_result?;
        let text = String::from_utf8_lossy(&bytes);

        // Ollama sends newline-delimited JSON
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                // Extract token from response
                if let Some(token) = json.get("response").and_then(|r| r.as_str()) {
                    if !token.is_empty() {
                        on_token(token);
                        full_response.push_str(token);
                    }
                }

                // Check if generation is done
                if let Some(true) = json.get("done").and_then(|d| d.as_bool()) {
                    return Ok(full_response);
                }
            }
        }
    }

    Ok(full_response)
}

/// Streaming chat with retry logic (v0.0.143)
pub async fn chat_streaming_with_retry<F>(
    model: &str,
    prompt: &str,
    timeout_secs: u64,
    mut on_token: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    const MAX_RETRIES: u32 = 2;
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay_ms = 500 * (1 << (attempt - 1));
            info!("LLM streaming retry {} after {}ms delay", attempt, delay_ms);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        // For retries, we can't replay tokens, so just collect silently on retry
        if attempt == 0 {
            match chat_streaming(model, prompt, timeout_secs, &mut on_token).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("LLM streaming attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        } else {
            // On retry, fall back to non-streaming to avoid duplicate output
            match super::ollama::chat_single_attempt(model, prompt, timeout_secs).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("LLM retry attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("LLM streaming request failed after retries")))
}
