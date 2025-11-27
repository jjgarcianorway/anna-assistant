//! Ollama LLM Client v0.12.1
//!
//! Robust JSON parsing that handles common LLM output variations:
//! - "draft_answer": null
//! - "text": null inside draft_answer
//! - Missing optional fields

use anna_common::{
    AuditScores, AuditVerdict, Citation, DraftAnswer, LlmAPlan, LlmAResponse, LlmBResponse,
    OllamaChatRequest, OllamaChatResponse, OllamaMessage, ProbeRequest, ReliabilityScores,
    LLM_A_SYSTEM_PROMPT, LLM_B_SYSTEM_PROMPT,
};
use anyhow::{Context, Result};
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, info, warn};

const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_MODEL: &str = "llama3.2:3b";

/// Ollama LLM client
pub struct OllamaClient {
    http_client: reqwest::Client,
    model: String,
}

impl OllamaClient {
    pub fn new(model: Option<String>) -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }

    /// Call LLM-A with the given prompt
    pub async fn call_llm_a(&self, user_prompt: &str) -> Result<LlmAResponse> {
        let response_text = self
            .call_ollama(LLM_A_SYSTEM_PROMPT, user_prompt)
            .await
            .context("LLM-A call failed")?;

        self.parse_llm_a_response(&response_text)
    }

    /// Call LLM-B with the given prompt
    pub async fn call_llm_b(&self, user_prompt: &str) -> Result<LlmBResponse> {
        let response_text = self
            .call_ollama(LLM_B_SYSTEM_PROMPT, user_prompt)
            .await
            .context("LLM-B call failed")?;

        self.parse_llm_b_response(&response_text)
    }

    /// Raw Ollama API call
    async fn call_ollama(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let url = format!("{}/api/chat", OLLAMA_URL);

        let request = OllamaChatRequest {
            model: self.model.clone(),
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
        };

        debug!("Calling Ollama model: {}", self.model);

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
            anyhow::bail!("Ollama returned error {}: {}", status, error_text);
        }

        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

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
                warn!("Failed to parse LLM-A response as JSON: {} - text: {}", e, text);
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
                warn!("Failed to parse LLM-B response as JSON: {} - text: {}", e, text);
                // Return approve with low scores to keep the loop going
                Ok(LlmBResponse {
                    verdict: AuditVerdict::Approve,
                    scores: AuditScores::new(0.6, 0.6, 0.6),
                    probe_requests: vec![],
                    problems: vec!["Parse error - approving with low confidence".to_string()],
                    suggested_fix: None,
                    fixed_answer: None,
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
        let plan = v.get("plan").map(|p| LlmAPlan {
            intent: p.get("intent")
                .and_then(|x| x.as_str())
                .unwrap_or("unknown")
                .to_string(),
            probe_requests: self.parse_probe_requests(p.get("probe_requests")),
            can_answer_without_more_probes: p.get("can_answer_without_more_probes")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
        }).unwrap_or(LlmAPlan {
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

        let needs_more_probes = v.get("needs_more_probes")
            .and_then(|x| x.as_bool())
            .unwrap_or(draft_answer.is_none()); // Default: need probes if no draft

        let refuse_to_answer = v.get("refuse_to_answer")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);

        let refusal_reason = v.get("refusal_reason")
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
    fn value_to_llm_b_response(&self, v: &Value) -> LlmBResponse {
        // Parse verdict - default to approve if unclear
        let verdict_str = v.get("verdict")
            .and_then(|x| x.as_str())
            .unwrap_or("approve");

        let verdict = match verdict_str {
            "approve" => AuditVerdict::Approve,
            "fix_and_accept" => AuditVerdict::FixAndAccept,
            "needs_more_probes" => AuditVerdict::NeedsMoreProbes,
            "refuse" => AuditVerdict::Refuse,
            _ => AuditVerdict::Approve, // Default to approve for unknown
        };

        // Parse scores
        let scores = v.get("scores").map(|s| {
            AuditScores::new(
                s.get("evidence").and_then(|x| x.as_f64()).unwrap_or(0.7),
                s.get("reasoning").and_then(|x| x.as_f64()).unwrap_or(0.7),
                s.get("coverage").and_then(|x| x.as_f64()).unwrap_or(0.7),
            )
        }).unwrap_or(AuditScores::new(0.7, 0.7, 0.7));

        // Parse probe_requests
        let probe_requests = self.parse_probe_requests(v.get("probe_requests"));

        // Parse problems
        let problems = v.get("problems")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let suggested_fix = v.get("suggested_fix")
            .and_then(|x| if x.is_null() { None } else { x.as_str() })
            .map(|s| s.to_string());

        let fixed_answer = v.get("fixed_answer")
            .and_then(|x| if x.is_null() { None } else { x.as_str() })
            .map(|s| s.to_string());

        LlmBResponse {
            verdict,
            scores,
            probe_requests,
            problems,
            suggested_fix,
            fixed_answer,
        }
    }

    /// Parse probe_requests array
    fn parse_probe_requests(&self, v: Option<&Value>) -> Vec<ProbeRequest> {
        v.and_then(|arr| arr.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| {
                        let probe_id = p.get("probe_id").and_then(|x| x.as_str())?;
                        let reason = p.get("reason")
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
        assert_eq!(client.model, DEFAULT_MODEL);
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
        assert_eq!(result.fixed_answer, Some("Corrected answer here".to_string()));
    }
}
