//! LLM Output Validation - v0.85.0
//!
//! Part 8: Internal self-test after each LLM exchange.
//! Brain checks output validity, JSON schema, contradictions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result of validating LLM output
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Output is valid
    Valid,
    /// Output is invalid, with reason
    Invalid(String),
    /// Output has contradictions
    Contradiction(String),
    /// JSON schema mismatch
    SchemaMismatch(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// LLM output validator
pub struct LlmValidator;

impl LlmValidator {
    /// Validate Junior output
    pub fn validate_junior(response: &str) -> ValidationResult {
        // Must contain JSON
        if !response.contains('{') {
            return ValidationResult::Invalid("No JSON found in response".to_string());
        }

        // Extract JSON
        let json_str = match Self::extract_json(response) {
            Some(s) => s,
            None => return ValidationResult::Invalid("Could not parse JSON".to_string()),
        };

        // Parse JSON
        let json: Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => return ValidationResult::SchemaMismatch(format!("JSON parse error: {}", e)),
        };

        // Check required fields
        if json.get("action").is_none() {
            return ValidationResult::SchemaMismatch("Missing 'action' field".to_string());
        }

        let action = json.get("action").and_then(|a| a.as_str()).unwrap_or("");
        match action {
            "command" => {
                if json.get("command").is_none() {
                    return ValidationResult::SchemaMismatch("Action 'command' requires 'command' field".to_string());
                }
            }
            "answer" => {
                if json.get("answer").is_none() {
                    return ValidationResult::SchemaMismatch("Action 'answer' requires 'answer' field".to_string());
                }
            }
            "refuse" => {
                // Refuse is valid without additional fields
            }
            _ => {
                return ValidationResult::Invalid(format!("Unknown action: {}", action));
            }
        }

        // Check for score
        if let Some(score) = json.get("score") {
            if let Some(s) = score.as_i64() {
                if !(0..=100).contains(&s) {
                    return ValidationResult::Invalid(format!("Score {} out of range 0-100", s));
                }
            }
        }

        ValidationResult::Valid
    }

    /// Validate Senior output
    pub fn validate_senior(response: &str) -> ValidationResult {
        // Must contain JSON
        if !response.contains('{') {
            return ValidationResult::Invalid("No JSON found in response".to_string());
        }

        // Extract JSON
        let json_str = match Self::extract_json(response) {
            Some(s) => s,
            None => return ValidationResult::Invalid("Could not parse JSON".to_string()),
        };

        // Parse JSON
        let json: Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => return ValidationResult::SchemaMismatch(format!("JSON parse error: {}", e)),
        };

        // Check required fields
        if json.get("verdict").is_none() {
            return ValidationResult::SchemaMismatch("Missing 'verdict' field".to_string());
        }

        let verdict = json.get("verdict").and_then(|v| v.as_str()).unwrap_or("");
        if !["approve", "fix", "refuse"].contains(&verdict) {
            return ValidationResult::Invalid(format!("Unknown verdict: {}", verdict));
        }

        // Check scores are in range
        for field in ["evidence", "coverage", "reasoning"] {
            if let Some(score) = json.get(field) {
                if let Some(s) = score.as_i64() {
                    if !(0..=100).contains(&s) {
                        return ValidationResult::Invalid(format!("{} score {} out of range 0-100", field, s));
                    }
                }
            }
        }

        // Check for contradictions
        if verdict == "approve" {
            let evidence = json.get("evidence").and_then(|e| e.as_i64()).unwrap_or(0);
            let coverage = json.get("coverage").and_then(|c| c.as_i64()).unwrap_or(0);
            let reasoning = json.get("reasoning").and_then(|r| r.as_i64()).unwrap_or(0);

            // Contradiction: approve with low scores
            if evidence < 50 || coverage < 50 || reasoning < 50 {
                return ValidationResult::Contradiction(
                    format!("Verdict 'approve' but scores are low: evidence={}, coverage={}, reasoning={}",
                        evidence, coverage, reasoning)
                );
            }
        }

        if verdict == "refuse" {
            let evidence = json.get("evidence").and_then(|e| e.as_i64()).unwrap_or(0);
            // Contradiction: refuse with high evidence
            if evidence > 90 {
                return ValidationResult::Contradiction(
                    format!("Verdict 'refuse' but evidence is high: {}", evidence)
                );
            }
        }

        ValidationResult::Valid
    }

    /// Extract JSON from mixed text
    fn extract_json(text: &str) -> Option<String> {
        let start = text.find('{')?;
        let end = text.rfind('}')? + 1;
        if end > start {
            Some(text[start..end].to_string())
        } else {
            None
        }
    }

    /// Check for hallucination indicators in answer
    pub fn check_hallucination(answer: &str, evidence: &str) -> ValidationResult {
        let answer_lower = answer.to_lowercase();
        let evidence_lower = evidence.to_lowercase();

        // List of hallucination red flags
        let red_flags = [
            "i believe", "i think", "probably", "might be", "could be",
            "as an ai", "as a language model", "i don't have access",
            "based on my training", "i cannot verify"
        ];

        for flag in red_flags {
            if answer_lower.contains(flag) && !evidence_lower.contains(flag) {
                return ValidationResult::Invalid(
                    format!("Potential hallucination: answer contains '{}' without evidence", flag)
                );
            }
        }

        // Check for specific numbers/facts that should be in evidence
        // Extract numbers from answer
        let answer_numbers: Vec<&str> = answer.split_whitespace()
            .filter(|w| w.chars().all(|c| c.is_numeric() || c == '.'))
            .collect();

        // For each number in answer, check if it appears in evidence
        for num in answer_numbers {
            if num.len() > 2 && !evidence.contains(num) {
                // This is a significant number not found in evidence
                // Could be hallucination, but not always
                // Just log for now, don't reject
            }
        }

        ValidationResult::Valid
    }
}

/// Retry strategy for invalid LLM outputs
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    pub max_retries: u32,
    pub current_retry: u32,
    pub last_error: Option<String>,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 1, // Only retry once per v0.85.0 spec
            current_retry: 0,
            last_error: None,
        }
    }
}

impl RetryStrategy {
    pub fn can_retry(&self) -> bool {
        self.current_retry < self.max_retries
    }

    pub fn record_failure(&mut self, error: &str) {
        self.current_retry += 1;
        self.last_error = Some(error.to_string());
    }

    pub fn reset(&mut self) {
        self.current_retry = 0;
        self.last_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_junior_valid() {
        let response = r#"{"action":"command","command":"lscpu","score":85,"probes":["cpu.info"]}"#;
        assert!(LlmValidator::validate_junior(response).is_valid());
    }

    #[test]
    fn test_validate_junior_missing_action() {
        let response = r#"{"command":"lscpu"}"#;
        assert!(!LlmValidator::validate_junior(response).is_valid());
    }

    #[test]
    fn test_validate_senior_valid() {
        let response = r#"{"verdict":"approve","evidence":95,"coverage":92,"reasoning":98}"#;
        assert!(LlmValidator::validate_senior(response).is_valid());
    }

    #[test]
    fn test_validate_senior_contradiction() {
        let response = r#"{"verdict":"approve","evidence":30,"coverage":40,"reasoning":50}"#;
        let result = LlmValidator::validate_senior(response);
        assert!(matches!(result, ValidationResult::Contradiction(_)));
    }

    #[test]
    fn test_extract_json() {
        let text = "Here is the answer: {\"verdict\":\"approve\"} done";
        let json = LlmValidator::extract_json(text);
        assert_eq!(json, Some("{\"verdict\":\"approve\"}".to_string()));
    }

    #[test]
    fn test_retry_strategy() {
        let mut retry = RetryStrategy::default();
        assert!(retry.can_retry());
        retry.record_failure("test error");
        assert!(!retry.can_retry());
    }

    #[test]
    fn test_hallucination_check() {
        let answer = "The CPU has 8 cores";
        let evidence = "CPU(s): 8";
        assert!(LlmValidator::check_hallucination(answer, evidence).is_valid());

        let bad_answer = "I think the CPU probably has 8 cores";
        let result = LlmValidator::check_hallucination(bad_answer, evidence);
        assert!(!result.is_valid());
    }
}
