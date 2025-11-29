//! LLM Client Trait Abstraction v1.0.0
//!
//! See `docs/architecture.md` Section 9 for design rationale.
//!
//! This module provides a trait abstraction over LLM backends to enable:
//! - Deterministic testing with fake implementations
//! - Future backend flexibility (Ollama, remote APIs, etc.)
//! - Clear interface boundaries
//!
//! ## Usage
//!
//! Production code uses `OllamaClient` which implements `LlmClient`.
//! Test code uses `FakeLlmClient` with pre-configured responses.

use super::llm_client::{DraftAnswerV80, JuniorResponseV80, OllamaClient, SeniorResponseV80};
use anna_common::ProbeRequest;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

// ============================================================================
// LLM Client Trait
// ============================================================================

/// Trait abstraction for LLM backends
///
/// This trait defines the minimal interface needed by UnifiedEngine
/// to call Junior and Senior LLM roles.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Get the junior model name
    fn junior_model(&self) -> &str;

    /// Get the senior model name
    fn senior_model(&self) -> &str;

    /// Check if the LLM backend is available
    async fn is_available(&self) -> bool;

    /// Call Junior (LLM-A) with v0.80.0 razorback-fast minimal prompt
    ///
    /// Returns (parsed response, raw response text)
    async fn call_junior_v80(&self, user_prompt: &str) -> Result<(JuniorResponseV80, String)>;

    /// Call Senior (LLM-B) with v0.80.0 razorback-fast minimal prompt
    ///
    /// Returns (parsed response, raw response text)
    async fn call_senior_v80(&self, user_prompt: &str) -> Result<(SeniorResponseV80, String)>;
}

// ============================================================================
// OllamaClient Implementation
// ============================================================================

#[async_trait]
impl LlmClient for OllamaClient {
    fn junior_model(&self) -> &str {
        self.junior_model()
    }

    fn senior_model(&self) -> &str {
        self.senior_model()
    }

    async fn is_available(&self) -> bool {
        self.is_available().await
    }

    async fn call_junior_v80(&self, user_prompt: &str) -> Result<(JuniorResponseV80, String)> {
        self.call_junior_v80(user_prompt).await
    }

    async fn call_senior_v80(&self, user_prompt: &str) -> Result<(SeniorResponseV80, String)> {
        self.call_senior_v80(user_prompt).await
    }
}

// ============================================================================
// Fake LLM Client for Testing
// ============================================================================

/// Pre-configured response for fake Junior calls
#[derive(Debug, Clone)]
pub struct FakeJuniorResponse {
    pub probe_requests: Vec<ProbeRequest>,
    pub draft_answer: Option<DraftAnswerV80>,
    pub raw_text: String,
}

impl Default for FakeJuniorResponse {
    fn default() -> Self {
        Self {
            probe_requests: vec![],
            draft_answer: Some(DraftAnswerV80 {
                text: "Fake Junior answer".to_string(),
                citations: vec!["cpu.info".to_string()],
            }),
            raw_text: r#"{"probe_requests":[],"draft_answer":{"text":"Fake Junior answer","citations":["cpu.info"]}}"#.to_string(),
        }
    }
}

/// Pre-configured response for fake Senior calls
#[derive(Debug, Clone)]
pub struct FakeSeniorResponse {
    pub verdict: String,
    pub fixed_answer: Option<String>,
    pub scores_overall: f64,
    pub raw_text: String,
}

impl Default for FakeSeniorResponse {
    fn default() -> Self {
        Self {
            verdict: "approve".to_string(),
            fixed_answer: None,
            scores_overall: 0.95,
            raw_text: r#"{"verdict":"approve","scores":{"overall":0.95}}"#.to_string(),
        }
    }
}

/// Fake LLM client for deterministic testing
///
/// Provides pre-configured responses without making real LLM calls.
/// Use `FakeLlmClientBuilder` for convenient test setup.
///
/// ## Example
///
/// ```rust,ignore
/// let fake = FakeLlmClientBuilder::new()
///     .junior_response(FakeJuniorResponse {
///         probe_requests: vec![ProbeRequest { probe_id: "cpu.info".into(), reason: "test".into() }],
///         draft_answer: None,
///         raw_text: "{}".into(),
///     })
///     .senior_response(FakeSeniorResponse::default())
///     .build();
///
/// let (response, _) = fake.call_junior_v80("test").await.unwrap();
/// assert_eq!(response.probe_requests.len(), 1);
/// ```
pub struct FakeLlmClient {
    junior_model: String,
    senior_model: String,
    is_available: bool,
    /// Queue of Junior responses (consumed in order)
    junior_responses: Arc<Mutex<Vec<FakeJuniorResponse>>>,
    /// Queue of Senior responses (consumed in order)
    senior_responses: Arc<Mutex<Vec<FakeSeniorResponse>>>,
    /// Default Junior response when queue is empty
    default_junior: FakeJuniorResponse,
    /// Default Senior response when queue is empty
    default_senior: FakeSeniorResponse,
    /// Track call counts for assertions
    junior_call_count: Arc<Mutex<usize>>,
    senior_call_count: Arc<Mutex<usize>>,
}

impl FakeLlmClient {
    /// Create a new fake client with default responses
    pub fn new() -> Self {
        Self {
            junior_model: "fake-junior".to_string(),
            senior_model: "fake-senior".to_string(),
            is_available: true,
            junior_responses: Arc::new(Mutex::new(vec![])),
            senior_responses: Arc::new(Mutex::new(vec![])),
            default_junior: FakeJuniorResponse::default(),
            default_senior: FakeSeniorResponse::default(),
            junior_call_count: Arc::new(Mutex::new(0)),
            senior_call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get the number of Junior calls made
    pub fn junior_call_count(&self) -> usize {
        *self.junior_call_count.lock().unwrap()
    }

    /// Get the number of Senior calls made
    pub fn senior_call_count(&self) -> usize {
        *self.senior_call_count.lock().unwrap()
    }

    /// Reset call counts for reuse in multiple test scenarios
    pub fn reset_counts(&self) {
        *self.junior_call_count.lock().unwrap() = 0;
        *self.senior_call_count.lock().unwrap() = 0;
    }
}

impl Default for FakeLlmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmClient for FakeLlmClient {
    fn junior_model(&self) -> &str {
        &self.junior_model
    }

    fn senior_model(&self) -> &str {
        &self.senior_model
    }

    async fn is_available(&self) -> bool {
        self.is_available
    }

    async fn call_junior_v80(&self, _user_prompt: &str) -> Result<(JuniorResponseV80, String)> {
        // Increment call count
        *self.junior_call_count.lock().unwrap() += 1;

        // Try to pop from queue, otherwise use default
        let response = {
            let mut queue = self.junior_responses.lock().unwrap();
            if !queue.is_empty() {
                queue.remove(0)
            } else {
                self.default_junior.clone()
            }
        };

        Ok((
            JuniorResponseV80 {
                probe_requests: response.probe_requests,
                draft_answer: response.draft_answer,
            },
            response.raw_text,
        ))
    }

    async fn call_senior_v80(&self, _user_prompt: &str) -> Result<(SeniorResponseV80, String)> {
        // Increment call count
        *self.senior_call_count.lock().unwrap() += 1;

        // Try to pop from queue, otherwise use default
        let response = {
            let mut queue = self.senior_responses.lock().unwrap();
            if !queue.is_empty() {
                queue.remove(0)
            } else {
                self.default_senior.clone()
            }
        };

        Ok((
            SeniorResponseV80 {
                verdict: response.verdict,
                fixed_answer: response.fixed_answer,
                scores_overall: response.scores_overall,
            },
            response.raw_text,
        ))
    }
}

// ============================================================================
// Builder for FakeLlmClient
// ============================================================================

/// Builder for FakeLlmClient with convenient test setup
pub struct FakeLlmClientBuilder {
    junior_model: String,
    senior_model: String,
    is_available: bool,
    junior_responses: Vec<FakeJuniorResponse>,
    senior_responses: Vec<FakeSeniorResponse>,
    default_junior: FakeJuniorResponse,
    default_senior: FakeSeniorResponse,
}

impl FakeLlmClientBuilder {
    /// Create a new builder with defaults
    pub fn new() -> Self {
        Self {
            junior_model: "fake-junior".to_string(),
            senior_model: "fake-senior".to_string(),
            is_available: true,
            junior_responses: vec![],
            senior_responses: vec![],
            default_junior: FakeJuniorResponse::default(),
            default_senior: FakeSeniorResponse::default(),
        }
    }

    /// Set the junior model name
    pub fn junior_model(mut self, model: &str) -> Self {
        self.junior_model = model.to_string();
        self
    }

    /// Set the senior model name
    pub fn senior_model(mut self, model: &str) -> Self {
        self.senior_model = model.to_string();
        self
    }

    /// Set availability status
    pub fn available(mut self, available: bool) -> Self {
        self.is_available = available;
        self
    }

    /// Add a Junior response to the queue
    pub fn junior_response(mut self, response: FakeJuniorResponse) -> Self {
        self.junior_responses.push(response);
        self
    }

    /// Add a Senior response to the queue
    pub fn senior_response(mut self, response: FakeSeniorResponse) -> Self {
        self.senior_responses.push(response);
        self
    }

    /// Set the default Junior response (used when queue is empty)
    pub fn default_junior(mut self, response: FakeJuniorResponse) -> Self {
        self.default_junior = response;
        self
    }

    /// Set the default Senior response (used when queue is empty)
    pub fn default_senior(mut self, response: FakeSeniorResponse) -> Self {
        self.default_senior = response;
        self
    }

    /// Build the FakeLlmClient
    pub fn build(self) -> FakeLlmClient {
        FakeLlmClient {
            junior_model: self.junior_model,
            senior_model: self.senior_model,
            is_available: self.is_available,
            junior_responses: Arc::new(Mutex::new(self.junior_responses)),
            senior_responses: Arc::new(Mutex::new(self.senior_responses)),
            default_junior: self.default_junior,
            default_senior: self.default_senior,
            junior_call_count: Arc::new(Mutex::new(0)),
            senior_call_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl Default for FakeLlmClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper constructors for common test scenarios
// ============================================================================

impl FakeLlmClient {
    /// Create a fake that always returns a specific answer from Junior
    /// (no probes needed, direct answer)
    pub fn with_direct_answer(answer: &str, reliability: f64) -> Self {
        FakeLlmClientBuilder::new()
            .default_junior(FakeJuniorResponse {
                probe_requests: vec![],
                draft_answer: Some(DraftAnswerV80 {
                    text: answer.to_string(),
                    citations: vec![],
                }),
                raw_text: format!(r#"{{"draft_answer":{{"text":"{}","citations":[]}}}}"#, answer),
            })
            .default_senior(FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: reliability,
                raw_text: format!(r#"{{"verdict":"approve","scores":{{"overall":{}}}}}"#, reliability),
            })
            .build()
    }

    /// Create a fake that always requests a specific probe
    pub fn with_probe_request(probe_id: &str, reason: &str) -> Self {
        FakeLlmClientBuilder::new()
            .default_junior(FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: probe_id.to_string(),
                    reason: reason.to_string(),
                }],
                draft_answer: None,
                raw_text: format!(
                    r#"{{"probe_requests":[{{"probe_id":"{}","reason":"{}"}}]}}"#,
                    probe_id, reason
                ),
            })
            .build()
    }

    /// Create a fake that simulates an unavailable LLM backend
    pub fn unavailable() -> Self {
        FakeLlmClientBuilder::new().available(false).build()
    }

    /// Create a fake that simulates Senior refusing the answer
    pub fn with_senior_refuse(reason: &str) -> Self {
        FakeLlmClientBuilder::new()
            .default_senior(FakeSeniorResponse {
                verdict: "refuse".to_string(),
                fixed_answer: Some(reason.to_string()),
                scores_overall: 0.0,
                raw_text: format!(r#"{{"verdict":"refuse","fixed_answer":"{}","scores":{{"overall":0.0}}}}"#, reason),
            })
            .build()
    }

    /// Create a fake that simulates Senior fixing the answer
    pub fn with_senior_fix(original: &str, fixed: &str, reliability: f64) -> Self {
        FakeLlmClientBuilder::new()
            .default_junior(FakeJuniorResponse {
                probe_requests: vec![],
                draft_answer: Some(DraftAnswerV80 {
                    text: original.to_string(),
                    citations: vec![],
                }),
                raw_text: "{}".to_string(),
            })
            .default_senior(FakeSeniorResponse {
                verdict: "fix_and_accept".to_string(),
                fixed_answer: Some(fixed.to_string()),
                scores_overall: reliability,
                raw_text: format!(
                    r#"{{"verdict":"fix_and_accept","fixed_answer":"{}","scores":{{"overall":{}}}}}"#,
                    fixed, reliability
                ),
            })
            .build()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_client_default() {
        let fake = FakeLlmClient::new();

        assert!(fake.is_available().await);
        assert_eq!(fake.junior_model(), "fake-junior");
        assert_eq!(fake.senior_model(), "fake-senior");
    }

    #[tokio::test]
    async fn test_fake_client_call_counts() {
        let fake = FakeLlmClient::new();

        assert_eq!(fake.junior_call_count(), 0);
        assert_eq!(fake.senior_call_count(), 0);

        let _ = fake.call_junior_v80("test").await;
        assert_eq!(fake.junior_call_count(), 1);

        let _ = fake.call_senior_v80("test").await;
        assert_eq!(fake.senior_call_count(), 1);

        let _ = fake.call_junior_v80("test").await;
        assert_eq!(fake.junior_call_count(), 2);
    }

    #[tokio::test]
    async fn test_fake_client_queued_responses() {
        let fake = FakeLlmClientBuilder::new()
            .junior_response(FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "cpu.info".to_string(),
                    reason: "first".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            })
            .junior_response(FakeJuniorResponse {
                probe_requests: vec![],
                draft_answer: Some(DraftAnswerV80 {
                    text: "second call answer".to_string(),
                    citations: vec![],
                }),
                raw_text: "{}".to_string(),
            })
            .build();

        // First call: probe request
        let (resp1, _) = fake.call_junior_v80("test").await.unwrap();
        assert_eq!(resp1.probe_requests.len(), 1);
        assert!(resp1.draft_answer.is_none());

        // Second call: answer
        let (resp2, _) = fake.call_junior_v80("test").await.unwrap();
        assert!(resp2.probe_requests.is_empty());
        assert!(resp2.draft_answer.is_some());
        assert_eq!(resp2.draft_answer.unwrap().text, "second call answer");

        // Third call: default (queue exhausted)
        let (resp3, _) = fake.call_junior_v80("test").await.unwrap();
        assert_eq!(resp3.draft_answer.unwrap().text, "Fake Junior answer");
    }

    #[tokio::test]
    async fn test_fake_with_direct_answer() {
        let fake = FakeLlmClient::with_direct_answer("Your CPU has 8 cores", 0.95);

        let (junior, _) = fake.call_junior_v80("how many cores?").await.unwrap();
        assert!(junior.probe_requests.is_empty());
        assert_eq!(junior.draft_answer.unwrap().text, "Your CPU has 8 cores");

        let (senior, _) = fake.call_senior_v80("review").await.unwrap();
        assert_eq!(senior.verdict, "approve");
        assert_eq!(senior.scores_overall, 0.95);
    }

    #[tokio::test]
    async fn test_fake_with_probe_request() {
        let fake = FakeLlmClient::with_probe_request("mem.info", "need memory info");

        let (junior, _) = fake.call_junior_v80("how much ram?").await.unwrap();
        assert_eq!(junior.probe_requests.len(), 1);
        assert_eq!(junior.probe_requests[0].probe_id, "mem.info");
        assert!(junior.draft_answer.is_none());
    }

    #[tokio::test]
    async fn test_fake_unavailable() {
        let fake = FakeLlmClient::unavailable();
        assert!(!fake.is_available().await);
    }

    #[tokio::test]
    async fn test_fake_senior_refuse() {
        let fake = FakeLlmClient::with_senior_refuse("Insufficient evidence");

        let (senior, _) = fake.call_senior_v80("review").await.unwrap();
        assert_eq!(senior.verdict, "refuse");
        assert_eq!(senior.scores_overall, 0.0);
    }

    #[tokio::test]
    async fn test_fake_senior_fix() {
        let fake = FakeLlmClient::with_senior_fix("wrong answer", "corrected answer", 0.85);

        let (junior, _) = fake.call_junior_v80("test").await.unwrap();
        assert_eq!(junior.draft_answer.unwrap().text, "wrong answer");

        let (senior, _) = fake.call_senior_v80("review").await.unwrap();
        assert_eq!(senior.verdict, "fix_and_accept");
        assert_eq!(senior.fixed_answer.unwrap(), "corrected answer");
        assert_eq!(senior.scores_overall, 0.85);
    }
}
