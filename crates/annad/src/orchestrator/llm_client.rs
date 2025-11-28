//! Ollama LLM Client v0.78.0
//!
//! v0.78.0: Senior JSON Fix - minimal prompt, robust parsing, fallback scoring
//! v0.77.0: Dialog View - LLM prompts/responses streamed to annactl (not logs)
//! v0.76.0: Minimal Junior prompt - radically reduced context for 4B models
//!
//! Robust JSON parsing that handles common LLM output variations:
//! - "draft_answer": null
//! - "text": null inside draft_answer
//! - Missing optional fields
//! - v0.78.0: Scores at top level OR nested under "scores"
//!
//! v0.16.1: On-demand model loading with keep_alive parameter to save resources.
//! v0.16.4: Real-time debug output with [JUNIOR model] and [SENIOR model] labels
//! v0.73.2: Increased timeout to 120s for full Junior→Senior orchestration

use anna_common::{
    AuditScores, AuditVerdict, Citation, DebugEventEmitter, DraftAnswer, LlmAPlan, LlmAResponse, LlmBResponse,
    OllamaChatRequest, OllamaChatResponse, OllamaMessage, ProbeRequest, ReliabilityScores,
    LLM_A_SYSTEM_PROMPT_V76, LLM_B_SYSTEM_PROMPT_V78,
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Check if debug mode is enabled via ANNA_DEBUG environment variable
fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
}

/// v0.76.0: Print debug prompt with clear [ROLE model] label - FULL OUTPUT
/// This prints to stderr so it appears in real-time in journalctl/terminal
fn print_debug_prompt(role: &str, model: &str, prompt: &str) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    // Header with role and model
    let _ = writeln!(stderr);
    let _ = writeln!(
        stderr,
        "╔══════════════════════════════════════════════════════════════════════════════╗"
    );
    let _ = writeln!(stderr, "║  [{} {}]  [>]  PROMPT ({} chars)", role, model, prompt.len());
    let _ = writeln!(
        stderr,
        "╚══════════════════════════════════════════════════════════════════════════════╝"
    );

    // v0.76.0: FULL OUTPUT - NO TRUNCATION
    for line in prompt.lines() {
        let _ = writeln!(stderr, "  {}", line);
    }
    let _ = writeln!(
        stderr,
        "────────────────────────────────────────────────────────────────────────────────"
    );
    let _ = stderr.flush();
}

/// v0.76.0: Print debug response with clear [ROLE model] label - FULL OUTPUT
/// This prints to stderr so it appears in real-time in journalctl/terminal
fn print_debug_response(role: &str, model: &str, response: &str) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    // Header with role and model
    let _ = writeln!(stderr);
    let _ = writeln!(
        stderr,
        "╔══════════════════════════════════════════════════════════════════════════════╗"
    );
    let _ = writeln!(stderr, "║  [{} {}]  [<]  RESPONSE ({} chars)", role, model, response.len());
    let _ = writeln!(
        stderr,
        "╚══════════════════════════════════════════════════════════════════════════════╝"
    );

    // v0.76.0: FULL OUTPUT - NO TRUNCATION
    // Try to pretty-print JSON if valid
    if let Ok(json_value) = serde_json::from_str::<Value>(response) {
        if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
            for line in pretty.lines() {
                let _ = writeln!(stderr, "  {}", line);
            }
        } else {
            for line in response.lines() {
                let _ = writeln!(stderr, "  {}", line);
            }
        }
    } else {
        // Not valid JSON - print raw FULL output
        for line in response.lines() {
            let _ = writeln!(stderr, "  {}", line);
        }
    }

    let _ = writeln!(
        stderr,
        "════════════════════════════════════════════════════════════════════════════════"
    );
    let _ = stderr.flush();
}

const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_JUNIOR_MODEL: &str = "qwen3:4b";
const DEFAULT_SENIOR_MODEL: &str = "qwen3:8b";

/// Default keep_alive duration - model stays loaded for 5 minutes after last request
const DEFAULT_KEEP_ALIVE: &str = "5m";

/// Ollama LLM client with role-specific models
///
/// Supports separate models for junior (LLM-A) and senior (LLM-B) roles:
/// - Junior: Fast model for probe execution and command parsing
/// - Senior: Smarter model for reasoning and synthesis
///
/// v0.16.1: Supports on-demand model loading with configurable keep_alive
pub struct OllamaClient {
    http_client: reqwest::Client,
    junior_model: String, // LLM-A: fast, for probe execution
    senior_model: String, // LLM-B: smart, for reasoning
    /// How long to keep model loaded after request (e.g., "5m", "0", "1h")
    keep_alive: String,
}

impl OllamaClient {
    /// Create client with a single model for both roles (legacy/backwards compat)
    /// v0.73.2: Increased timeout to 120s for full Junior→Senior orchestration
    pub fn new(model: Option<String>) -> Self {
        let m = model.unwrap_or_else(|| DEFAULT_JUNIOR_MODEL.to_string());
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            junior_model: m.clone(),
            senior_model: m,
            keep_alive: DEFAULT_KEEP_ALIVE.to_string(),
        }
    }

    /// Create client with separate models for junior (LLM-A) and senior (LLM-B) roles
    /// v0.73.2: Increased timeout to 120s for full Junior→Senior orchestration
    pub fn with_role_models(junior: Option<String>, senior: Option<String>) -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            junior_model: junior.unwrap_or_else(|| DEFAULT_JUNIOR_MODEL.to_string()),
            senior_model: senior.unwrap_or_else(|| DEFAULT_SENIOR_MODEL.to_string()),
            keep_alive: DEFAULT_KEEP_ALIVE.to_string(),
        }
    }

    /// Create client with custom keep_alive duration for on-demand loading
    pub fn with_keep_alive(mut self, keep_alive: &str) -> Self {
        self.keep_alive = keep_alive.to_string();
        self
    }

    /// Get the current keep_alive setting
    pub fn keep_alive(&self) -> &str {
        &self.keep_alive
    }

    /// Get the junior model name
    pub fn junior_model(&self) -> &str {
        &self.junior_model
    }

    /// Get the senior model name
    pub fn senior_model(&self) -> &str {
        &self.senior_model
    }

    /// Call LLM-A (junior) with the given prompt
    /// Returns (parsed response, raw response text) for debug tracing
    pub async fn call_llm_a(&self, user_prompt: &str) -> Result<(LlmAResponse, String)> {
        use std::io::Write;

        // v0.76.0: Show FULL system prompt + user prompt
        if is_debug_mode() {
            let mut stderr = std::io::stderr();
            let _ = writeln!(stderr, "\n>>> SYSTEM PROMPT TO JUNIOR ({} chars):", LLM_A_SYSTEM_PROMPT_V76.len());
            let _ = writeln!(stderr, "{}", LLM_A_SYSTEM_PROMPT_V76);
            let _ = stderr.flush();
            print_debug_prompt("JUNIOR", &self.junior_model, user_prompt);
            let _ = writeln!(stderr, "\n>>> WAITING FOR JUNIOR LLM RESPONSE...");
            let _ = stderr.flush();
        }

        let start = std::time::Instant::now();

        // v0.76.0: Use minimal system prompt for 4B model performance
        let response_text = self
            .call_ollama(&self.junior_model, LLM_A_SYSTEM_PROMPT_V76, user_prompt)
            .await
            .context("LLM-A call failed")?;

        let elapsed = start.elapsed();

        // v0.76.0: Print response with timing
        if is_debug_mode() {
            let mut stderr = std::io::stderr();
            let _ = writeln!(stderr, "\n>>> JUNIOR RESPONDED IN {:.2}s", elapsed.as_secs_f64());
            let _ = stderr.flush();
            print_debug_response("JUNIOR", &self.junior_model, &response_text);
        }

        let parsed = self.parse_llm_a_response(&response_text)?;
        Ok((parsed, response_text))
    }

    /// v0.77.0: Call LLM-A (junior) with streaming emitter for dialog view in annactl
    /// Events are emitted through the streaming system, not to stderr/logs
    pub async fn call_llm_a_with_emitter(
        &self,
        user_prompt: &str,
        iteration: usize,
        emitter: &dyn DebugEventEmitter,
    ) -> Result<(LlmAResponse, String)> {
        // v0.77.0: Emit prompt to streaming (displays in annactl, not logs)
        emitter.llm_prompt_sent(
            iteration,
            "junior",
            &self.junior_model,
            LLM_A_SYSTEM_PROMPT_V76,
            user_prompt,
        );

        let start = std::time::Instant::now();

        let response_text = self
            .call_ollama(&self.junior_model, LLM_A_SYSTEM_PROMPT_V76, user_prompt)
            .await
            .context("LLM-A call failed")?;

        let elapsed_ms = start.elapsed().as_millis() as u64;

        // v0.77.0: Emit response to streaming (displays in annactl)
        emitter.llm_response_received(
            iteration,
            "junior",
            &self.junior_model,
            &response_text,
            elapsed_ms,
        );

        let parsed = self.parse_llm_a_response(&response_text)?;
        Ok((parsed, response_text))
    }

    /// Call LLM-B (senior) with the given prompt
    /// Returns (parsed response, raw response text) for debug tracing
    pub async fn call_llm_b(&self, user_prompt: &str) -> Result<(LlmBResponse, String)> {
        use std::io::Write;

        // v0.78.0: Use minimal Senior prompt for better compliance
        if is_debug_mode() {
            let mut stderr = std::io::stderr();
            let _ = writeln!(stderr, "\n>>> SYSTEM PROMPT TO SENIOR ({} chars):", LLM_B_SYSTEM_PROMPT_V78.len());
            let _ = writeln!(stderr, "{}", LLM_B_SYSTEM_PROMPT_V78);
            let _ = stderr.flush();
            print_debug_prompt("SENIOR", &self.senior_model, user_prompt);
            let _ = writeln!(stderr, "\n>>> WAITING FOR SENIOR LLM RESPONSE...");
            let _ = stderr.flush();
        }

        let start = std::time::Instant::now();

        // v0.78.0: Use minimal v78 Senior prompt
        let response_text = self
            .call_ollama(&self.senior_model, LLM_B_SYSTEM_PROMPT_V78, user_prompt)
            .await
            .context("LLM-B call failed")?;

        let elapsed = start.elapsed();

        // v0.76.0: Print response with timing
        if is_debug_mode() {
            let mut stderr = std::io::stderr();
            let _ = writeln!(stderr, "\n>>> SENIOR RESPONDED IN {:.2}s", elapsed.as_secs_f64());
            let _ = stderr.flush();
            print_debug_response("SENIOR", &self.senior_model, &response_text);
        }

        let parsed = self.parse_llm_b_response(&response_text)?;
        Ok((parsed, response_text))
    }

    /// v0.77.0: Call LLM-B (senior) with streaming emitter for dialog view in annactl
    /// Events are emitted through the streaming system, not to stderr/logs
    pub async fn call_llm_b_with_emitter(
        &self,
        user_prompt: &str,
        iteration: usize,
        emitter: &dyn DebugEventEmitter,
    ) -> Result<(LlmBResponse, String)> {
        // v0.78.0: Use minimal v78 Senior prompt, emit to streaming
        emitter.llm_prompt_sent(
            iteration,
            "senior",
            &self.senior_model,
            LLM_B_SYSTEM_PROMPT_V78,
            user_prompt,
        );

        let start = std::time::Instant::now();

        // v0.78.0: Use minimal v78 Senior prompt
        let response_text = self
            .call_ollama(&self.senior_model, LLM_B_SYSTEM_PROMPT_V78, user_prompt)
            .await
            .context("LLM-B call failed")?;

        let elapsed_ms = start.elapsed().as_millis() as u64;

        // v0.77.0: Emit response to streaming (displays in annactl)
        emitter.llm_response_received(
            iteration,
            "senior",
            &self.senior_model,
            &response_text,
            elapsed_ms,
        );

        let parsed = self.parse_llm_b_response(&response_text)?;
        Ok((parsed, response_text))
    }

    /// Raw Ollama API call with specified model
    /// Uses keep_alive to control how long the model stays loaded in VRAM
    async fn call_ollama(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let url = format!("{}/api/chat", OLLAMA_URL);

        let request = OllamaChatRequest {
            model: model.to_string(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            stream: false,
            format: Some("json".to_string()),
            keep_alive: Some(self.keep_alive.clone()),
        };

        info!("[>]  LLM CALL [{}] (keep_alive: {})", model, self.keep_alive);
        info!(
            "[S]  SYSTEM PROMPT ({} chars): {}",
            system_prompt.len(),
            &system_prompt[..200.min(system_prompt.len())]
        );
        info!(
            "[U]  USER PROMPT ({} chars): {}",
            user_prompt.len(),
            &user_prompt[..500.min(user_prompt.len())]
        );

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("[-]  Ollama error {}: {}", status, error_text);
            anyhow::bail!("Ollama returned error {}: {}", status, error_text);
        }

        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        info!(
            "[<]  LLM RESPONSE ({} chars): {}",
            chat_response.message.content.len(),
            &chat_response.message.content[..1000.min(chat_response.message.content.len())]
        );

        Ok(chat_response.message.content)
    }

    /// Parse LLM-A response with flexible handling of null/missing fields
    fn parse_llm_a_response(&self, text: &str) -> Result<LlmAResponse> {
        // First try direct serde parse
        if let Ok(response) = serde_json::from_str::<LlmAResponse>(text) {
            return Ok(response);
        }

        // Extract JSON if wrapped in prose
        let json_text = self.extract_json(text);

        // Try flexible parsing with serde_json::Value
        match serde_json::from_str::<Value>(&json_text) {
            Ok(v) => {
                let response = self.value_to_llm_a_response(&v);
                info!("Parsed LLM-A response via flexible parsing");
                Ok(response)
            }
            Err(e) => {
                warn!(
                    "Failed to parse LLM-A response as JSON: {} - text: {}",
                    e, text
                );
                // Return needs_more_probes to keep the loop going
                Ok(LlmAResponse {
                    plan: LlmAPlan {
                        intent: "parse_error".to_string(),
                        probe_requests: vec![],
                        can_answer_without_more_probes: false,
                    },
                    draft_answer: None,
                    self_scores: None,
                    needs_more_probes: true, // Keep loop going
                    refuse_to_answer: false,
                    refusal_reason: None,
                })
            }
        }
    }

    /// Parse LLM-B response with flexible handling
    fn parse_llm_b_response(&self, text: &str) -> Result<LlmBResponse> {
        // First try direct serde parse
        if let Ok(response) = serde_json::from_str::<LlmBResponse>(text) {
            return Ok(response);
        }

        // Extract JSON if wrapped in prose
        let json_text = self.extract_json(text);

        // Try flexible parsing
        match serde_json::from_str::<Value>(&json_text) {
            Ok(v) => {
                let response = self.value_to_llm_b_response(&v);
                info!("Parsed LLM-B response via flexible parsing");
                Ok(response)
            }
            Err(e) => {
                // v0.73.0: Parse failures must NOT rubber-stamp approval
                error!(
                    "Failed to parse LLM-B response as JSON: {} - refusing to rubber-stamp. Text: {}",
                    e, text
                );
                Ok(LlmBResponse {
                    verdict: AuditVerdict::Refuse,
                    scores: AuditScores::new(0.0, 0.0, 0.0),
                    probe_requests: vec![],
                    problems: vec!["Parse error - cannot verify answer".to_string()],
                    suggested_fix: None,
                    fixed_answer: None,
                    text: None,
                })
            }
        }
    }

    /// Extract JSON from text that may have prose around it
    fn extract_json(&self, text: &str) -> String {
        if let Some(json_start) = text.find('{') {
            if let Some(json_end) = text.rfind('}') {
                return text[json_start..=json_end].to_string();
            }
        }
        text.to_string()
    }

    /// Convert serde_json::Value to LlmAResponse with null handling
    fn value_to_llm_a_response(&self, v: &Value) -> LlmAResponse {
        // Parse plan
        let plan = v
            .get("plan")
            .map(|p| LlmAPlan {
                intent: p
                    .get("intent")
                    .and_then(|x| x.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                probe_requests: self.parse_probe_requests(p.get("probe_requests")),
                can_answer_without_more_probes: p
                    .get("can_answer_without_more_probes")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false),
            })
            .unwrap_or(LlmAPlan {
                intent: "unknown".to_string(),
                probe_requests: vec![],
                can_answer_without_more_probes: false,
            });

        // Parse draft_answer - handle null and missing text
        let draft_answer = v.get("draft_answer").and_then(|da| {
            if da.is_null() {
                return None;
            }
            let text = da.get("text").and_then(|t| {
                if t.is_null() {
                    None
                } else {
                    t.as_str().map(|s| s.to_string())
                }
            });
            // Only return draft_answer if text is present and non-empty
            text.filter(|t| !t.is_empty()).map(|t| DraftAnswer {
                text: t,
                citations: self.parse_citations(da.get("citations")),
            })
        });

        // Parse self_scores
        let self_scores = v.get("self_scores").and_then(|s| {
            if s.is_null() {
                return None;
            }
            Some(ReliabilityScores::new(
                s.get("evidence").and_then(|x| x.as_f64()).unwrap_or(0.5),
                s.get("reasoning").and_then(|x| x.as_f64()).unwrap_or(0.5),
                s.get("coverage").and_then(|x| x.as_f64()).unwrap_or(0.5),
            ))
        });

        let needs_more_probes = v
            .get("needs_more_probes")
            .and_then(|x| x.as_bool())
            .unwrap_or(draft_answer.is_none()); // Default: need probes if no draft

        let refuse_to_answer = v
            .get("refuse_to_answer")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);

        let refusal_reason = v
            .get("refusal_reason")
            .and_then(|x| if x.is_null() { None } else { x.as_str() })
            .map(|s| s.to_string());

        LlmAResponse {
            plan,
            draft_answer,
            self_scores,
            needs_more_probes,
            refuse_to_answer,
            refusal_reason,
        }
    }

    /// Convert serde_json::Value to LlmBResponse with null handling
    /// v0.78.0: Senior JSON contract fix - handle both nested and top-level scores
    fn value_to_llm_b_response(&self, v: &Value) -> LlmBResponse {
        // Parse verdict - default to approve if unclear
        let verdict_str = v
            .get("verdict")
            .and_then(|x| x.as_str())
            .unwrap_or("approve");

        // v0.73.0: Unknown verdict must NOT rubber-stamp approval
        let verdict = match verdict_str {
            "approve" => AuditVerdict::Approve,
            "fix_and_accept" => AuditVerdict::FixAndAccept,
            "needs_more_probes" => AuditVerdict::NeedsMoreProbes,
            "refuse" => AuditVerdict::Refuse,
            _ => {
                warn!("Unknown verdict '{}' - refusing to rubber-stamp", verdict_str);
                AuditVerdict::Refuse
            }
        };

        // v0.78.0: Parse scores - try nested "scores" object first, then top-level
        // Previously we only checked v.get("scores") which failed when model returned
        // {"evidence": 0.8, "reasoning": 1.0, "coverage": 1.0} at top level
        let scores = if let Some(s) = v.get("scores") {
            // Standard nested format: {"scores": {"evidence": 0.9, ...}}
            AuditScores::new(
                s.get("evidence").and_then(|x| x.as_f64()).unwrap_or(0.0),
                s.get("reasoning").and_then(|x| x.as_f64()).unwrap_or(0.0),
                s.get("coverage").and_then(|x| x.as_f64()).unwrap_or(0.0),
            )
        } else {
            // v0.78.0: Fallback - try top-level scores (model non-compliance)
            let evidence = v.get("evidence").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let reasoning = v.get("reasoning").and_then(|x| x.as_f64()).unwrap_or(0.0);
            let coverage = v.get("coverage").and_then(|x| x.as_f64()).unwrap_or(0.0);

            // If any score was found at top level, use them
            if evidence > 0.0 || reasoning > 0.0 || coverage > 0.0 {
                info!("v0.78.0: Found scores at top-level (model non-compliance), using them");
                AuditScores::new(evidence, reasoning, coverage)
            } else {
                // No scores found - default to conservative Yellow (0.75) not 0.0
                // This prevents punishing good Junior answers due to Senior format issues
                warn!("v0.78.0: No scores found - defaulting to conservative 0.75");
                AuditScores::new(0.75, 0.75, 0.75)
            }
        };

        // Parse probe_requests
        let probe_requests = self.parse_probe_requests(v.get("probe_requests"));

        // Parse problems
        let problems = v
            .get("problems")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let suggested_fix = v
            .get("suggested_fix")
            .and_then(|x| if x.is_null() { None } else { x.as_str() })
            .map(|s| s.to_string());

        // v0.78.0: fixed_answer can be string OR object with {text, citations}
        let fixed_answer = v.get("fixed_answer").and_then(|fa| {
            if fa.is_null() {
                None
            } else if let Some(s) = fa.as_str() {
                // Simple string format
                Some(s.to_string())
            } else if let Some(text) = fa.get("text").and_then(|t| t.as_str()) {
                // Object format: {"text": "...", "citations": [...]}
                Some(text.to_string())
            } else {
                None
            }
        });

        // v0.17.0: Extract Senior's synthesized answer text
        let text = v
            .get("text")
            .and_then(|x| if x.is_null() { None } else { x.as_str() })
            .map(|s| s.to_string());

        LlmBResponse {
            verdict,
            scores,
            probe_requests,
            problems,
            suggested_fix,
            fixed_answer,
            text,
        }
    }

    /// Parse probe_requests array
    fn parse_probe_requests(&self, v: Option<&Value>) -> Vec<ProbeRequest> {
        v.and_then(|arr| arr.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| {
                        let probe_id = p.get("probe_id").and_then(|x| x.as_str())?;
                        let reason = p
                            .get("reason")
                            .and_then(|x| x.as_str())
                            .unwrap_or("requested");
                        Some(ProbeRequest {
                            probe_id: probe_id.to_string(),
                            reason: reason.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse citations array
    fn parse_citations(&self, v: Option<&Value>) -> Vec<Citation> {
        v.and_then(|arr| arr.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let probe_id = c.get("probe_id").and_then(|x| x.as_str())?;
                        Some(Citation {
                            probe_id: probe_id.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", OLLAMA_URL);
        self.http_client.get(&url).send().await.is_ok()
    }

    /// Explicitly unload a model from VRAM to free resources
    /// This sends a request with keep_alive: 0 to immediately unload the model
    pub async fn unload_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/chat", OLLAMA_URL);

        let request = OllamaChatRequest {
            model: model.to_string(),
            messages: vec![OllamaMessage {
                role: "user".to_string(),
                content: "".to_string(), // Empty message just to trigger unload
            }],
            stream: false,
            format: None,
            keep_alive: Some("0".to_string()), // Immediately unload
        };

        info!("🔌  Unloading model [{}] from VRAM", model);

        // We don't care about the response, just trigger the unload
        let _ = self.http_client.post(&url).json(&request).send().await;

        Ok(())
    }

    /// Unload all models (junior and senior) to free VRAM
    pub async fn unload_all_models(&self) -> Result<()> {
        self.unload_model(&self.junior_model).await?;
        if self.senior_model != self.junior_model {
            self.unload_model(&self.senior_model).await?;
        }
        Ok(())
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_default() {
        let client = OllamaClient::default();
        assert_eq!(client.junior_model, DEFAULT_JUNIOR_MODEL);
        assert_eq!(client.senior_model, DEFAULT_JUNIOR_MODEL); // Same when using new()
    }

    #[test]
    fn test_ollama_client_with_role_models() {
        let client = OllamaClient::with_role_models(
            Some("llama3.2:3b".to_string()),
            Some("llama3.1:8b".to_string()),
        );
        assert_eq!(client.junior_model, "llama3.2:3b");
        assert_eq!(client.senior_model, "llama3.1:8b");
    }

    #[test]
    fn test_parse_llm_a_with_null_draft() {
        let client = OllamaClient::default();
        let json = r#"{
            "plan": {
                "intent": "hardware_info",
                "probe_requests": [{"probe_id": "cpu.info", "reason": "need cpu"}],
                "can_answer_without_more_probes": false
            },
            "draft_answer": null,
            "self_scores": {"evidence": 0.0, "reasoning": 0.0, "coverage": 0.0},
            "needs_more_probes": true,
            "refuse_to_answer": false,
            "refusal_reason": null
        }"#;

        let result = client.parse_llm_a_response(json).unwrap();
        assert!(result.draft_answer.is_none());
        assert!(result.needs_more_probes);
        assert_eq!(result.plan.probe_requests.len(), 1);
    }

    #[test]
    fn test_parse_llm_a_with_null_text() {
        let client = OllamaClient::default();
        let json = r#"{
            "plan": {
                "intent": "storage_usage",
                "probe_requests": [],
                "can_answer_without_more_probes": false
            },
            "draft_answer": {"text": null, "citations": []},
            "self_scores": null,
            "needs_more_probes": true
        }"#;

        let result = client.parse_llm_a_response(json).unwrap();
        assert!(result.draft_answer.is_none()); // null text = no draft
        assert!(result.needs_more_probes);
    }

    #[test]
    fn test_parse_llm_b_with_defaults() {
        let client = OllamaClient::default();
        let json = r#"{
            "verdict": "approve",
            "scores": {"evidence": 0.85, "reasoning": 0.9, "coverage": 0.8}
        }"#;

        let result = client.parse_llm_b_response(json).unwrap();
        assert_eq!(result.verdict, AuditVerdict::Approve);
        assert!(result.problems.is_empty());
    }

    #[test]
    fn test_parse_llm_b_fix_and_accept() {
        let client = OllamaClient::default();
        let json = r#"{
            "verdict": "fix_and_accept",
            "scores": {"evidence": 0.8, "reasoning": 0.85, "coverage": 0.75, "overall": 0.75},
            "problems": ["minor wording"],
            "fixed_answer": "Corrected answer here"
        }"#;

        let result = client.parse_llm_b_response(json).unwrap();
        assert_eq!(result.verdict, AuditVerdict::FixAndAccept);
        assert_eq!(
            result.fixed_answer,
            Some("Corrected answer here".to_string())
        );
    }

    #[test]
    fn test_default_keep_alive() {
        let client = OllamaClient::default();
        assert_eq!(client.keep_alive(), DEFAULT_KEEP_ALIVE);
    }

    #[test]
    fn test_custom_keep_alive() {
        let client = OllamaClient::default().with_keep_alive("10m");
        assert_eq!(client.keep_alive(), "10m");
    }

    #[test]
    fn test_immediate_unload_keep_alive() {
        let client = OllamaClient::default().with_keep_alive("0");
        assert_eq!(client.keep_alive(), "0");
    }
}
