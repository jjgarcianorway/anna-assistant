//! Specialist call execution wrapper (v0.0.421).
//!
//! Wraps LLM calls with:
//! - JSON parsing
//! - Validation
//! - Error handling
//! - Timeout tracking

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::answer::AnswerType;
use super::prompt::{build_compact_prompt, build_specialist_prompt, SpecialistPromptConfig};
use super::schema::{SpecialistResponseV2, SpecialistStatus};
use super::validate::{parse_and_validate, ValidationResult};
use super::{FALLBACK_CONFIDENCE, MIN_USEFUL_CONFIDENCE, SPECIALIST_TIMEOUT_MS};

/// Result of a specialist call
#[derive(Debug, Clone)]
pub struct SpecialistCallResult {
    /// The response (either from LLM or fallback)
    pub response: SpecialistResponseV2,
    /// Whether this came from fallback
    pub used_fallback: bool,
    /// Whether this came from deterministic path
    pub used_deterministic: bool,
    /// Duration of the call in milliseconds
    pub duration_ms: u64,
    /// Any errors encountered (logged, not shown to user)
    pub internal_errors: Vec<String>,
    /// Source: "llm", "fallback", "deterministic"
    pub source: String,
}

impl SpecialistCallResult {
    /// Create a successful LLM result
    pub fn from_llm(response: SpecialistResponseV2, duration_ms: u64) -> Self {
        Self {
            response,
            used_fallback: false,
            used_deterministic: false,
            duration_ms,
            internal_errors: vec![],
            source: "llm".to_string(),
        }
    }

    /// Create a fallback result
    pub fn from_fallback(response: SpecialistResponseV2, errors: Vec<String>) -> Self {
        Self {
            response,
            used_fallback: true,
            used_deterministic: false,
            duration_ms: 0,
            internal_errors: errors,
            source: "fallback".to_string(),
        }
    }

    /// Create a deterministic result
    pub fn from_deterministic(response: SpecialistResponseV2) -> Self {
        Self {
            response,
            used_fallback: false,
            used_deterministic: true,
            duration_ms: 0,
            internal_errors: vec![],
            source: "deterministic".to_string(),
        }
    }

    /// Check if result is usable
    pub fn is_usable(&self) -> bool {
        self.response.is_successful() || self.response.confidence >= MIN_USEFUL_CONFIDENCE
    }
}

/// Specialist call configuration
#[derive(Debug, Clone)]
pub struct SpecialistCall {
    /// Domain: performance, network, storage, etc.
    pub domain: String,
    /// Question type detected from intent
    pub question_type: AnswerType,
    /// The user's question
    pub question: String,
    /// Normalized intent
    pub intent: String,
    /// Probe results (probe_id -> output)
    pub probes: HashMap<String, String>,
    /// Knowledge snippets (source -> content)
    pub knowledge: HashMap<String, String>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

impl SpecialistCall {
    /// Create a new specialist call
    pub fn new(domain: &str, question: &str, intent: &str) -> Self {
        Self {
            domain: domain.to_string(),
            question_type: AnswerType::from_intent(intent),
            question: question.to_string(),
            intent: intent.to_string(),
            probes: HashMap::new(),
            knowledge: HashMap::new(),
            timeout_ms: SPECIALIST_TIMEOUT_MS,
        }
    }

    /// Add probe result
    pub fn with_probe(mut self, probe_id: &str, output: &str) -> Self {
        self.probes.insert(probe_id.to_string(), output.to_string());
        self
    }

    /// Add multiple probes
    pub fn with_probes(mut self, probes: HashMap<String, String>) -> Self {
        self.probes.extend(probes);
        self
    }

    /// Add knowledge snippet
    pub fn with_knowledge(mut self, source: &str, content: &str) -> Self {
        self.knowledge
            .insert(source.to_string(), content.to_string());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Build the full prompt for LLM
    pub fn build_prompt(&self) -> String {
        let config = SpecialistPromptConfig::for_domain(&self.domain)
            .with_question_type(&format!("{:?}", self.question_type).to_lowercase());

        let mut prompt = build_specialist_prompt(&config);

        // Add context
        prompt.push_str("\n\n## Context\n");
        prompt.push_str(&format!("Question: {}\n", self.question));
        prompt.push_str(&format!("Intent: {}\n", self.intent));

        // Add probe results
        if !self.probes.is_empty() {
            prompt.push_str("\n## Probe Results\n");
            for (probe_id, output) in &self.probes {
                prompt.push_str(&format!("### {}\n```\n{}\n```\n", probe_id, output));
            }
        }

        // Add knowledge
        if !self.knowledge.is_empty() {
            prompt.push_str("\n## Knowledge\n");
            for (source, content) in &self.knowledge {
                let truncated = if content.len() > 1000 {
                    format!("{}...", &content[..1000])
                } else {
                    content.clone()
                };
                prompt.push_str(&format!("### {}\n{}\n", source, truncated));
            }
        }

        prompt.push_str("\n\nReturn ONLY the JSON response:");
        prompt
    }

    /// Build a compact prompt for retry attempts
    pub fn build_compact_prompt(&self) -> String {
        let mut prompt = build_compact_prompt(
            &self.domain,
            &format!("{:?}", self.question_type).to_lowercase(),
        );

        prompt.push_str(&format!("\n\nQuestion: {}\n", self.question));

        // Only include key probes (first 500 chars each)
        if !self.probes.is_empty() {
            prompt.push_str("\nProbes:\n");
            for (probe_id, output) in &self.probes {
                let truncated = if output.len() > 500 {
                    format!("{}...", &output[..500])
                } else {
                    output.clone()
                };
                prompt.push_str(&format!("{}: {}\n", probe_id, truncated));
            }
        }

        prompt
    }

    /// Process LLM response
    pub fn process_response(
        &self,
        raw_response: &str,
    ) -> Result<(SpecialistResponseV2, ValidationResult), String> {
        parse_and_validate(raw_response)
    }
}

/// Timeout tracking for specialist calls
#[derive(Debug)]
pub struct CallTimer {
    start: Instant,
    timeout: Duration,
}

impl CallTimer {
    /// Create a new timer
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            start: Instant::now(),
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    /// Check if timeout exceeded
    pub fn is_timed_out(&self) -> bool {
        self.start.elapsed() > self.timeout
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get remaining time in milliseconds
    pub fn remaining_ms(&self) -> u64 {
        let elapsed = self.start.elapsed();
        if elapsed > self.timeout {
            0
        } else {
            (self.timeout - elapsed).as_millis() as u64
        }
    }
}

/// Log entry for specialist call (for debugging)
#[derive(Debug, Clone)]
pub struct SpecialistCallLog {
    /// Domain
    pub domain: String,
    /// Intent
    pub intent: String,
    /// Question (truncated)
    pub question_preview: String,
    /// Probes used
    pub probes_used: Vec<String>,
    /// Duration in ms
    pub duration_ms: u64,
    /// Whether it succeeded
    pub success: bool,
    /// Source: llm, fallback, deterministic
    pub source: String,
    /// Errors if any
    pub errors: Vec<String>,
}

impl SpecialistCallLog {
    /// Create from call result
    pub fn from_result(call: &SpecialistCall, result: &SpecialistCallResult) -> Self {
        Self {
            domain: call.domain.clone(),
            intent: call.intent.clone(),
            question_preview: if call.question.len() > 50 {
                format!("{}...", &call.question[..50])
            } else {
                call.question.clone()
            },
            probes_used: call.probes.keys().cloned().collect(),
            duration_ms: result.duration_ms,
            success: result.response.is_successful(),
            source: result.source.clone(),
            errors: result.internal_errors.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialist_call_builder() {
        let call = SpecialistCall::new("performance", "How much free RAM?", "show_memory")
            .with_probe("free", "Mem: 32Gi 15Gi 17Gi")
            .with_timeout(3000);

        assert_eq!(call.domain, "performance");
        assert_eq!(call.question_type, AnswerType::Fact);
        assert!(call.probes.contains_key("free"));
    }

    #[test]
    fn test_build_prompt() {
        let call = SpecialistCall::new("services", "Any failed services?", "check_failed_services")
            .with_probe("systemctl_failed", "0 loaded units");

        let prompt = call.build_prompt();
        assert!(prompt.contains("Specialist Instructions"));
        assert!(prompt.contains("Any failed services?"));
        assert!(prompt.contains("systemctl_failed"));
    }

    #[test]
    fn test_call_timer() {
        let timer = CallTimer::new(100);
        assert!(!timer.is_timed_out());
        assert!(timer.elapsed_ms() < 100);
    }

    #[test]
    fn test_process_response() {
        let call = SpecialistCall::new("performance", "Test", "test");

        let json = r#"{"status": "ok", "confidence": 0.9, "direct_answer": {"short_text": "Test answer"}, "key_findings": [], "citations": ["probe:test"]}"#;

        let result = call.process_response(json);
        assert!(result.is_ok());

        let (response, validation) = result.unwrap();
        assert_eq!(response.status, SpecialistStatus::Ok);
        assert!(validation.is_valid);
    }
}
