//! LLM Client v0.18.0
//!
//! Handles communication with Ollama for the v0.18.0 step-by-step protocol.
//! Parses Junior steps and Senior responses from JSON.

use anna_common::{
    generate_junior_prompt, generate_senior_prompt, JuniorScores, JuniorStep, SeniorResponse,
    SeniorScores, LLM_A_SYSTEM_PROMPT_V18, LLM_B_SYSTEM_PROMPT_V18,
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error, info, warn};

const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_JUNIOR_MODEL: &str = "qwen3:4b";
const DEFAULT_SENIOR_MODEL: &str = "qwen3:8b";
const DEFAULT_KEEP_ALIVE: &str = "5m";

/// Check if debug mode is enabled
fn is_debug_mode() -> bool {
    std::env::var("ANNA_DEBUG").is_ok()
}

/// Print debug output
fn print_debug(role: &str, model: &str, direction: &str, content: &str) {
    use std::io::Write;
    let mut stderr = std::io::stderr();

    let _ = writeln!(stderr);
    let _ = writeln!(stderr, "[{} {}] {} :", role, model, direction);
    let _ = writeln!(stderr, "------------------------------------------------------------");

    // Try to pretty-print JSON
    if let Ok(v) = serde_json::from_str::<Value>(content) {
        if let Ok(pretty) = serde_json::to_string_pretty(&v) {
            for line in pretty.lines().take(50) {
                let _ = writeln!(stderr, "  {}", line);
            }
        } else {
            for line in content.lines().take(30) {
                let _ = writeln!(stderr, "  {}", line);
            }
        }
    } else {
        for line in content.lines().take(30) {
            let _ = writeln!(stderr, "  {}", line);
        }
    }
    let _ = writeln!(stderr, "------------------------------------------------------------");
    let _ = stderr.flush();
}

/// LLM Client for v0.18.0 protocol
pub struct LlmClientV18 {
    http_client: reqwest::Client,
    junior_model: String,
    senior_model: String,
    keep_alive: String,
}

impl LlmClientV18 {
    /// Create client with role-specific models
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

    /// Get junior model name
    pub fn junior_model(&self) -> &str {
        &self.junior_model
    }

    /// Get senior model name
    pub fn senior_model(&self) -> &str {
        &self.senior_model
    }

    /// Call Junior (LLM-A) for next step
    pub async fn call_junior(
        &self,
        question: &str,
        available_probes: &[String],
        history: &str,
        iteration: usize,
    ) -> Result<JuniorStep> {
        let prompt = generate_junior_prompt(question, available_probes, history, iteration);

        if is_debug_mode() {
            print_debug("JUNIOR", &self.junior_model, "PROMPT", &prompt);
        }

        let response = self
            .call_ollama(&self.junior_model, LLM_A_SYSTEM_PROMPT_V18, &prompt)
            .await?;

        if is_debug_mode() {
            print_debug("JUNIOR", &self.junior_model, "RESPONSE", &response);
        }

        self.parse_junior_step(&response)
    }

    /// Call Senior (LLM-B) for review
    pub async fn call_senior(
        &self,
        question: &str,
        history: &str,
        junior_draft: Option<&str>,
        junior_scores: Option<&JuniorScores>,
        escalation_reason: &str,
    ) -> Result<SeniorResponse> {
        let scores_json = junior_scores
            .map(|s| serde_json::to_string_pretty(s).unwrap_or_default());

        let prompt = generate_senior_prompt(
            question,
            history,
            junior_draft,
            scores_json.as_deref(),
            escalation_reason,
        );

        if is_debug_mode() {
            print_debug("SENIOR", &self.senior_model, "PROMPT", &prompt);
        }

        let response = self
            .call_ollama(&self.senior_model, LLM_B_SYSTEM_PROMPT_V18, &prompt)
            .await?;

        if is_debug_mode() {
            print_debug("SENIOR", &self.senior_model, "RESPONSE", &response);
        }

        self.parse_senior_response(&response)
    }

    /// Raw Ollama API call
    async fn call_ollama(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let url = format!("{}/api/chat", OLLAMA_URL);

        let request = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "stream": false,
            "format": "json",
            "keep_alive": self.keep_alive
        });

        info!("LLM CALL [{}]", model);

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
            anyhow::bail!("Ollama error {}: {}", status, error_text);
        }

        let chat_response: Value = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let content = chat_response["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    /// Parse Junior's step response
    fn parse_junior_step(&self, text: &str) -> Result<JuniorStep> {
        let json_text = self.extract_json(text);

        let v: Value = serde_json::from_str(&json_text)
            .context("Failed to parse Junior response as JSON")?;

        let action = v["action"].as_str().unwrap_or("unknown");

        match action {
            "run_probe" => Ok(JuniorStep::RunProbe {
                probe_id: v["probe_id"].as_str().unwrap_or("").to_string(),
                reason: v["reason"].as_str().unwrap_or("").to_string(),
            }),

            "run_command" => Ok(JuniorStep::RunCommand {
                cmd: v["cmd"].as_str().unwrap_or("").to_string(),
                reason: v["reason"].as_str().unwrap_or("").to_string(),
            }),

            "ask_clarification" => Ok(JuniorStep::AskClarification {
                question: v["question"].as_str().unwrap_or("").to_string(),
            }),

            "propose_answer" => {
                let scores = self.parse_junior_scores(&v["scores"]);
                let citations: Vec<String> = v["citations"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(JuniorStep::ProposeAnswer {
                    text: v["text"].as_str().unwrap_or("").to_string(),
                    citations,
                    scores,
                    ready_for_user: v["ready_for_user"].as_bool().unwrap_or(false),
                })
            }

            "escalate_to_senior" => {
                // Parse escalation summary
                let summary = &v["summary"];
                Ok(JuniorStep::EscalateToSenior {
                    summary: anna_common::EscalationSummary {
                        original_question: summary["original_question"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        probes_run: vec![], // Simplified for now
                        draft_answer: summary["draft_answer"].as_str().map(|s| s.to_string()),
                        draft_scores: None,
                        reason_for_escalation: summary["reason_for_escalation"]
                            .as_str()
                            .unwrap_or("escalation requested")
                            .to_string(),
                        request: anna_common::EscalationRequest::AuditAnswer,
                    },
                })
            }

            _ => {
                warn!("Unknown Junior action: {}", action);
                // Default to requesting a probe if we can't parse
                Ok(JuniorStep::RunProbe {
                    probe_id: "cpu.info".to_string(),
                    reason: "parse error fallback".to_string(),
                })
            }
        }
    }

    /// Parse Junior's scores from JSON
    fn parse_junior_scores(&self, v: &Value) -> JuniorScores {
        JuniorScores {
            evidence: v["evidence"].as_u64().unwrap_or(50) as u8,
            reasoning: v["reasoning"].as_u64().unwrap_or(50) as u8,
            coverage: v["coverage"].as_u64().unwrap_or(50) as u8,
            overall: v["overall"].as_u64().unwrap_or(50) as u8,
        }
    }

    /// Parse Senior's response
    fn parse_senior_response(&self, text: &str) -> Result<SeniorResponse> {
        let json_text = self.extract_json(text);

        let v: Value = serde_json::from_str(&json_text)
            .context("Failed to parse Senior response as JSON")?;

        let action = v["action"].as_str().unwrap_or("approve_answer");

        match action {
            "approve_answer" => {
                let scores = self.parse_senior_scores(&v["scores"]);
                Ok(SeniorResponse::ApproveAnswer { scores })
            }

            "correct_answer" => {
                let scores = self.parse_senior_scores(&v["scores"]);
                let corrections: Vec<String> = v["corrections"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(SeniorResponse::CorrectAnswer {
                    text: v["text"].as_str().unwrap_or("").to_string(),
                    scores,
                    corrections,
                })
            }

            "request_probe" => Ok(SeniorResponse::RequestProbe {
                probe_id: v["probe_id"].as_str().unwrap_or("").to_string(),
                reason: v["reason"].as_str().unwrap_or("").to_string(),
            }),

            "request_command" => Ok(SeniorResponse::RequestCommand {
                cmd: v["cmd"].as_str().unwrap_or("").to_string(),
                reason: v["reason"].as_str().unwrap_or("").to_string(),
            }),

            "refuse" => {
                let probes: Vec<String> = v["probes_attempted"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(SeniorResponse::Refuse {
                    reason: v["reason"].as_str().unwrap_or("").to_string(),
                    probes_attempted: probes,
                })
            }

            _ => {
                // v0.73.0: Parse failures must NOT rubber-stamp approval
                // Return Refuse instead of silently approving
                error!("Senior response parse failed - refusing to rubber-stamp");
                Ok(SeniorResponse::Refuse {
                    reason: "Senior response could not be parsed - cannot verify answer".to_string(),
                    probes_attempted: vec![],
                })
            }
        }
    }

    /// Parse Senior's scores
    /// v0.73.0: Missing scores default to 0, not 70 - no rubber-stamping
    fn parse_senior_scores(&self, v: &Value) -> SeniorScores {
        SeniorScores::new(
            v["evidence"].as_u64().unwrap_or(0) as u8,
            v["reasoning"].as_u64().unwrap_or(0) as u8,
            v["coverage"].as_u64().unwrap_or(0) as u8,
            v["reliability_note"].as_str().unwrap_or("scores not provided by Senior"),
        )
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

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", OLLAMA_URL);
        self.http_client.get(&url).send().await.is_ok()
    }
}

impl Default for LlmClientV18 {
    fn default() -> Self {
        Self::with_role_models(None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LlmClientV18::default();
        assert_eq!(client.junior_model(), DEFAULT_JUNIOR_MODEL);
        assert_eq!(client.senior_model(), DEFAULT_SENIOR_MODEL);
    }

    #[test]
    fn test_parse_run_probe() {
        let client = LlmClientV18::default();
        let json = r#"{"action": "run_probe", "probe_id": "cpu.info", "reason": "need CPU"}"#;
        let step = client.parse_junior_step(json).unwrap();
        match step {
            JuniorStep::RunProbe { probe_id, reason } => {
                assert_eq!(probe_id, "cpu.info");
                assert_eq!(reason, "need CPU");
            }
            _ => panic!("Expected RunProbe"),
        }
    }

    #[test]
    fn test_parse_propose_answer() {
        let client = LlmClientV18::default();
        let json = r#"{
            "action": "propose_answer",
            "text": "You have 32 GB RAM",
            "citations": ["mem.info"],
            "scores": {"evidence": 90, "reasoning": 85, "coverage": 100, "overall": 85},
            "ready_for_user": true
        }"#;
        let step = client.parse_junior_step(json).unwrap();
        match step {
            JuniorStep::ProposeAnswer {
                text,
                citations,
                scores,
                ready_for_user,
            } => {
                assert_eq!(text, "You have 32 GB RAM");
                assert_eq!(citations, vec!["mem.info"]);
                assert_eq!(scores.overall, 85);
                assert!(ready_for_user);
            }
            _ => panic!("Expected ProposeAnswer"),
        }
    }

    #[test]
    fn test_parse_senior_approve() {
        let client = LlmClientV18::default();
        let json = r#"{
            "action": "approve_answer",
            "scores": {"evidence": 92, "reasoning": 90, "coverage": 95, "reliability_note": "strong evidence"}
        }"#;
        let resp = client.parse_senior_response(json).unwrap();
        match resp {
            SeniorResponse::ApproveAnswer { scores } => {
                assert_eq!(scores.overall, 90); // min of 92, 90, 95
            }
            _ => panic!("Expected ApproveAnswer"),
        }
    }
}
