//! LLM Client v0.19.0 - Subproblem Decomposition
//!
//! Handles JSON parsing for Junior decomposition/work actions
//! and Senior mentor responses.

use anna_common::{
    JuniorStepV19, SeniorMentor, JuniorScoresV19, SeniorScoresV19,
    generate_junior_decomposition_prompt, generate_junior_work_prompt,
    generate_senior_mentor_prompt, generate_senior_review_prompt,
    LLM_A_SYSTEM_PROMPT_V19, LLM_B_SYSTEM_PROMPT_V19,
};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, warn};

/// Default model for Junior
const DEFAULT_JUNIOR_MODEL: &str = "llama3.2:3b";
/// Default model for Senior
const DEFAULT_SENIOR_MODEL: &str = "llama3.2:3b";
/// Ollama API endpoint
const OLLAMA_API: &str = "http://localhost:11434/api/generate";

/// LLM Client for v0.19.0 protocol
pub struct LlmClientV19 {
    client: Client,
    junior_model: String,
    senior_model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl LlmClientV19 {
    /// Create client with optional role-specific models
    pub fn with_role_models(junior_model: Option<String>, senior_model: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            junior_model: junior_model.unwrap_or_else(|| DEFAULT_JUNIOR_MODEL.to_string()),
            senior_model: senior_model.unwrap_or_else(|| DEFAULT_SENIOR_MODEL.to_string()),
        }
    }

    /// Get Junior model name
    pub fn junior_model(&self) -> &str {
        &self.junior_model
    }

    /// Get Senior model name
    pub fn senior_model(&self) -> &str {
        &self.senior_model
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        self.client
            .get("http://localhost:11434/api/tags")
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Call Junior for decomposition (first iteration)
    pub async fn call_junior_decompose(
        &self,
        question: &str,
        known_facts: &str,
        available_probes: &[String],
    ) -> Result<JuniorStepV19> {
        let prompt = generate_junior_decomposition_prompt(question, known_facts, available_probes);
        let response = self
            .call_ollama(&self.junior_model, LLM_A_SYSTEM_PROMPT_V19, &prompt)
            .await?;
        parse_junior_step(&response)
    }

    /// Call Junior for work action
    pub async fn call_junior_work(
        &self,
        question: &str,
        subproblems_json: &str,
        probe_history: &str,
        iteration: usize,
    ) -> Result<JuniorStepV19> {
        let prompt = generate_junior_work_prompt(question, subproblems_json, probe_history, iteration);
        let response = self
            .call_ollama(&self.junior_model, LLM_A_SYSTEM_PROMPT_V19, &prompt)
            .await?;
        parse_junior_step(&response)
    }

    /// Call Senior for mentoring
    pub async fn call_senior_mentor(
        &self,
        question: &str,
        mentor_context_json: &str,
        junior_question: &str,
    ) -> Result<SeniorMentor> {
        let prompt = generate_senior_mentor_prompt(question, mentor_context_json, junior_question);
        let response = self
            .call_ollama(&self.senior_model, LLM_B_SYSTEM_PROMPT_V19, &prompt)
            .await?;
        parse_senior_mentor(&response)
    }

    /// Call Senior for final review
    pub async fn call_senior_review(
        &self,
        question: &str,
        final_answer: &str,
        summaries_json: &str,
        scores_json: &str,
        probes_used: &str,
    ) -> Result<SeniorMentor> {
        let prompt = generate_senior_review_prompt(
            question,
            final_answer,
            summaries_json,
            scores_json,
            probes_used,
        );
        let response = self
            .call_ollama(&self.senior_model, LLM_B_SYSTEM_PROMPT_V19, &prompt)
            .await?;
        parse_senior_mentor(&response)
    }

    /// Make an Ollama API call
    async fn call_ollama(&self, model: &str, system: &str, prompt: &str) -> Result<String> {
        debug!("Calling Ollama model: {}", model);

        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: 0.3,
                num_predict: 2048,
            },
        };

        let response = self
            .client
            .post(OLLAMA_API)
            .json(&request)
            .send()
            .await
            .context("Failed to call Ollama")?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama returned error: {}", response.status());
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        debug!("Ollama response length: {}", ollama_resp.response.len());
        Ok(ollama_resp.response)
    }
}

impl Default for LlmClientV19 {
    fn default() -> Self {
        Self::with_role_models(None, None)
    }
}

/// Parse Junior's step response from JSON
fn parse_junior_step(response: &str) -> Result<JuniorStepV19> {
    // Extract JSON from response
    let json_str = extract_json(response);

    // Try to parse
    serde_json::from_str(&json_str)
        .or_else(|_| {
            // Try with lenient parsing
            parse_junior_step_lenient(&json_str)
        })
        .context("Failed to parse Junior step response")
}

/// Lenient parsing for Junior step
fn parse_junior_step_lenient(json_str: &str) -> Result<JuniorStepV19> {
    // Check for action type
    if json_str.contains("\"action\"") && json_str.contains("\"decompose\"") {
        // Parse as decomposition
        let decomposition = serde_json::from_str(json_str)?;
        Ok(JuniorStepV19::Decompose { decomposition })
    } else if json_str.contains("\"work_subproblem\"") {
        // Parse work subproblem
        #[derive(Deserialize)]
        struct WorkSubproblem {
            subproblem_id: String,
            probe_id: String,
            reason: Option<String>,
        }
        let parsed: WorkSubproblem = serde_json::from_str(json_str)?;
        Ok(JuniorStepV19::WorkSubproblem {
            subproblem_id: parsed.subproblem_id,
            probe_id: parsed.probe_id,
            reason: parsed.reason.unwrap_or_default(),
        })
    } else if json_str.contains("\"solve_subproblem\"") {
        #[derive(Deserialize)]
        struct SolveSubproblem {
            subproblem_id: String,
            partial_answer: String,
            confidence: Option<u8>,
        }
        let parsed: SolveSubproblem = serde_json::from_str(json_str)?;
        Ok(JuniorStepV19::SolveSubproblem {
            subproblem_id: parsed.subproblem_id,
            partial_answer: parsed.partial_answer,
            confidence: parsed.confidence.unwrap_or(70),
        })
    } else if json_str.contains("\"synthesize\"") {
        #[derive(Deserialize)]
        struct Synthesize {
            text: String,
            subproblem_summaries: Option<Vec<serde_json::Value>>,
            scores: Option<JuniorScoresV19>,
        }
        let parsed: Synthesize = serde_json::from_str(json_str)?;
        Ok(JuniorStepV19::Synthesize {
            text: parsed.text,
            subproblem_summaries: vec![],
            scores: parsed.scores.unwrap_or_default(),
        })
    } else {
        anyhow::bail!("Unrecognized Junior action format")
    }
}

/// Parse Senior's mentor response from JSON
fn parse_senior_mentor(response: &str) -> Result<SeniorMentor> {
    let json_str = extract_json(response);

    serde_json::from_str(&json_str)
        .or_else(|_| parse_senior_mentor_lenient(&json_str))
        .context("Failed to parse Senior mentor response")
}

/// Lenient parsing for Senior mentor
fn parse_senior_mentor_lenient(json_str: &str) -> Result<SeniorMentor> {
    if json_str.contains("\"approve_approach\"") {
        #[derive(Deserialize)]
        struct ApproveApproach {
            feedback: Option<String>,
        }
        let parsed: ApproveApproach = serde_json::from_str(json_str)?;
        Ok(SeniorMentor::ApproveApproach {
            feedback: parsed.feedback.unwrap_or_default(),
        })
    } else if json_str.contains("\"approve_answer\"") {
        #[derive(Deserialize)]
        struct ApproveAnswer {
            scores: Option<SeniorScoresV19>,
        }
        let parsed: ApproveAnswer = serde_json::from_str(json_str)?;
        Ok(SeniorMentor::ApproveAnswer {
            scores: parsed.scores.unwrap_or_default(),
        })
    } else if json_str.contains("\"correct_answer\"") {
        #[derive(Deserialize)]
        struct CorrectAnswer {
            corrected_text: String,
            corrections: Option<Vec<String>>,
            scores: Option<SeniorScoresV19>,
        }
        let parsed: CorrectAnswer = serde_json::from_str(json_str)?;
        Ok(SeniorMentor::CorrectAnswer {
            corrected_text: parsed.corrected_text,
            corrections: parsed.corrections.unwrap_or_default(),
            scores: parsed.scores.unwrap_or_default(),
        })
    } else if json_str.contains("\"refine_subproblems\"") {
        #[derive(Deserialize)]
        struct RefineSubproblems {
            feedback: Option<String>,
        }
        let parsed: RefineSubproblems = serde_json::from_str(json_str)?;
        Ok(SeniorMentor::RefineSubproblems {
            feedback: parsed.feedback.unwrap_or_default(),
            suggested_additions: vec![],
            suggested_removals: vec![],
            suggested_merges: vec![],
        })
    } else {
        // Default to approve approach
        warn!("Unrecognized Senior response, defaulting to approve");
        Ok(SeniorMentor::ApproveApproach {
            feedback: "Proceeding with current approach".to_string(),
        })
    }
}

/// Extract JSON object from LLM response
fn extract_json(response: &str) -> String {
    // Find first { and last }
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return response[start..=end].to_string();
            }
        }
    }
    // Return as-is if no JSON found
    response.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LlmClientV19::default();
        assert_eq!(client.junior_model(), DEFAULT_JUNIOR_MODEL);
        assert_eq!(client.senior_model(), DEFAULT_SENIOR_MODEL);
    }

    #[test]
    fn test_extract_json() {
        let response = "Here is the JSON: {\"action\": \"test\"} done.";
        let json = extract_json(response);
        assert_eq!(json, "{\"action\": \"test\"}");
    }

    #[test]
    fn test_parse_work_subproblem() {
        let json = r#"{"action": "work_subproblem", "subproblem_id": "sp1", "probe_id": "cpu.info", "reason": "test"}"#;
        let step = parse_junior_step(json).unwrap();
        match step {
            JuniorStepV19::WorkSubproblem { probe_id, .. } => {
                assert_eq!(probe_id, "cpu.info");
            }
            _ => panic!("Expected WorkSubproblem"),
        }
    }

    #[test]
    fn test_parse_approve_answer() {
        let json = r#"{"response": "approve_answer", "scores": {"evidence": 90, "reasoning": 85, "completeness": 80, "overall": 80, "reliability_note": "Good"}}"#;
        let response = parse_senior_mentor(json).unwrap();
        match response {
            SeniorMentor::ApproveAnswer { scores } => {
                assert_eq!(scores.overall, 80);
            }
            _ => panic!("Expected ApproveAnswer"),
        }
    }
}
